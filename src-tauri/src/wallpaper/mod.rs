use std::path::{Path, PathBuf};
use std::fs;

pub struct WallpaperManager {
    base_path: PathBuf,
}

impl WallpaperManager {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Result<Self, String> {
        fs::create_dir_all(&base_path).map_err(|e| e.to_string())?;
        Ok(WallpaperManager { base_path: base_path.as_ref().to_path_buf() })
    }

    pub fn is_installed(&self, wallpaper_id: u64) -> bool {
        self.base_path.join(wallpaper_id.to_string()).exists()
    }

    pub fn apply_wallpaper(&self, wallpaper_id: u64) -> Result<(), String> {
        let path = self.base_path.join(wallpaper_id.to_string());
        if !path.exists() {
            return Err("Wallpaper not downloaded".into());
        }

        // Write a config file the KDE plugin can pick up
        let config_path = home_config_dir()?
            .join("wallpaper-engine-kde-plugin")
            .join("current_wallpaper.json");

        let json = serde_json::json!({
            "wallpaper_id": wallpaper_id,
            "path": path.to_string_lossy()
        });

        fs::create_dir_all(config_path.parent().unwrap()).map_err(|e| e.to_string())?;
        fs::write(config_path, serde_json::to_string_pretty(&json).unwrap())
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}

fn home_config_dir() -> Result<PathBuf, String> {
    let home = std::env::var("HOME").map_err(|_| "HOME not set".to_string())?;
    Ok(PathBuf::from(home).join(".config"))
}
