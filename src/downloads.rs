//! Downloading thread files.
//!
//! A *job* is a batch of files saved into one folder (usually a
//! thread's). At most 4 files download at once across all jobs; each is
//! streamed to `<name>.part` and renamed when complete, so a cancelled
//! or failed download never leaves a truncated file under the real name.
//! Names are made unique up front (`image (1).png`), against both the
//! folder and the rest of the batch.
//!
//! The job list lasts for the session. Signals:
//! - `changed(job_id: u64)` - a job was added, updated or removed
//!   (`0` = several at once)
//! - `job-finished(job_id: u64)` - every file in a job is done or failed

use std::cell::{Cell, OnceCell, RefCell};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use adw::prelude::*;
use channex::data::downloads::{sanitize_file_name, unique_name};
use channex::data::http::HttpClient;
use gtk::glib;
use gtk::glib::subclass::Signal;
use gtk::subclass::prelude::*;
use tokio::io::AsyncWriteExt;
use tokio::sync::Semaphore;

use crate::runtime;
use crate::state::state;

const MAX_PARALLEL: usize = 4;

#[derive(Debug, Clone, PartialEq)]
pub enum FileStatus {
    Pending,
    Done,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct FileTask {
    pub url: String,
    /// Final (unique, sanitized) file name in the job's folder.
    pub name: String,
    pub status: FileStatus,
}

#[derive(Debug, Clone)]
pub struct Job {
    pub id: u64,
    pub label: String,
    pub dir: PathBuf,
    pub files: Vec<FileTask>,
}

impl Job {
    pub fn total(&self) -> usize {
        self.files.len()
    }
    pub fn done(&self) -> usize {
        self.files
            .iter()
            .filter(|f| f.status == FileStatus::Done)
            .count()
    }
    pub fn failed(&self) -> usize {
        self.files
            .iter()
            .filter(|f| matches!(f.status, FileStatus::Failed(_)))
            .count()
    }
    pub fn finished(&self) -> bool {
        self.files.iter().all(|f| f.status != FileStatus::Pending)
    }
    /// Fraction of files that are done or failed.
    pub fn progress(&self) -> f64 {
        if self.files.is_empty() {
            1.0
        } else {
            (self.done() + self.failed()) as f64 / self.total() as f64
        }
    }
}

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct DownloadManager {
        pub jobs: RefCell<Vec<Job>>,
        pub next_id: Cell<u64>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DownloadManager {
        const NAME: &'static str = "ChannexDownloadManager";
        type Type = super::DownloadManager;
    }

    impl ObjectImpl for DownloadManager {
        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder("changed")
                        .param_types([u64::static_type()])
                        .build(),
                    Signal::builder("job-finished")
                        .param_types([u64::static_type()])
                        .build(),
                ]
            })
        }
    }
}

glib::wrapper! {
    pub struct DownloadManager(ObjectSubclass<imp::DownloadManager>);
}

thread_local! {
    static MANAGER: OnceCell<DownloadManager> = const { OnceCell::new() };
}

/// The app's download manager.
pub fn downloads() -> DownloadManager {
    MANAGER.with(|m| m.get_or_init(glib::Object::new).clone())
}

fn semaphore() -> Arc<Semaphore> {
    static SEM: OnceLock<Arc<Semaphore>> = OnceLock::new();
    SEM.get_or_init(|| Arc::new(Semaphore::new(MAX_PARALLEL)))
        .clone()
}

/// Where downloads go: the folder chosen in Preferences, else
/// `~/Downloads/Channex`.
pub fn base_dir() -> PathBuf {
    let chosen = state().data().settings.download_dir.clone();
    if !chosen.trim().is_empty() {
        return PathBuf::from(chosen);
    }
    glib::user_special_dir(glib::UserDirectory::Downloads)
        .unwrap_or_else(|| glib::home_dir().join("Downloads"))
        .join("Channex")
}

impl DownloadManager {
    pub fn jobs(&self) -> Vec<Job> {
        self.imp().jobs.borrow().clone()
    }

    pub fn job(&self, id: u64) -> Option<Job> {
        self.imp()
            .jobs
            .borrow()
            .iter()
            .find(|j| j.id == id)
            .cloned()
    }

    pub fn active_count(&self) -> usize {
        self.imp()
            .jobs
            .borrow()
            .iter()
            .filter(|j| !j.finished())
            .count()
    }

    /// Starts downloading `files` (URL, original name) into `dir`.
    /// Returns the job id.
    pub fn start_job(&self, label: &str, dir: PathBuf, files: Vec<(String, String)>) -> u64 {
        let imp = self.imp();
        let id = imp.next_id.get() + 1;
        imp.next_id.set(id);

        let mut taken = HashSet::new();
        let tasks: Vec<FileTask> = files
            .into_iter()
            .map(|(url, original)| {
                let safe = sanitize_file_name(&original);
                let name = unique_name(&safe, &taken, |candidate| dir.join(candidate).exists());
                taken.insert(name.to_lowercase());
                FileTask {
                    url,
                    name,
                    status: FileStatus::Pending,
                }
            })
            .collect();
        let count = tasks.len();
        imp.jobs.borrow_mut().push(Job {
            id,
            label: label.to_owned(),
            dir,
            files: tasks,
        });
        self.emit_changed(id);
        for index in 0..count {
            self.spawn_file(id, index);
        }
        id
    }

    /// Re-queues a job's failed files.
    pub fn retry_failed(&self, id: u64) {
        let retry: Vec<usize> = {
            let mut jobs = self.imp().jobs.borrow_mut();
            let Some(job) = jobs.iter_mut().find(|j| j.id == id) else {
                return;
            };
            job.files
                .iter_mut()
                .enumerate()
                .filter(|(_, f)| matches!(f.status, FileStatus::Failed(_)))
                .map(|(i, f)| {
                    f.status = FileStatus::Pending;
                    i
                })
                .collect()
        };
        self.emit_changed(id);
        for index in retry {
            self.spawn_file(id, index);
        }
    }

    /// Removes a finished job from the list (its files stay on disk).
    pub fn remove_job(&self, id: u64) {
        self.imp()
            .jobs
            .borrow_mut()
            .retain(|j| j.id != id || !j.finished());
        self.emit_changed(id);
    }

    /// Removes every finished job from the list (files stay on disk).
    pub fn clear_finished(&self) {
        self.imp().jobs.borrow_mut().retain(|j| !j.finished());
        self.emit_changed(0);
    }

    fn emit_changed(&self, id: u64) {
        self.emit_by_name::<()>("changed", &[&id]);
    }

    fn spawn_file(&self, job_id: u64, index: usize) {
        let Some((url, path)) = self.job(job_id).and_then(|j| {
            j.files
                .get(index)
                .map(|f| (f.url.clone(), j.dir.join(&f.name)))
        }) else {
            return;
        };
        let client = state().http();
        let this = self.clone();
        glib::spawn_future_local(async move {
            let result =
                runtime::spawn(async move { download_to(&client, &url, &path).await }).await;
            let finished = {
                let mut jobs = this.imp().jobs.borrow_mut();
                let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) else {
                    return;
                };
                if let Some(file) = job.files.get_mut(index) {
                    file.status = match result {
                        Ok(()) => FileStatus::Done,
                        Err(err) => FileStatus::Failed(err),
                    };
                }
                job.finished()
            };
            this.emit_changed(job_id);
            if finished {
                this.emit_by_name::<()>("job-finished", &[&job_id]);
            }
        });
    }
}

/// Streams `url` to `path` via `<path>.part`.
async fn download_to(client: &HttpClient, url: &str, path: &Path) -> Result<(), String> {
    let _permit = semaphore()
        .acquire_owned()
        .await
        .map_err(|e| e.to_string())?;
    stream_to_file(client, url, path).await
}

/// Streams `url` to `path` via `<path>.part`, so a failed or interrupted
/// download never leaves a truncated file under the real name. Shared
/// with the media viewer's cache (which doesn't queue behind downloads).
pub async fn stream_to_file(client: &HttpClient, url: &str, path: &Path) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        tokio::fs::create_dir_all(dir)
            .await
            .map_err(|e| format!("Couldn't create the folder: {e}"))?;
    }
    let part = path.with_file_name(format!(
        "{}.part",
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("download")
    ));

    let result = async {
        let mut response = client
            .reqwest()
            .get(url)
            .send()
            .await
            .and_then(|r| r.error_for_status())
            .map_err(|e| match e.status() {
                Some(status) => format!("The server answered {}", status.as_u16()),
                None => "Couldn't connect".to_owned(),
            })?;
        let mut file = tokio::fs::File::create(&part)
            .await
            .map_err(|e| format!("Couldn't write the file: {e}"))?;
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| "The download was interrupted".to_owned())?
        {
            file.write_all(&chunk)
                .await
                .map_err(|e| format!("Couldn't write the file: {e}"))?;
        }
        file.flush()
            .await
            .map_err(|e| format!("Couldn't write the file: {e}"))?;
        tokio::fs::rename(&part, path)
            .await
            .map_err(|e| format!("Couldn't write the file: {e}"))
    }
    .await;

    if result.is_err() {
        let _ = tokio::fs::remove_file(&part).await;
    }
    result
}
