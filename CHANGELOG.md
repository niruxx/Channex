# Changelog

All notable changes to Channex are listed here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions
follow [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- **Linux packages:** build scripts for Debian/Ubuntu (`.deb`), Fedora
  (`.rpm`) and an AppImage, alongside the existing Arch package and
  Flatpak. The release workflow now builds all of them.
- **Birthday hats:** party hats on post images and thread thumbnails every
  October 1st (4chan's birthday). They can be turned off in Preferences →
  Content.
- **Storage settings** (Preferences → Storage):
  - see how much space the image cache uses, and clear it;
  - clear it automatically every day, week or month;
  - keep it under a size limit (100 MB to 2 GB), removing the oldest
    images first.
- **Video status messages:** when a site holds a video up, a note over
  the video says why, for example "The site is limiting how fast files
  load. Trying again in 3 s…". A note also appears after 15 seconds of
  loading.
- **Clear reasons when a file can't be shown:**
  - rate limited (HTTP 429);
  - file deleted (404);
  - another error from the site;
  - connection lost;
  - missing GStreamer plugins, with the packages to install.

### Changed

- **Videos stream instead of downloading.** Channex fetches only the parts
  of a video it needs, over HTTP range requests through its own
  connection. Nothing is written to disk, and seeking fetches just that
  part.
- **Fewer requests to rate-limited sites:**
  - video data already received (up to 8 MB per video) is reused from
    memory;
  - retries wait as long as the site asks (`Retry-After`, up to 10 s) and
    stop after two attempts.

### Fixed

- **Videos and GIFs not opening at all** on most desktops. Channex was
  handing GTK an `https://` address, which only works when a gvfs HTTP
  backend is running.
- **Hard freeze when switching or closing a video.** A network wait during
  video start-up could block the whole window, without end under rate
  limiting.
- **Videos stuck on "loading" forever when rate-limited.** After the
  retries ran out, the video player kept requesting other parts of the
  file. Now the video stops, the reason is shown, and no more requests
  are made.
- **GIFs freezing on their last frame instead of looping.** GIFs are now
  decoded in memory and loop forever.

## [0.2.0] - 2026-11-04

A full rewrite of Channex as a native **GTK 4 + libadwaita** app in Rust,
replacing the Qt 6 / QML version. The last Qt version is commit `7cef950`.

### Added

- **Sites:** 4chan, 8kun and Lainchan built in. Any custom site with a
  4chan-style API or running LynxChan can be added, with live board lists
  and hideable boards.
- **Catalog:** grid, compact grid and list views, with search and sorting.
  Paste a thread link to jump straight to it. NSFW and spoiler thumbnails
  are blurred until clicked.
- **Threads:**
  - quote links, hover previews and reply lists;
  - three post styles;
  - a right-click menu to reply, copy, download, or report (on 4chan).
- **Media viewer:**
  - images, GIFs and videos full-window;
  - swipe or the arrow keys to move between files;
  - click or pinch to zoom;
  - a filmstrip of every file in the thread.
- **Bookmarks** that count new replies.
- **Downloads:** choose which of a thread's files to save. Four download
  at a time into a folder per thread, with progress and retries.
- **Replying:** in the app on LynxChan sites. On 4chan, Lainchan and 8kun,
  write your reply in Channex and finish on the site.
- **Appearance:**
  - follows the system's light/dark style and accent color, or set your
    own;
  - five classic imageboard skins.
- **Keyboard shortcuts** and a welcome tour.
- Settings and bookmarks from the Qt version are imported automatically.

### Removed

- **8chan.moe:** its API now blocks apps that aren't web browsers.
- The Qt / QML code and its dependencies.

## [0.1.2] - Qt version

The last release of the Qt 6 / QML version.

[Unreleased]: https://github.com/niruxx/Channex/compare/0.2.0...HEAD
[0.2.0]: https://github.com/niruxx/Channex/compare/0.1.2-1...0.2.0
[0.1.2]: https://github.com/niruxx/Channex/releases/tag/0.1.2-1
