//! Remembers the main window's size and the last board opened between
//! runs, in
//! `<user data dir>/channex/window-state.json`. (GSettings isn't used:
//! it needs an installed schema, which complicates Windows builds.)

use std::path::PathBuf;

use gtk::glib;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WindowState {
    pub width: i32,
    pub height: i32,
    pub maximized: bool,
    pub last_site: String,
    pub last_board: String,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            width: 1200,
            height: 800,
            maximized: false,
            last_site: "4chan".into(),
            last_board: "g".into(),
        }
    }
}

impl WindowState {
    fn path() -> PathBuf {
        glib::user_data_dir()
            .join("channex")
            .join("window-state.json")
    }

    /// Falls back to defaults if the file is missing or unreadable.
    pub fn load() -> Self {
        std::fs::read_to_string(Self::path())
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::path();
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(path, serde_json::to_string_pretty(self)?)
    }
}
