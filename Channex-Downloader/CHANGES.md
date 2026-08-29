# Changes

Commit history for Channex Downloader, carried over from its original repository ([github.com/niruxx/Channex-Downloader](https://github.com/niruxx/Channex-Downloader)) before it was merged into this monorepo. Newest first.

## 2026-08-17 — `81aba62` Refresh UI and add theme switching

Modernizes the app shell with a new theme system that supports light, dark, and system preferences, persisted in local storage and synced with OS changes. Updates shared color tokens, surfaces, borders, and typography for a cleaner, more polished look across the app. The media grid, toolbar, modal, and settings panel were restyled for improved readability and consistency, along with refreshed empty/loading states and card interactions.

## 2026-08-17 — `917cadc` Rename app branding to Channex Downloader

Updates the app's package metadata, window titles, HTML document title, and README references from the old `chan_dl` branding to the official Channex Downloader naming for consistency across the desktop app and release instructions.

## 2026-08-13 — `e4d67ee` Bump version to 0.2.0

Updates application version across all configuration files: `package.json`, `Cargo.toml`, `tauri.conf.json`, and their respective lock files.

## 2026-08-12 — `b92559b` Add preview modal and backend media proxy

Introduces an in-app preview modal with navigation and single-file download support. Adds `PreviewModal` component, `useMediaBlob` hook, and `MediaCard` double-click to open previews. Implements `fetch_media_bytes` in the Tauri backend and exposes `fetchMediaBytes` in `src/lib/api` to proxy media bytes (works around CDN CORS issues). Adds `mimeForExt` helper, preview state and navigation, and `downloadSingle` in the app store.

## 2026-08-11 — `88497ee` Add installation instructions to README

Adds a comprehensive "How to install" section documenting installation procedures for Linux (Debian/Ubuntu, Fedora/RHEL, Arch Linux, AppImage), Windows, and macOS, directing users to pre-built binaries on GitHub Releases with platform-specific installation commands.

## 2026-08-02 — `4707558` Rebrand to chan_dl; add README screenshots

Replaces the generic Tauri+React template README with a full `chan_dl` project README (features, requirements, OS support, build/run instructions). Adds three screenshots under `docs/screenshots`. Updates `index.html` title to "chan_dl" and removes the Vite favicon link. Removes unused public SVGs.

## 2026-08-02 — `4e20367` Add chan_dl Tauri + React app

Scaffolds a full Tauri + React + TypeScript application implementing a chan thread media downloader. Adds the frontend (`src/`) with components, store, API bridge, types, Vite/TS configs, CSS and assets; backend Rust (`src-tauri/`) with thread parsers (4chan, lynxchan, vichan), downloader, settings and Tauri commands; `package.json` and lockfile; icons and Tauri config. Implements thread parsing, download batching with concurrency, progress/done/error events, settings persistence, and UI for selecting and downloading media.

## 2026-08-01 — `e57724a` Initial commit
