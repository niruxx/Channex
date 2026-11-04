# Channex GTK Roadmap

Channex is moving from Qt 6 / QML to **GTK 4 + libadwaita, written in Rust**.
The goal is an app that looks and behaves like a native GNOME app on Linux
and still runs on Windows.

- **Linux first, Windows last.** Every phase is built and tested on Linux.
  Windows support (Phase 10) is the last phase to be worked on.
- **Use real libadwaita widgets, not look-alikes.** Header bars, split views,
  preferences, status pages, toasts and dialogs come from libadwaita. The Qt
  app's custom frameless window chrome, hand-drawn controls and Windows
  drag workaround are all dropped. GTK draws its own window decorations on
  both platforms.
- **Qt is already gone from `main`.** The Qt app was removed at the start of
  the rewrite; its last version is commit `7cef950` (`git checkout 7cef950`),
  which is also the reference for porting. The
  [parity checklist](#parity-checklist) tracks what still has to come back.

Each phase ends with something that builds and runs. Tick boxes as work
lands.

---

## Phase 0 — Project setup ✅

- [x] Install the toolchain: `sudo pacman -S rust blueprint-compiler`.
- [x] Decide app ID (`io.github.niruxx.Channex`) and branch strategy
      (replace Qt on `main` right away).
- [x] Remove the Qt app (`src/`, `qml/`, `resources/`, `CMakeLists.txt`) and
      rewrite the README for the Rust project.
- [x] Cargo project with `gtk4` 0.11 and `libadwaita` 0.9. Minimum is
      **GTK 4.18 / libadwaita 1.8** (`AdwToggleGroup`, `AdwWrapBox`,
      `AdwShortcutsDialog`).
- [x] `build.rs` compiles Blueprint `.blp` files and bundles them, plus
      the icon and CSS, as a GResource.
- [x] `AdwApplication` + `AdwApplicationWindow` that opens and remembers
      its size, with a primary menu, About dialog, Keyboard Shortcuts
      dialog, and Ctrl+Q / Ctrl+W.
- [x] App icon, `.desktop` file and AppStream metainfo (both validate).
- [x] `cargo fmt` / `cargo clippy` / `cargo test` / metadata validation in
      GitHub Actions on an Arch container. Not run on GitHub yet; it runs
      on the first push.
- [x] Debug-only headless screenshots (`CHANNEX_SCREENSHOT`) for checking
      UI changes without a desktop, documented in the README.

## Phase 1 — Data layer (no UI) ✅

The C++ backend is now GTK-independent Rust in `src/data/` (the crate's
library half), checked by 43 unit tests against fixtures and by live tests
against the real sites (`cargo test --test live -- --ignored`).

- [x] **Models** (`model.rs`): `Site`, `Board`, `Post`, `PostFile`,
      `Thread` (with backlinks and all-files helpers). JSON names match the
      Qt app, so saved custom sites load as-is.
- [x] **HTTP client** (`http.rs`): `reqwest` + rustls, HTTP/1.1, 20 s
      inactivity timeout, one retry for dropped connections, and a clear
      error when a site answers with a web page instead of data.
- [x] **Yotsuba adapter** (4chan, vichan, Lainchan, 8kun) with all three
      media layouts, 8kun's per-file `fpath`, vichan multi-file posts, and
      4chan/8kun worksafe flags on boards.
- [x] **LynxChan adapter**, trying three board-index endpoints.
- [x] **Comment parser** (`comment.rs`): real HTML parser → a small tree
      (quotes, links, greentext, spoilers, dead quotes, emphasis, code),
      rendered to Pango markup with theme colors. Bare URLs become links.
      Every comment in live 4chan and Lainchan threads produces markup
      Pango accepts.
- [x] **Sites** (`sites.rs`): presets, board merging, validated custom
      sites, thread-link resolution (including non-ASCII boards like λ).
- [x] **Catalog search and sort** (`catalog.rs`).
- [x] **Settings + bookmarks store** (`store.rs`): same JSON layout as the
      Qt app, imports the Qt file on first run (checked against a copy of
      your real file: every key kept), atomic saves, unknown keys
      preserved, a corrupt file is set aside instead of overwritten.

**Fixed while porting** (bugs in the Qt version):

- "Latest Bump" sorted by thread *creation* time; it now keeps the
  site's bump order.
- Lainchan threads never loaded (wrong `/thread/` path, it's `/res/`),
  its thumbnails never loaded (they're `.png`, not `.jpg`), and its
  sidebar was empty (no board index; the preset now has Lainchan's real
  board list).
- Names and subjects only decoded five HTML entities (`It&#039;s`).
- 4chan spoilers (`<s>`) showed as strikethrough.

**Removed:** the 8chan.moe preset. Its API now sits behind a
proof-of-work check meant to keep non-browser clients out, so it can't
be supported. LynxChan support stays for custom sites.

**Moved to Phase 2:** starting the background runtime and opening the
store at app startup, since that's where the UI first needs them.

## Phase 2 — App shell and navigation ✅

- [x] `AdwOverlaySplitView`: F9 or the header toggle hides the sidebar,
      and below 720sp it becomes an overlay. The sidebar header has the
      site switcher (favicons, NSFW sites hidden when that setting is on,
      "Show N Hidden Sites") and the ☰ menu (Keyboard Shortcuts, About,
      Quit).
- [x] Sidebar body: board filter, "Library" (Bookmarks with a count
      badge) and the Boards list. Right-click or long-press a board →
      "Hide from Sidebar", with an Undo toast.
- [x] Content in an `AdwNavigationView`: opening a thread pushes a page,
      with back button, Alt+Left, mouse back button and swipe-back.
- [x] Shortcuts: Ctrl+F, F5/Ctrl+R, F9, Ctrl+W, Ctrl+Q, and the
      Keyboard Shortcuts dialog.
- [x] About dialog with version, links and the AI usage note.
- [x] Errors surface as toasts and status pages.
- [x] Also: reopens the last site and board, applies the saved
      light/dark setting, and announces the one-time import of the Qt
      version's settings.
- Moved: "Add Site…" and "Preferences" to Phase 5 (they open dialogs
  built there), the Downloads row to Phase 7, Ctrl+D to Phase 4.

## Phase 3 — Catalog ✅

- [x] `GtkGridView` of cards: thumbnail, subject, excerpt, counts,
      sticky/closed badges, bookmark button on hover (shown permanently
      once bookmarked).
- [x] Grid / Compact / List as an `AdwToggleGroup` in the header, saved
      to settings. List mode is a clamped column of card rows.
- [x] `GtkSearchBar` (Ctrl+F, or just start typing). Pasting a thread link
      and pressing Enter opens that thread.
- [x] Sort menu: Latest Bump, Most Replies, Most Images, Oldest, stickies
      first.
- [x] Thumbnail loader: at most 8 downloads at once, an in-memory cache of
      the 250 most recent textures (~60 MB max), and a disk cache in the
      user cache dir pruned of files older than 7 days. Decoding happens
      on the main thread (thumbnails are small); revisit if scrolling
      stutters.
- [x] NSFW/spoiler thumbnails blurred until clicked (CSS `filter: blur`),
      following the "Blur NSFW" setting.
- [x] Loading (`AdwSpinner`), error (`AdwStatusPage` + Try Again), and
      "No Results Found" states. Refreshing keeps the threads on screen
      and reports failures as a toast.
- [x] Opening a thread shows a temporary page with "Open in Browser"
      until Phase 4.

## Phase 4 — Thread view ✅

- [x] `GtkListView` of post cards in an `AdwClampScrollable`; the OP is
      accent-tinted with an "OP" badge.
- [x] Comments as Pango markup. Clicking `>>123` scrolls to that post and
      highlights it for a moment (a toast says so if it's not in the
      thread). "Replies:" backlinks under each post.
- [x] Quote hover preview in a popover. *(Built, but only rendering has
      been checked headlessly - hover needs a real desktop.)*
- [x] Files with thumbnail, type, size and dimensions (PDFs on Lainchan
      too). Clicking one opens it in the browser until the lightbox
      (Phase 6).
- [x] Header: subject as title, "site · /board/ · N posts · N files" as
      subtitle; Bookmark (Ctrl+D), Open in Browser, Refresh (F5).
- [x] Post styles Compact / Standard / Modern (Modern adds avatars).
- [x] Opening a bookmarked thread marks its replies as seen.
- [x] Right-click a post: Copy Text, Copy Link, and (4chan only) Report
      Post….
- [x] Also: whitespace in comments now collapses like a browser's (the
      4chan /g/ sticky was indented).

## Phase 5 — Preferences, bookmarks and onboarding ✅

- [x] `AdwPreferencesDialog` (Ctrl+, or the ☰ menu):
  - **Appearance:** style (System / Light / Dark), accent color (follow
    the system, a GNOME accent, or your own color imported from the Qt
    version), classic imageboard skin, catalog view, post style.
  - **Content:** blur NSFW, hide NSFW sites.
  - **Sites page:** custom sites (add / remove) and hidden boards per
    site (with a Show button to bring each back).
  - "Show Welcome Tour".
- [x] Classic skins (Yotsuba, Yotsuba B, Futaba, Photon, Burichan) as
      overrides of libadwaita's color variables; they force light mode.
- [x] Add Site dialog (☰ menu, site switcher, or Preferences): name,
      address, engine, media layout, thumbnail format, NSFW. Validates
      the address and switches to the new site.
- [x] Bookmarks page (done in Phase 2-3).
- [x] Welcome tour: a 4-page `AdwCarousel` (welcome, appearance,
      content, shortcuts), shown on first run, skippable, replayable.
- Moved: "Mute videos by default" and "Birthday hats" to
  [Future features](#future-features), and the download folder / folder
  naming settings to Phase 7, so each arrives with the feature it
  controls.

## Phase 6 — Media viewer (moved to Future features)

Deferred: see [Future features](#future-features). Until then, clicking a
file in a thread opens it in the browser.

## Phase 7 — Downloads ✅

- [x] Download manager: at most 4 files at once across all jobs, each
      streamed to `<name>.part` and renamed when complete; names
      sanitized for Linux and Windows and made unique (`image (1).png`)
      against the folder and the rest of the batch.
- [x] Downloads page (sidebar → Library): one card per job with status,
      "N of M files", a progress bar, Retry Failed, Open Folder, Remove,
      and the file list on demand; Clear History in the header (files
      are never deleted). Session-only, like the Qt version.
- [x] "Download Files" dialog from the thread header (or a post's
      right-click menu for just its files): thumbnails, all selected by
      default, Select All / Select None, and the destination shown.
- [x] A toast when a job finishes, with Open Folder. The sidebar's
      Downloads row shows how many jobs are running.
- [x] Folders open with `GtkFileLauncher` (works on both platforms).
- [x] Preferences → Downloads: download folder (native folder picker,
      or back to the default) and thread folder names (site-board-number
      or the thread's subject).
- [x] Default folder is now `~/Downloads/Channex`. The Qt version used
      the current directory when none was set, which is how a download
      folder ended up inside the repository.
- [x] Checked end to end against a real Lainchan thread: six files,
      including a 19 MB PDF, saved intact.

## Phase 8 — Posting ✅

- [x] Reply dialog from the thread header, or **Reply** on a post's
      right-click menu (starts the comment with `>>id`). Unsent text is
      kept as a draft for that thread.
- [x] **LynxChan sites, in the app:** name, options, comment, the site's
      captcha (with a refresh button) and its answer. Errors such as
      "Wrong captcha." show in the dialog with a fresh captcha; on success
      the thread reloads and scrolls to the new post. The HTTP client now
      keeps cookies, which LynxChan uses to match a reply to its captcha.
      Checked against Endchan up to the captcha; no post was submitted
      to a real site.
- [x] **Captcha-gated sites (4chan, Lainchan, 8kun):** write the reply,
      then **Continue on Site** copies it to the clipboard and opens the
      thread in the browser. 4chan also gets **Sign In with 4chan Pass**.
- [x] Fixed: the 8kun preset was set to post the LynxChan way, which 8kun
      doesn't support; it now uses the browser hand-off.
- [x] Also fixed while testing against Endchan (a working LynxChan site):
      its board list shape wasn't recognised, and LynxChan catalogs showed
      one reply too few (the Qt version subtracted the OP from a count
      that never included it). A live LynxChan test now covers boards,
      catalog, thread, thumbnails and captcha.
- Skipped: the optional embedded WebKitGTK captcha window (Linux-only).

## Phase 9 — Linux packaging ✅

- [x] `packaging/arch/PKGBUILD` rewritten for the Rust build, following
      Arch's Rust guidelines (`cargo fetch --locked`, `--frozen` build,
      tests in `check()`, `!lto` - see below). Built here with makepkg:
      a 3 MB package (8.2 MB installed) depending only on the GTK and
      libadwaita stack, and its binary starts cleanly.
- [x] Installs `/usr/bin/channex`, the desktop entry, hicolor icon and
      AppStream metainfo under the app ID (both validate).
- [x] Flatpak manifest (`packaging/flatpak/`) on the GNOME 50 runtime with
      the Rust SDK extension; blueprint-compiler built in as a module.
      Rust dependencies are vendored through `cargo-sources.json`,
      generated from `Cargo.lock` by flatpak-cargo-generator. *(Not
      test-built here: flatpak-builder isn't installed. CI builds it.)*
- [x] `release.yml`: on a `v*` tag, builds the Arch package and a Flatpak
      bundle and attaches both to the GitHub release (or uploads them as
      artifacts when run by hand). *(Runs on the first tag push.)*
- Note: makepkg's default LTO breaks linking the bundled aws-lc crypto
  library, hence `!lto`. aws-lc also needs CMake to build; see open
  question 2.
- Still needed for Flathub itself: a license in the repo (for
  `<project_license>`), screenshots in the metainfo, and a submission PR
  to flathub/flathub.

## Phase 11 — Release ✅ (except the tag)

The Qt code itself was removed in Phase 0.

- [x] Removed the Qt-era leftovers: `package-history/` (21 MB of old Qt
      packages) and the Qt desktop file. (`build-appimage/` is a local,
      ignored build folder; delete it by hand if you like.)
- [x] Final README pass: what Channex does now, installing, building,
      known limits, architecture, and new screenshots (taken on
      Lainchan's programming board, checked to be work-safe, no personal
      paths).
- [x] AppStream metainfo: description, four screenshots, 0.2.0 release
      notes, and the correct GitHub links (they pointed at the wrong
      account).
- [ ] Commit, then tag `v0.2.0` (the release workflow builds and attaches
      the packages).

## Phase 10 — Windows (last)

The last phase to be worked on: it comes after every other phase.

- [ ] Build with MSYS2 UCRT64 (its `gtk4`, `libadwaita`, `gstreamer`
      packages) and Rust's `x86_64-pc-windows-gnu` target.
- [ ] Bundle the runtime next to the `.exe`: GTK/libadwaita DLLs,
      gdk-pixbuf loaders, GStreamer plugins (VP8/VP9/libav), compiled
      GSettings schemas and the Adwaita icon theme.
- [ ] Installer (NSIS or MSI), Start Menu shortcut and uninstaller,
      replacing the CMake/CPack setup.
- [ ] Check on Windows: dark/light following the system, HiDPI scaling,
      fonts, video playback, file dialogs, opening folders.
- [ ] Windows build job in GitHub Actions producing the installer.

## Future features

Planned, but after the phases above.

### Media viewer (was Phase 6)

- [ ] Full-window media viewer over the app, using `AdwCarousel` for
      prev/next and swipe.
- [ ] Images in `GtkPicture`: click or pinch to zoom, drag to pan, plus
      animated GIF support.
- [ ] Video (webm/mp4) through `GtkVideo` / `GtkMediaFile` (GStreamer
      backend): play/pause, seek, mute, honouring "mute by default".
- [ ] Filmstrip of every file in the thread.
- [ ] Save to Disk and Add to Downloads buttons, Esc to close, arrow keys
      to move between files.
- [ ] "Couldn't load" state with Retry and Open in Browser.
- [ ] "Mute videos by default" setting (Preferences → Content), applied
      when a video starts.

### Birthday hats

- [ ] Party hats on posts on October 1st (4chan's anniversary), as the
      site does it - and the "Birthday hats" switch in Preferences →
      Content to turn them off. The Qt version had the setting but never
      drew the hats; the saved preference is already carried over.

---

## Parity checklist

Everything the Qt app does today (from the README and the source),
with the phase that brings it over.

| Feature | Phase |
|---|---|
| Site presets (4chan, 8kun, Lainchan) + custom sites, saved to disk | 1, 5 |
| Board discovery with default-board fallback | 1 |
| Hide boards per site | 2, 5 |
| Catalog grid / compact / list, search, 4 sort modes, stickies first | 3 |
| NSFW / spoiler blur until clicked | 3, 4 |
| Paste a thread link to jump to it | 3 |
| Thread view with quote links and backlinks | 4 |
| Bookmarks with "new replies since last seen" | 4, 5 |
| Light / dark / system theme, accent color | 5 |
| Classic imageboard skins (5) | 5 |
| Reply display styles (3) | 4 |
| Onboarding wizard (5 steps, skippable, replayable) | 5 |
| Lightbox: prev/next, swipe, zoom, filmstrip, video controls | Future |
| Save to disk / queue from lightbox | Future |
| Download picker dialog + 4-way concurrent download manager | 7 |
| Thread folder naming (ID or name) | 7 |
| Mute videos by default | Future |
| Birthday hats | Future |
| LynxChan in-app posting with captcha | 8 |
| External reply handoff + 4chan Pass sign-in | 8 |
| Report post (4chan) | 4 |

**Dropped on purpose:**

- Custom frameless window chrome and the Windows manual-drag workaround.
  GTK's own header bars handle this.
- Animated backgrounds (aurora / particles / grid). They don't fit the
  GNOME style; this can be revisited (see open questions).

---

## Technical decisions

| Area | Choice | Why |
|---|---|---|
| Language | Rust (gtk4-rs, libadwaita-rs) | GNOME's main choice for new apps; compile-time thread safety for the downloader |
| UI definitions | Blueprint (`.blp`) + composite templates | Readable, the standard in modern GNOME Rust apps |
| Async / HTTP | `tokio` + `reqwest` (rustls), results sent to the GLib main loop | Cross-platform, no OpenSSL to ship on Windows |
| JSON | `serde` / `serde_json` | Same field names as the Qt app, so data migrates |
| Comment HTML | `html5ever` / `scraper` → Pango markup | A real parser instead of regexes |
| Media | GTK's GStreamer media backend | Plays webm/mp4; bundled on Windows |
| Settings storage | JSON file, not GSettings | No schema to install; identical on Windows |

## Decided

- **App ID:** `io.github.niruxx.Channex`.
- **Branch strategy:** Qt removed from `main` immediately; the GTK app is
  built directly on `main`.
- **8chan.moe:** removed (its API blocks non-browser clients).
- **Windows:** Phase 10 is the last phase to be worked on.
- **Accent color:** follows the system accent by default, with an
  app-level override in Preferences.

## Open questions

1. **Animated backgrounds:** drop them (planned), or bring back a subtle
   one later?
2. **Windows Rust target:** GNU via MSYS2 (simplest with GTK, planned) or
   MSVC with gvsbuild? Related: reqwest's TLS uses aws-lc, which needs
   CMake (and NASM on Windows) and broke under makepkg's LTO. Switching
   rustls to its `ring` backend would remove all three issues.
3. **License:** the repo has none, which Flathub (and anyone wanting to
   contribute) needs. Which license should Channex use?
