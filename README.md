# Channex Qt6 (placeholder)

A from-scratch **native Qt6 / C++ / QML** rebuild of the Channex imageboard
viewer, aiming for feature parity with the Tauri + React app in
[`Channex/`](../) one level up. This directory is a **placeholder** — it
is not a replacement yet. The plan is to keep building this out until it
covers everything the Tauri version does, then swap it in as the "real"
Channex.

Nothing here shares code, data files, or a build system with the Tauri
app. It has its own settings file (`channex-qt-data.json`, under the
OS's app-data directory) so the two can be run side by side during
development without clobbering each other's state.

## Why Qt6 / C++ / QML

Chosen for closest visual/animation fidelity to the existing app's
custom titlebar, accent theming, and animated backgrounds, while being
a fully native compiled binary (no embedded Chromium/webview runtime,
no Python interpreter).

## Architecture

The Tauri app's Rust side (`src-tauri/`) turned out to carry almost no
business logic — it's a bare shell that just registers Tauri plugins
(HTTP, fs, dialog, store, opener, clipboard). Essentially everything —
API fetching, JSON normalization, downloads, settings persistence,
posting — lives in the TypeScript frontend instead. This port follows
that shape: C++ owns everything (there's no separate "backend"), and
QML is purely the view layer.

| Concept (Tauri app, `src/...`) | Qt6 port (`src/...`) |
|---|---|
| `types.ts` (`ChanSite`/`Board`/`Post`/`PostFile`/`ChanAdapter`) | `adapters/ChanAdapter.h` (same field names, `QVariantMap`-based) |
| `lib/adapters/yotsuba.ts` | `adapters/YotsubaAdapter.{h,cpp}` |
| `lib/adapters/lynxchan.ts` | `adapters/LynxchanAdapter.{h,cpp}` |
| `lib/sanitize.ts` (DOMPurify + quote-link hook) | `adapters/CommentFormatter.{h,cpp}` (regex-based, see caveat below) |
| `store/useSitesStore.ts` + `lib/sites.ts` | `sites/SitesController.{h,cpp}` |
| `hooks/useCatalog.ts` + `components/catalog/sort.ts` | `catalog/CatalogController.{h,cpp}` (`QAbstractListModel`) |
| `hooks/useThread.ts` + backlink scan in `ThreadView.tsx` | `thread/ThreadController.{h,cpp}` (`QAbstractListModel`) |
| `store/useBookmarksStore.ts` | `bookmarks/BookmarksController.{h,cpp}` |
| `store/useDownloadsStore.ts` + `lib/download.ts` | `downloads/DownloadManager.{h,cpp}` |
| `store/useLightboxStore.ts` + fetch-bytes trick in `MediaLightbox.tsx` | `lightbox/LightboxController.{h,cpp}` + `LightboxImageProvider` |
| `lib/lynxchanPosting.ts` | `posting/LynxchanPoster.{h,cpp}` |
| `lib/externalReply.ts` | `posting/ExternalReply.{h,cpp}` (scope-reduced, see below) |
| `store/useNavStore.ts` | `navigation/NavigationController.{h,cpp}` |
| `lib/persist.ts` (`imageboarder-data.json`) | `settings/SettingsManager.{h,cpp}` (`channex-qt-data.json`) |

QML mirrors the component tree under `src/components/` fairly directly
(`qml/components/`, `qml/views/`), with `Theme.qml` standing in for the
CSS custom-property theme system and `qml/MediaLightbox.qml` mounted
once at the window root, same as upstream.

## What works

- Site switching across the 4 presets (4chan, 8kun, 8chan.moe,
  Lainchan) plus adding/removing custom sites, persisted to disk.
- Board discovery per site with fallback to built-in default boards.
- Catalog view (grid/compact/list), live search, 4 sort modes with
  stickies pinned first, NSFW/spoiler blur-until-tapped.
- Thread view: OP + replies, quote-link and backlink navigation
  (via QML's native `Text.onLinkActivated`, see caveat below),
  per-file download buttons, "download all" bulk queuing.
- Media lightbox: prev/next, drag-to-swipe, click-to-zoom, muted
  autoplay video, save-to-disk / queue-to-downloads.
- Download manager: 4-way concurrent worker pool, collision-safe
  filenames, session-only job list.
- Bookmarks with "new replies since last seen" tracking.
- Settings: theme (dark/light/system), accent color, animated
  background, catalog view mode, NSFW toggles, mute-by-default,
  download folder, custom site management.
- First-run onboarding wizard (5 steps, skippable, replayable).
- In-app LynxChan posting (captcha fetch + multipart reply) for
  8kun/8chan.moe-style sites.
- Custom frameless window chrome with platform-correct control
  placement (macOS traffic lights left, Windows/Linux buttons right).

## Known parity gaps

This is a first pass, not a finished port. Things intentionally left
for later:

- **External-site reply flow has no auto-refresh.** The Tauri app
  opens an *embedded* secondary window for CAPTCHA-gated sites (4chan,
  Lainchan) and auto-refreshes the thread when the user closes it.
  Doing that here needs `QtWebEngine` embedded in a popup, which this
  placeholder doesn't pull in yet — `ExternalReply` currently opens the
  user's system browser instead (comment still copied to clipboard),
  and the user has to hit refresh manually. See `ExternalReply.h`.
- **`CommentFormatter` is a regex-based approximation of DOMPurify**,
  not a real HTML parser. It's tuned against the shapes 4chan/vichan
  and LynxChan actually emit, but it isn't a hardened sanitizer.
  Revisit with a real parser before this replaces the Tauri app.
- **No animated-background pixel parity.** `aurora`/`particles`/`grid`
  are reimplemented with QML-native mechanisms (drifting blurred
  `Rectangle`s, `QtQuick.Particles`, a panning `Canvas` grid) rather
  than porting the exact CSS keyframes, so timing/easing differs from
  upstream.
- **No birthday hats, no favicon cascade for custom sites.** Both are
  pure "delight" features in the Tauri app (`src/lib/birthday.ts`,
  `src/lib/favicon.ts`) and haven't been ported yet.
- **Spoiler reveal-on-hover isn't replicated** in thread comment text
  (`<span class="spoiler">` becomes a static "black box" instead of a
  hover-to-reveal one) — a `Text.RichText` limitation vs. real DOM/CSS.
- **Theme color tokens are an approximation**, not extracted from the
  Tailwind config in `src/index.css` — visually close, not pixel-matched.
- Not yet build-tested in this environment (no Qt6/CMake toolchain was
  available where this was written) — see below before relying on it.

## Building

Requires **Qt 6.5+** (Core, Gui, Qml, Quick, QuickControls2, Network,
Multimedia) and **CMake 3.21+**.

```sh
cd Channex/QT
cmake -B build -DCMAKE_PREFIX_PATH="<path-to-your-Qt6-install>/lib/cmake"
cmake --build build
```

Then run the built `channex_qt` (`channex_qt.exe` on Windows) binary
from the build output directory.

No CI or packaging config yet (unlike the Tauri app's `tauri build`) —
add platform installers once the app is further along.

## AI usage disclaimer

Parts of this project (code, documentation, and/or assets) were written or assisted by AI tools. Review changes accordingly.
