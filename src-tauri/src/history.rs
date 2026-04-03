use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Serialize, Deserialize)]
pub struct HistoryItem {
    pub wallpaper_id: u64,
    pub title: Option<String>,
    pub preview: Option<String>,
    pub creator: Option<String>,
    pub applied_at: String,
    pub local_path: String,
}

pub fn load_history() -> Result<Vec<HistoryItem>, String> {
    let path = history_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }

    let raw = fs::read_to_string(&path).map_err(|e| format!("Failed to read history: {e}"))?;
    serde_json::from_str(&raw).map_err(|e| format!("Failed to parse history: {e}"))
}

pub fn save_history(items: &[HistoryItem]) -> Result<(), String> {
    let path = history_path()?;
    let Some(parent) = path.parent() else {
        return Err("Failed to determine history directory.".into());
    };

    fs::create_dir_all(parent).map_err(|e| format!("Failed to create history directory: {e}"))?;
    let serialized = serde_json::to_string_pretty(items)
        .map_err(|e| format!("Failed to serialize history: {e}"))?;
    fs::write(path, serialized).map_err(|e| format!("Failed to write history: {e}"))
}

pub fn record(item: HistoryItem) -> Result<(), String> {
    let mut history = load_history()?;
    history.retain(|entry| entry.wallpaper_id != item.wallpaper_id);
    history.insert(0, item);
    if history.len() > 25 {
        history.truncate(25);
    }
    save_history(&history)
}

fn history_path() -> Result<PathBuf, String> {
    let dirs = ProjectDirs::from("online", "opencom", "wallpaper-engine-linux")
        .ok_or_else(|| "Failed to resolve the app data directory.".to_string())?;
    Ok(dirs.data_local_dir().join("history.json"))
}
