import { getCurrentWindow } from "@tauri-apps/api/window";

export const appWindow = getCurrentWindow();

export async function startDrag() {
  await appWindow.startDragging();
}

export async function minimizeWindow() {
  await appWindow.minimize();
}

export async function toggleMaximizeWindow() {
  await appWindow.toggleMaximize();
}

export async function closeWindow() {
  await appWindow.close();
}

/** Bypasses the closeRequested event — use once a close has already been confirmed/animated. */
export async function destroyWindow() {
  await appWindow.destroy();
}
