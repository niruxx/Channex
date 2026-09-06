# Channex

This repository holds two independent desktop apps, each in its own directory with its own dependencies, build config, and README. They share no code and are versioned/built separately — treat each subdirectory as its own project root.

## Directories

### [Channex/](Channex/) — Imageboarder

A desktop viewer for chan-style imageboards (4chan, 8kun, 8chan.moe, Lainchan, and other 4chan-API-compatible or LynxChan-based sites). Built with Tauri 2, React 19, TypeScript, and Tailwind CSS v4.

See [Channex/README.md](Channex/README.md) for features and setup.

### [Channex-Downloader/](Channex-Downloader/) — Channex Downloader

A companion desktop app for bulk-downloading media from imageboard threads: paste a thread link, preview every image/video in a thumbnail grid, and download what you want. Built with Tauri, React, TypeScript, and Tailwind CSS.

Originally maintained at [github.com/niruxx/Channex-Downloader](https://github.com/niruxx/Channex-Downloader), now developed here alongside Channex.

See [Channex-Downloader/README.md](Channex-Downloader/README.md) for features and setup.

## Working in this repo

Each project keeps its own `package.json`, `package-lock.json`, and `src-tauri/` (Rust/Tauri) setup — install and run them independently:

```sh
cd Channex
npm install
npm run tauri dev
```

```sh
cd Channex-Downloader
npm install
npm run tauri dev
```

The root `.gitignore` covers common editor/OS/build clutter (`node_modules/`, `dist/`, `src-tauri/target/`, `.vscode/`, `.idea/`, etc.) across both directories.

## AI usage disclaimer

Parts of this repository (code, documentation, and/or assets) were written or assisted by AI tools. Review changes accordingly.
