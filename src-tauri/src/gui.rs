use crate::desktop::{DesktopBackend, DesktopState};
use crate::history::{self, HistoryItem};
use crate::steam::{SteamCMD, SteamState};
use crate::wallpaper::WallpaperManager;
use crate::workshop::{self, WorkshopItem};
use serde::Serialize;

#[derive(Serialize)]
pub struct DashboardState {
    pub environment: DesktopState,
    pub steam: SteamState,
    pub ready: bool,
    pub message: String,
    pub history: Vec<HistoryItem>,
}

#[derive(Serialize)]
pub struct ActionResponse {
    pub message: String,
    pub dashboard: DashboardState,
}

#[tauri::command]
pub fn dashboard_state() -> Result<DashboardState, String> {
    build_dashboard_state()
}

#[tauri::command]
pub fn check_environment() -> Result<DashboardState, String> {
    build_dashboard_state()
}

#[tauri::command]
pub fn install_plugin() -> Result<ActionResponse, String> {
    let backend = DesktopBackend::detect();
    backend.install_plugin()?;

    Ok(ActionResponse {
        message: "The desktop wallpaper plugin was installed for the active backend.".into(),
        dashboard: build_dashboard_state()?,
    })
}

#[tauri::command]
pub fn install_steamcmd() -> Result<ActionResponse, String> {
    SteamCMD::install_managed()?;
    Ok(ActionResponse {
        message: "SteamCMD was installed into the app runtime.".into(),
        dashboard: build_dashboard_state()?,
    })
}

#[tauri::command]
pub fn fetch_workshop(query: Option<String>) -> Result<Vec<WorkshopItem>, String> {
    workshop::browse_workshop(query)
}

#[tauri::command]
pub fn get_history() -> Result<Vec<HistoryItem>, String> {
    history::load_history()
}

#[tauri::command]
pub fn install_wallpaper(
    wallpaper_id: u64,
    title: Option<String>,
    preview: Option<String>,
    creator: Option<String>,
) -> Result<ActionResponse, String> {
    let steam = SteamCMD::new()?;
    let manager = WallpaperManager::new();
    let wallpaper_path = steam.workshop_path(wallpaper_id);

    let resolved_path = if manager.is_installed(&wallpaper_path) {
        wallpaper_path
    } else {
        steam.download_workshop_item(wallpaper_id)?
    };

    manager.apply_wallpaper(wallpaper_id, &resolved_path)?;

    let enriched = if title.is_none() || preview.is_none() || creator.is_none() {
        workshop::fetch_workshop_item_details(wallpaper_id).ok()
    } else {
        None
    };

    history::record(HistoryItem {
        wallpaper_id,
        title: title.or_else(|| enriched.as_ref().map(|item| item.title.clone())),
        preview: preview.or_else(|| enriched.as_ref().map(|item| item.preview.clone())),
        creator: creator.or_else(|| enriched.as_ref().map(|item| item.creator.clone())),
        applied_at: workshop::unix_timestamp_now(),
        local_path: resolved_path.to_string_lossy().into_owned(),
    })?;

    Ok(ActionResponse {
        message: format!(
            "Wallpaper {wallpaper_id} is now active in the configured desktop backend."
        ),
        dashboard: build_dashboard_state()?,
    })
}

fn build_dashboard_state() -> Result<DashboardState, String> {
    let backend = DesktopBackend::detect();
    let environment = backend.state()?;
    let steam = SteamCMD::state()?;
    let history = history::load_history()?;
    let ready = environment.supported && environment.plugin_installed && steam.installed;

    let message = if ready {
        "Setup complete. You can search, install, and apply wallpapers now.".into()
    } else if !environment.supported {
        format!(
            "Detected desktop: {}. KDE Plasma is implemented today, and the backend layer is ready for other desktops later.",
            environment.detected_session
        )
    } else if !steam.installed && !environment.plugin_installed {
        "SteamCMD and the desktop plugin still need to be installed.".into()
    } else if !steam.installed {
        "Install SteamCMD to enable live workshop downloads.".into()
    } else {
        "Install the desktop plugin to start applying wallpapers.".into()
    };

    Ok(DashboardState {
        environment,
        steam,
        ready,
        message,
        history,
    })
}
