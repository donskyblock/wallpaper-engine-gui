use std::fs;
use std::path::{Path, PathBuf};

pub struct WallpaperManager;

impl WallpaperManager {
    pub fn new() -> Self {
        Self
    }

    pub fn is_installed<P: AsRef<Path>>(&self, wallpaper_path: P) -> bool {
        wallpaper_path.as_ref().exists()
    }

    pub fn apply_wallpaper<P: AsRef<Path>>(
        &self,
        wallpaper_id: u64,
        wallpaper_path: P,
    ) -> Result<(), String> {
        let wallpaper_path = wallpaper_path.as_ref();
        if !wallpaper_path.exists() {
            return Err("Wallpaper not downloaded".into());
        }

        let config_path = home_config_dir()?
            .join("wallpaper-engine-kde-plugin")
            .join("current_wallpaper.json");

        let json = serde_json::json!({
            "wallpaper_id": wallpaper_id,
            "path": wallpaper_path.to_string_lossy()
        });

        let Some(parent) = config_path.parent() else {
            return Err("Failed to determine the wallpaper config directory.".into());
        };

        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        fs::write(
            config_path,
            serde_json::to_string_pretty(&json).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }
}

fn home_config_dir() -> Result<PathBuf, String> {
    let home = std::env::var("HOME").map_err(|_| "HOME not set".to_string())?;
    Ok(PathBuf::from(home).join(".config"))
}
