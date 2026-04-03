use std::process::Command;
use std::path::PathBuf;

#[derive(Debug)]
pub enum KdeStatus {
    PluginInstalled,
    KdeButNoPlugin,
    NotKde,
}

pub fn detect_environment() -> KdeStatus {
    // Check if running KDE by looking at XDG_CURRENT_DESKTOP or DESKTOP_SESSION
    let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default().to_lowercase();
    let session = std::env::var("DESKTOP_SESSION").unwrap_or_default().to_lowercase();

    let is_kde = desktop.contains("kde") || session.contains("plasma") || session.contains("kde");

    if !is_kde {
        return KdeStatus::NotKde;
    }

    // Check if the plugin is installed by looking for its plasmoid
    let plugin_paths = [
        PathBuf::from(std::env::var("HOME").unwrap_or_default())
            .join(".local/share/plasma/wallpapers/com.github.casout.wallpaperEngineKde"),
        PathBuf::from("/usr/share/plasma/wallpapers/com.github.casout.wallpaperEngineKde"),
    ];

    for path in &plugin_paths {
        if path.exists() {
            return KdeStatus::PluginInstalled;
        }
    }

    KdeStatus::KdeButNoPlugin
}

pub fn install_plugin() -> Result<(), String> {
    // Clone and build the plugin
    let home = std::env::var("HOME").map_err(|e| e.to_string())?;
    let clone_path = format!("{}/wallpaper-engine-kde-plugin", home);

    if !std::path::Path::new(&clone_path).exists() {
        let status = Command::new("git")
            .args(["clone", "https://github.com/catsout/wallpaper-engine-kde-plugin.git", &clone_path])
            .status()
            .map_err(|e| format!("Failed to clone plugin repo: {}", e))?;

        if !status.success() {
            return Err("git clone failed".into());
        }
    }

    // Build with cmake
    let build_path = format!("{}/build", clone_path);
    std::fs::create_dir_all(&build_path).map_err(|e| e.to_string())?;

    let cmake = Command::new("cmake")
        .args(["-DUSE_PLASMAPKG=ON", ".."])
        .current_dir(&build_path)
        .status()
        .map_err(|e| format!("cmake failed: {}", e))?;

    if !cmake.success() {
        return Err("cmake configuration failed".into());
    }

    let make = Command::new("cmake")
        .args(["--build", ".", "--", "-j4"])
        .current_dir(&build_path)
        .status()
        .map_err(|e| format!("build failed: {}", e))?;

    if !make.success() {
        return Err("build failed".into());
    }

    let install = Command::new("cmake")
        .args(["--install", "."])
        .current_dir(&build_path)
        .status()
        .map_err(|e| format!("install failed: {}", e))?;

    if !install.success() {
        return Err("install failed".into());
    }

    Ok(())
}