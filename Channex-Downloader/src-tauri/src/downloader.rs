use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::Semaphore;

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DownloadItem {
    pub id: String,
    pub url: String,
    pub filename: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ProgressPayload {
    id: String,
    downloaded: u64,
    total: Option<u64>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct DonePayload {
    id: String,
    path: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ErrorPayload {
    id: String,
    message: String,
}

pub async fn run_batch(
    app: AppHandle,
    client: reqwest::Client,
    items: Vec<DownloadItem>,
    dest_dir: String,
    concurrency: usize,
    cancel_flag: Arc<AtomicBool>,
) -> Result<(), String> {
    cancel_flag.store(false, Ordering::SeqCst);
    let dest = PathBuf::from(dest_dir);
    tokio::fs::create_dir_all(&dest)
        .await
        .map_err(|e| format!("Couldn't create destination folder: {e}"))?;

    let semaphore = Arc::new(Semaphore::new(concurrency.max(1)));
    let mut handles = Vec::with_capacity(items.len());
    for item in items {
        let permit_sem = semaphore.clone();
        let app = app.clone();
        let client = client.clone();
        let dest = dest.clone();
        let cancel_flag = cancel_flag.clone();

        handles.push(tokio::spawn(async move {
            let _permit = permit_sem.acquire_owned().await;
            if cancel_flag.load(Ordering::SeqCst) {
                return;
            }
            if let Err(message) = download_one(&app, &client, &item, &dest, &cancel_flag).await {
                let _ = app.emit(
                    "chan-dl://download-error",
                    ErrorPayload {
                        id: item.id.clone(),
                        message,
                    },
                );
            }
        }));
    }

    for handle in handles {
        let _ = handle.await;
    }
    Ok(())
}

async fn download_one(
    app: &AppHandle,
    client: &reqwest::Client,
    item: &DownloadItem,
    dest_dir: &PathBuf,
    cancel_flag: &Arc<AtomicBool>,
) -> Result<(), String> {
    let safe_name = sanitize_filename::sanitize(&item.filename);
    let target_path = dest_dir.join(&safe_name);

    let resp = client
        .get(&item.url)
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Server responded with {}", resp.status()));
    }

    let total = resp.content_length();
    let mut file = File::create(&target_path)
        .await
        .map_err(|e| format!("Couldn't create file: {e}"))?;

    let mut stream = resp.bytes_stream();
    let mut downloaded: u64 = 0;
    let mut last_emit = Instant::now();

    while let Some(chunk) = stream.next().await {
        if cancel_flag.load(Ordering::SeqCst) {
            drop(file);
            let _ = tokio::fs::remove_file(&target_path).await;
            return Err("Cancelled".to_string());
        }
        let chunk = chunk.map_err(|e| format!("Download interrupted: {e}"))?;
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("Disk write failed: {e}"))?;
        downloaded += chunk.len() as u64;

        if last_emit.elapsed().as_millis() >= 100 {
            let _ = app.emit(
                "chan-dl://download-progress",
                ProgressPayload {
                    id: item.id.clone(),
                    downloaded,
                    total,
                },
            );
            last_emit = Instant::now();
        }
    }

    file.flush().await.map_err(|e| format!("Disk write failed: {e}"))?;

    let _ = app.emit(
        "chan-dl://download-progress",
        ProgressPayload {
            id: item.id.clone(),
            downloaded,
            total,
        },
    );
    let _ = app.emit(
        "chan-dl://download-done",
        DonePayload {
            id: item.id.clone(),
            path: target_path.to_string_lossy().to_string(),
        },
    );

    Ok(())
}
