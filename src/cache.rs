//! Keeping the on-disk image cache (thumbnails and site icons) in check,
//! per Preferences → Storage: clear it on demand, clear it on a schedule,
//! and evict the least recently used images when it grows past a size
//! limit. Checked at startup and every 10 minutes. Videos are streamed,
//! never stored, so they don't appear here.

use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use channex::data::cache::{CachedFile, clear_due, files_to_evict};
use gtk::gio;
use gtk::glib;

use crate::state::state;
use crate::thumbnails;

const CHECK_EVERY: Duration = Duration::from_secs(10 * 60);

fn dir() -> PathBuf {
    thumbnails::cache_dir()
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}

/// Every cached file with its size and last-used time.
fn list() -> Vec<(PathBuf, CachedFile)> {
    let Ok(entries) = std::fs::read_dir(dir()) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter_map(|entry| {
            let meta = entry.metadata().ok()?;
            if !meta.is_file() {
                return None;
            }
            let modified = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map_or(0, |d| d.as_secs() as i64);
            Some((
                entry.path(),
                CachedFile {
                    size: meta.len(),
                    modified,
                },
            ))
        })
        .collect()
}

/// Bytes used by the cache (measured off the main thread).
pub async fn usage() -> u64 {
    gio::spawn_blocking(|| list().iter().map(|(_, f)| f.size).sum())
        .await
        .unwrap_or(0)
}

/// Deletes every cached image, from memory and disk, and restarts the
/// clearing schedule.
pub async fn clear() {
    thumbnails::clear_memory();
    let _ = gio::spawn_blocking(|| {
        for (path, _) in list() {
            let _ = std::fs::remove_file(path);
        }
    })
    .await;
    let now = now();
    state().update_settings(|d| d.settings.cache_last_cleared = now);
}

/// Applies the schedule and size limit once.
pub fn maintain() {
    let (interval, limit_mb, last) = {
        let st = state();
        let s = &st.data().settings;
        (
            s.cache_clear_interval,
            s.cache_size_limit_mb,
            s.cache_last_cleared,
        )
    };
    let now = now();
    if last == 0 {
        // First run with these settings: start the schedule's clock.
        state().update_settings(|d| d.settings.cache_last_cleared = now);
    } else if clear_due(interval, last, now) {
        glib::spawn_future_local(clear());
        return;
    }
    if limit_mb > 0 {
        let limit = u64::from(limit_mb) * 1024 * 1024;
        gio::spawn_blocking(move || {
            let files = list();
            let infos: Vec<CachedFile> = files.iter().map(|(_, f)| f.clone()).collect();
            for i in files_to_evict(&infos, limit) {
                let _ = std::fs::remove_file(&files[i].0);
            }
        });
    }
}

/// Runs [`maintain`] now and every 10 minutes while the app is open.
pub fn start() {
    maintain();
    glib::timeout_add_local(CHECK_EVERY, || {
        maintain();
        glib::ControlFlow::Continue
    });
}
