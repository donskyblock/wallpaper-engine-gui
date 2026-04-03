use tauri::command;
use crate::steam::SteamCMD;
use crate::wallpaper::WallpaperManager;
use crate::kde::{self, KdeStatus};
use serde::Serialize;

#[derive(Serialize)]
pub struct WorkshopItem {
    pub id: u64,
    pub title: String,
    pub preview: String,
}

#[derive(Serialize)]
pub struct EnvironmentStatus {
    pub supported: bool,
    pub plugin_installed: bool,
    pub message: String,
}

#[command]
pub async fn check_environment() -> EnvironmentStatus {
    match kde::detect_environment() {
        KdeStatus::PluginInstalled => EnvironmentStatus {
            supported: true,
            plugin_installed: true,
            message: "Ready to go!".into(),
        },
        KdeStatus::KdeButNoPlugin => EnvironmentStatus {
            supported: true,
            plugin_installed: false,
            message: "KDE detected but plugin not installed. Click to install.".into(),
        },
        KdeStatus::NotKde => EnvironmentStatus {
            supported: false,
            plugin_installed: false,
            message: "Only KDE Plasma is currently supported.".into(),
        },
    }
}

#[command]
pub async fn install_plugin() -> Result<(), String> {
    kde::install_plugin()
}

#[command]
pub async fn fetch_workshop() -> Vec<WorkshopItem> {
    vec![
        WorkshopItem { id: 123, title: "Cool Wallpaper 1".into(), preview: "https://picsum.photos/200/150?random=1".into() },
        WorkshopItem { id: 456, title: "Cool Wallpaper 2".into(), preview: "https://picsum.photos/200/150?random=2".into() },
        WorkshopItem { id: 789, title: "Cool Wallpaper 3".into(), preview: "https://picsum.photos/200/150?random=3".into() },
    ]
}

#[command]
pub async fn install_wallpaper(wallpaper_id: u64) -> Result<(), String> {
    let steam = SteamCMD::new()?;
    let wm = WallpaperManager::new("downloads")?;

    if !wm.is_installed(wallpaper_id) {
        steam.download_workshop_item(431960, wallpaper_id)?;
    }

    wm.apply_wallpaper(wallpaper_id)?;
    Ok(())
}