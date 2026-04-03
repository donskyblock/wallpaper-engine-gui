mod steam;
mod wallpaper;

use wallpaper::WallpaperManager;
use steam::SteamCMD;

#[tokio::main]
async fn main() {
    println!("Starting Rust Wallpaper Engine...");

    let steam = SteamCMD::new().expect("Failed to init SteamCMD");
    let mut wm = WallpaperManager::new("downloads").expect("Failed to init WallpaperManager");

    let wallpaper_id = 123456789; // replace with actual Workshop ID
    if !wm.is_installed(wallpaper_id) {
        println!("Wallpaper not installed, downloading...");
        steam.download_workshop_item(431960, wallpaper_id).expect("Failed to download wallpaper");
    }

    wm.apply_wallpaper(wallpaper_id).expect("Failed to apply wallpaper");
    println!("Wallpaper applied successfully!");
}