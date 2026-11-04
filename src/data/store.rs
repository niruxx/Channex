//! Settings and bookmarks, saved together in one JSON file.
//!
//! The layout is the Qt version's `channex-qt-data.json` (settings as
//! top-level keys plus a `bookmarks` array), and on first run that file
//! is imported, so upgrading keeps everything. Keys this version doesn't
//! use are kept and written back untouched.

use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::model::Site;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CatalogViewMode {
    Compact,
    List,
    #[default]
    #[serde(other)]
    Grid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReplyStyle {
    Compact,
    Modern,
    #[default]
    #[serde(other)]
    Standard,
}

/// How per-thread download folders are named.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FolderNaming {
    /// The thread's subject (or the start of its first post).
    Name,
    /// `site-board-threadId`
    #[default]
    #[serde(other)]
    Id,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub download_dir: String,
    pub blur_nsfw: bool,
    pub hide_nsfw_sites: bool,
    /// `system`, `light`, `dark`, or a classic skin id (`yotsuba`, …).
    pub theme: String,
    /// `None` follows the system accent color.
    pub accent_color: Option<String>,
    pub catalog_view_mode: CatalogViewMode,
    pub reply_display_style: ReplyStyle,
    pub download_folder_naming: FolderNaming,
    pub mute_webms_by_default: bool,
    pub birthday_hats: bool,
    pub has_completed_onboarding: bool,
    pub custom_sites: Vec<Site>,
    /// Site id → board codes hidden from the sidebar.
    pub hidden_boards: HashMap<String, Vec<String>>,
    /// How often the image cache is cleared.
    pub cache_clear_interval: super::cache::ClearInterval,
    /// Image cache size limit in MB (0 = no limit).
    pub cache_size_limit_mb: u32,
    /// When the image cache was last cleared (Unix seconds; 0 = never).
    pub cache_last_cleared: i64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            download_dir: String::new(),
            blur_nsfw: true,
            hide_nsfw_sites: false,
            theme: "system".into(),
            accent_color: None,
            catalog_view_mode: CatalogViewMode::default(),
            reply_display_style: ReplyStyle::default(),
            download_folder_naming: FolderNaming::default(),
            mute_webms_by_default: true,
            birthday_hats: true,
            has_completed_onboarding: false,
            custom_sites: Vec::new(),
            hidden_boards: HashMap::new(),
            cache_clear_interval: super::cache::ClearInterval::default(),
            cache_size_limit_mb: 500,
            cache_last_cleared: 0,
        }
    }
}

impl Settings {
    pub fn is_board_hidden(&self, site_id: &str, board: &str) -> bool {
        self.hidden_boards
            .get(site_id)
            .is_some_and(|b| b.iter().any(|c| c == board))
    }

    pub fn set_board_hidden(&mut self, site_id: &str, board: &str, hidden: bool) {
        let list = self.hidden_boards.entry(site_id.to_owned()).or_default();
        let present = list.iter().position(|c| c == board);
        match (hidden, present) {
            (true, None) => list.push(board.to_owned()),
            (false, Some(i)) => {
                list.remove(i);
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Bookmark {
    pub site_id: String,
    pub board_code: String,
    pub thread_id: String,
    pub subject: String,
    pub thumb_url: String,
    pub reply_count: u32,
    pub last_seen_reply_count: u32,
    /// Unix ms.
    pub added_at: i64,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Bookmark {
    pub fn new_replies(&self) -> u32 {
        self.reply_count.saturating_sub(self.last_seen_reply_count)
    }

    fn is(&self, site_id: &str, board: &str, thread_id: &str) -> bool {
        self.site_id == site_id && self.board_code == board && self.thread_id == thread_id
    }
}

/// Everything in the data file.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AppData {
    #[serde(flatten)]
    pub settings: Settings,
    pub bookmarks: Vec<Bookmark>,
    /// Keys from older (or newer) versions, preserved as-is.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl AppData {
    pub fn bookmark(&self, site_id: &str, board: &str, thread_id: &str) -> Option<&Bookmark> {
        self.bookmarks
            .iter()
            .find(|b| b.is(site_id, board, thread_id))
    }

    pub fn is_bookmarked(&self, site_id: &str, board: &str, thread_id: &str) -> bool {
        self.bookmark(site_id, board, thread_id).is_some()
    }

    /// Adds the bookmark, or removes it if it's already there. Returns
    /// whether the thread is bookmarked afterwards. New bookmarks start
    /// with everything "seen".
    pub fn toggle_bookmark(&mut self, mut bookmark: Bookmark, now_ms: i64) -> bool {
        let (site, board, thread) = (
            bookmark.site_id.clone(),
            bookmark.board_code.clone(),
            bookmark.thread_id.clone(),
        );
        if self.remove_bookmark(&site, &board, &thread) {
            return false;
        }
        bookmark.added_at = now_ms;
        bookmark.last_seen_reply_count = bookmark.reply_count;
        self.bookmarks.push(bookmark);
        true
    }

    /// Returns whether anything was removed.
    pub fn remove_bookmark(&mut self, site_id: &str, board: &str, thread_id: &str) -> bool {
        let before = self.bookmarks.len();
        self.bookmarks.retain(|b| !b.is(site_id, board, thread_id));
        self.bookmarks.len() != before
    }

    /// Records the thread's current reply count as seen (opening the
    /// thread). Returns whether anything changed.
    pub fn mark_seen(
        &mut self,
        site_id: &str,
        board: &str,
        thread_id: &str,
        reply_count: u32,
    ) -> bool {
        match self
            .bookmarks
            .iter_mut()
            .find(|b| b.is(site_id, board, thread_id))
        {
            Some(b) if b.last_seen_reply_count != reply_count || b.reply_count != reply_count => {
                b.reply_count = reply_count;
                b.last_seen_reply_count = reply_count;
                true
            }
            _ => false,
        }
    }
}

/// What happened when the store was opened, for a one-time notice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadOutcome {
    /// Normal load (or a fresh start with no file yet).
    Loaded,
    /// No data file yet; settings and bookmarks were imported from the
    /// Qt version's file.
    ImportedFromQt,
    /// The data file couldn't be read as JSON. It was moved aside to
    /// this path and defaults are in use.
    Corrupt { backup: PathBuf },
}

#[derive(Debug)]
pub struct Store {
    path: PathBuf,
    data: AppData,
}

impl Store {
    /// Opens `path`, importing `legacy` (the Qt data file) if `path`
    /// doesn't exist yet.
    pub fn open(
        path: impl Into<PathBuf>,
        legacy: Option<&Path>,
    ) -> io::Result<(Self, LoadOutcome)> {
        let path = path.into();
        if path.exists() {
            let text = std::fs::read_to_string(&path)?;
            return match serde_json::from_str::<AppData>(&text) {
                Ok(data) => Ok((Self { path, data }, LoadOutcome::Loaded)),
                Err(_) => {
                    let backup = path.with_extension(format!("broken-{}.json", std::process::id()));
                    std::fs::rename(&path, &backup)?;
                    Ok((
                        Self {
                            path,
                            data: AppData::default(),
                        },
                        LoadOutcome::Corrupt { backup },
                    ))
                }
            };
        }

        if let Some(legacy) = legacy.filter(|p| p.exists()) {
            if let Ok(data) = std::fs::read_to_string(legacy)
                .map_err(|_| ())
                .and_then(|t| serde_json::from_str::<AppData>(&t).map_err(|_| ()))
            {
                let store = Self { path, data };
                store.save()?;
                return Ok((store, LoadOutcome::ImportedFromQt));
            }
        }
        Ok((
            Self {
                path,
                data: AppData::default(),
            },
            LoadOutcome::Loaded,
        ))
    }

    pub fn data(&self) -> &AppData {
        &self.data
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Changes the data and saves it.
    pub fn update<R>(&mut self, f: impl FnOnce(&mut AppData) -> R) -> io::Result<R> {
        let result = f(&mut self.data);
        self.save()?;
        Ok(result)
    }

    /// Writes to a temporary file, then renames it over the real one,
    /// so a crash mid-write can't leave a truncated file.
    pub fn save(&self) -> io::Result<()> {
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let tmp = self.path.with_extension("json.tmp");
        std::fs::write(&tmp, serde_json::to_string_pretty(&self.data)?)?;
        std::fs::rename(tmp, &self.path)
    }
}

/// Where the Qt version kept its data file (`QStandardPaths::AppDataLocation`
/// for organization "Channex", application "Channex Qt").
pub fn legacy_qt_data_file() -> Option<PathBuf> {
    let base = if cfg!(windows) {
        std::env::var_os("APPDATA").map(PathBuf::from)?
    } else if let Some(xdg) = std::env::var_os("XDG_DATA_HOME").filter(|v| !v.is_empty()) {
        PathBuf::from(xdg)
    } else {
        PathBuf::from(std::env::var_os("HOME")?).join(".local/share")
    };
    Some(
        base.join("Channex")
            .join("Channex Qt")
            .join("channex-qt-data.json"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const QT_FILE: &str = r##"{
        "accentColor": "#6ee7c9",
        "backgroundTheme": "aurora",
        "birthdayHats": true,
        "blurNsfw": false,
        "catalogViewMode": "list",
        "customSites": [{"id": "custom-1", "name": "Mine", "schema": "lynxchan", "mediaLayout": "board-dirs",
                         "apiOrigin": "https://x.org", "siteOrigin": "https://x.org", "mediaOrigin": "https://x.org",
                         "accent": "#6ee7c9", "isCustom": true, "nsfw": true, "postEngine": "lynxchan",
                         "defaultBoards": [], "favicon": "https://x.org/favicon.ico"}],
        "downloadDir": "/home/me/Pictures",
        "downloadFolderNaming": "name",
        "hasCompletedOnboarding": true,
        "hiddenBoards": {"4chan": ["b", "pol"]},
        "hideNsfwSites": true,
        "muteWebmsByDefault": false,
        "replyDisplayStyle": "modern",
        "theme": "dark",
        "bookmarks": [{"siteId": "4chan", "boardCode": "g", "threadId": "1", "subject": "Desktop",
                       "replyCount": 40, "lastSeenReplyCount": 30, "addedAt": 5, "thumbUrl": ""}]
    }"##;

    #[test]
    fn reads_qt_file_and_keeps_unknown_keys() {
        let data: AppData = serde_json::from_str(QT_FILE).unwrap();
        let s = &data.settings;
        assert_eq!(s.theme, "dark");
        assert_eq!(s.accent_color.as_deref(), Some("#6ee7c9"));
        assert!(!s.blur_nsfw && s.hide_nsfw_sites && !s.mute_webms_by_default);
        assert_eq!(s.catalog_view_mode, CatalogViewMode::List);
        assert_eq!(s.reply_display_style, ReplyStyle::Modern);
        assert_eq!(s.download_folder_naming, FolderNaming::Name);
        assert_eq!(s.custom_sites[0].name, "Mine");
        assert!(s.is_board_hidden("4chan", "pol"));
        assert_eq!(data.bookmarks[0].new_replies(), 10);
        assert_eq!(data.extra["backgroundTheme"], "aurora");

        let round_trip: Value = serde_json::to_value(&data).unwrap();
        assert_eq!(round_trip["backgroundTheme"], "aurora");
        assert_eq!(round_trip["catalogViewMode"], "list");
        assert_eq!(round_trip["bookmarks"][0]["lastSeenReplyCount"], 30);
    }

    #[test]
    fn defaults_for_a_new_user() {
        let s = Settings::default();
        assert_eq!(s.theme, "system");
        assert!(s.accent_color.is_none() && s.blur_nsfw && !s.has_completed_onboarding);
    }

    #[test]
    fn bookmark_operations() {
        let mut data = AppData::default();
        let b = Bookmark {
            site_id: "4chan".into(),
            board_code: "g".into(),
            thread_id: "1".into(),
            reply_count: 7,
            ..Default::default()
        };
        assert!(data.toggle_bookmark(b.clone(), 99));
        assert_eq!(data.bookmarks[0].last_seen_reply_count, 7);
        assert_eq!(data.bookmarks[0].added_at, 99);
        assert!(data.is_bookmarked("4chan", "g", "1"));
        assert!(data.mark_seen("4chan", "g", "1", 12));
        assert!(
            !data.mark_seen("4chan", "g", "1", 12),
            "no change the second time"
        );
        assert!(!data.toggle_bookmark(b, 100), "toggling again removes it");
        assert!(data.bookmarks.is_empty());
    }

    #[test]
    fn hidden_boards() {
        let mut s = Settings::default();
        s.set_board_hidden("4chan", "b", true);
        s.set_board_hidden("4chan", "b", true);
        assert_eq!(s.hidden_boards["4chan"], ["b"]);
        s.set_board_hidden("4chan", "b", false);
        assert!(!s.is_board_hidden("4chan", "b"));
    }

    #[test]
    fn imports_qt_file_once_then_uses_own_file() {
        let dir = tempfile::tempdir().unwrap();
        let legacy = dir.path().join("qt.json");
        std::fs::write(&legacy, QT_FILE).unwrap();
        let path = dir.path().join("channex/channex.json");

        let (store, outcome) = Store::open(&path, Some(&legacy)).unwrap();
        assert_eq!(outcome, LoadOutcome::ImportedFromQt);
        assert_eq!(store.data().settings.theme, "dark");
        assert!(path.exists(), "imported data is written to the new file");
        assert!(legacy.exists(), "the Qt file is left alone");

        let (mut store, outcome) = Store::open(&path, Some(&legacy)).unwrap();
        assert_eq!(outcome, LoadOutcome::Loaded);
        store.update(|d| d.settings.theme = "light".into()).unwrap();
        let (store, _) = Store::open(&path, Some(&legacy)).unwrap();
        assert_eq!(store.data().settings.theme, "light");
        assert_eq!(
            store.data().extra["backgroundTheme"],
            "aurora",
            "unknown keys survive saves"
        );
    }

    #[test]
    fn corrupt_file_is_moved_aside() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("channex.json");
        std::fs::write(&path, "{ not json").unwrap();
        let (store, outcome) = Store::open(&path, None).unwrap();
        let LoadOutcome::Corrupt { backup } = outcome else {
            panic!("expected Corrupt")
        };
        assert_eq!(std::fs::read_to_string(backup).unwrap(), "{ not json");
        assert_eq!(store.data().settings, Settings::default());
    }

    #[test]
    fn fresh_start_without_any_file() {
        let dir = tempfile::tempdir().unwrap();
        let (store, outcome) = Store::open(
            dir.path().join("x.json"),
            Some(&dir.path().join("missing.json")),
        )
        .unwrap();
        assert_eq!(outcome, LoadOutcome::Loaded);
        assert_eq!(store.data(), &AppData::default());
    }
}
