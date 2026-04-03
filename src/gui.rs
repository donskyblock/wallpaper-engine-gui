use tauri::command;
use crate::steam::SteamCMD;
use crate::wallpaper::WallpaperManager;
use serde::Serialize;

#[derive(Serialize)]
pub struct WorkshopItem {
    pub id: u64,
    pub title: String,
    pub preview: String,
}

#[command]
pub async fn fetch_workshop() -> Vec<WorkshopItem> {
    // For MVP: static examples, later fetch real workshop metadata
    vec![
        WorkshopItem { id: 123, title: "Cool Wallpaper 1".into(), preview: "https://via.placeholder.com/200".into() },
        WorkshopItem { id: 456, title: "Cool Wallpaper 2".into(), preview: "https://via.placeholder.com/200".into() },
    ]
}

#[command]
pub async fn install_wallpaper(wallpaper_id: u64) -> Result<(), String> {
    let steam = SteamCMD::new()?;
    let mut wm = WallpaperManager::new("downloads")?;

    if !wm.is_installed(wallpaper_id) {
        steam.download_workshop_item(431960, wallpaper_id)?;
    }

    wm.apply_wallpaper(wallpaper_id)?;
    Ok(())
}