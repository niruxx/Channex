use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    #[serde(default = "default_preferred_site")]
    pub preferred_site: String,
    #[serde(default)]
    pub custom_origin: Option<String>,
    #[serde(default)]
    pub download_root: Option<String>,
    #[serde(default = "default_concurrency")]
    pub concurrency: usize,
}

fn default_preferred_site() -> String {
    "4chan".to_string()
}

fn default_concurrency() -> usize {
    6
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            preferred_site: default_preferred_site(),
            custom_origin: None,
            download_root: None,
            concurrency: default_concurrency(),
        }
    }
}

fn settings_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("Couldn't resolve config directory: {e}"))?;
    Ok(dir.join("settings.json"))
}

pub fn load(app: &AppHandle) -> AppSettings {
    let Ok(path) = settings_path(app) else {
        return AppSettings::default();
    };
    let Ok(contents) = std::fs::read_to_string(&path) else {
        return AppSettings::default();
    };
    serde_json::from_str(&contents).unwrap_or_default()
}

pub fn save(app: &AppHandle, settings: &AppSettings) -> Result<(), String> {
    let path = settings_path(app)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Couldn't create config dir: {e}"))?;
    }
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| format!("Couldn't write settings: {e}"))
}
