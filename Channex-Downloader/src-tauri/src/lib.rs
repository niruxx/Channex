mod chan;
mod downloader;
mod settings;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use downloader::DownloadItem;
use settings::AppSettings;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

struct AppState {
    client: reqwest::Client,
    cancel_flag: Arc<AtomicBool>,
}

#[tauri::command]
async fn fetch_thread_info(
    state: State<'_, AppState>,
    url: String,
) -> Result<chan::ThreadInfo, String> {
    chan::fetch_thread(&state.client, &url).await
}

#[tauri::command]
async fn download_files(
    app: AppHandle,
    state: State<'_, AppState>,
    items: Vec<DownloadItem>,
    dest_dir: String,
    concurrency: usize,
) -> Result<(), String> {
    let client = state.client.clone();
    let cancel_flag = state.cancel_flag.clone();
    downloader::run_batch(app, client, items, dest_dir, concurrency, cancel_flag).await
}

#[tauri::command]
fn cancel_downloads(state: State<'_, AppState>) {
    state.cancel_flag.store(true, Ordering::SeqCst);
}

#[tauri::command]
async fn pick_directory(app: AppHandle) -> Option<String> {
    let folder = app.dialog().file().blocking_pick_folder();
    folder.map(|p| p.to_string())
}

#[tauri::command]
fn default_download_dir(app: AppHandle) -> Option<String> {
    app.path()
        .download_dir()
        .ok()
        .map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
fn reveal_in_file_manager(app: AppHandle, path: String) -> Result<(), String> {
    app.opener()
        .reveal_item_in_dir(&path)
        .map_err(|e| e.to_string())
}

/// Fetches a remote media file's bytes through our own HTTP client rather than
/// letting the webview load it directly. Some imageboard CDNs (e.g. 4chan's
/// full-resolution images) omit CORS headers on that endpoint even though
/// their thumbnails include them, which makes Chromium/WebView2 block the
/// direct `<img>`/`<video>` load with a cross-origin error. Proxying through
/// Rust sidesteps that entirely, the same way thread-fetching already does.
#[tauri::command]
async fn fetch_media_bytes(
    state: State<'_, AppState>,
    url: String,
) -> Result<tauri::ipc::Response, String> {
    let resp = state
        .client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Server responded with {}", resp.status()));
    }

    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("Download failed: {e}"))?;

    Ok(tauri::ipc::Response::new(bytes.to_vec()))
}

#[tauri::command]
fn load_settings(app: AppHandle) -> AppSettings {
    settings::load(&app)
}

#[tauri::command]
fn save_settings(app: AppHandle, settings: AppSettings) -> Result<(), String> {
    settings::save(&app, &settings)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            client: reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (compatible; chan_dl/0.1)")
                .build()
                .expect("failed to build http client"),
            cancel_flag: Arc::new(AtomicBool::new(false)),
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            fetch_thread_info,
            download_files,
            cancel_downloads,
            pick_directory,
            default_download_dir,
            reveal_in_file_manager,
            fetch_media_bytes,
            load_settings,
            save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
