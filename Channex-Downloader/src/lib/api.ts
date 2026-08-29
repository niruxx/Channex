import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { AppSettings, ThreadInfo } from "../types";

export function fetchThreadInfo(url: string): Promise<ThreadInfo> {
  return invoke("fetch_thread_info", { url });
}

export function pickDirectory(): Promise<string | null> {
  return invoke("pick_directory");
}

export function defaultDownloadDir(): Promise<string | null> {
  return invoke("default_download_dir");
}

export function revealInFileManager(path: string): Promise<void> {
  return invoke("reveal_in_file_manager", { path });
}

export interface DownloadRequestItem {
  id: string;
  url: string;
  filename: string;
}

export function downloadFiles(
  items: DownloadRequestItem[],
  destDir: string,
  concurrency: number,
): Promise<void> {
  return invoke("download_files", { items, destDir, concurrency });
}

export function cancelDownloads(): Promise<void> {
  return invoke("cancel_downloads");
}

export interface ProgressEvent {
  id: string;
  downloaded: number;
  total: number | null;
}

export interface DoneEvent {
  id: string;
  path: string;
}

export interface ErrorEvent {
  id: string;
  message: string;
}

export function onDownloadProgress(cb: (p: ProgressEvent) => void) {
  return listen<ProgressEvent>("chan-dl://download-progress", (e) => cb(e.payload));
}

export function onDownloadDone(cb: (p: DoneEvent) => void) {
  return listen<DoneEvent>("chan-dl://download-done", (e) => cb(e.payload));
}

export function onDownloadError(cb: (p: ErrorEvent) => void) {
  return listen<ErrorEvent>("chan-dl://download-error", (e) => cb(e.payload));
}

export function loadSettings(): Promise<AppSettings> {
  return invoke("load_settings");
}

export function saveSettings(settings: AppSettings): Promise<void> {
  return invoke("save_settings", { settings });
}

/**
 * Fetches a remote media file's raw bytes through the Rust backend instead of
 * letting the webview request it directly. Some imageboard CDNs omit CORS
 * headers on full-resolution media (even though their thumbnails include
 * them), which makes the webview block a direct `<img>`/`<video>` load.
 */
export function fetchMediaBytes(url: string): Promise<ArrayBuffer> {
  return invoke<ArrayBuffer>("fetch_media_bytes", { url });
}
