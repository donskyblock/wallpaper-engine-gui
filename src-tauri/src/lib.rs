mod desktop;
mod gui;
mod history;
mod kde;
mod steam;
mod wallpaper;
mod workshop;

use tauri::Builder;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    Builder::default()
        .invoke_handler(tauri::generate_handler![
            gui::dashboard_state,
            gui::fetch_workshop,
            gui::get_history,
            gui::install_wallpaper,
            gui::check_environment,
            gui::install_plugin,
            gui::install_steamcmd
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri app");
}
