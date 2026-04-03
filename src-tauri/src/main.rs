#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod steam;
mod wallpaper;
mod gui;
mod kde;

use tauri::Builder;

fn main() {
    Builder::default()
        .invoke_handler(tauri::generate_handler![
            gui::fetch_workshop,
            gui::install_wallpaper,
            gui::check_environment,
            gui::install_plugin
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri app");
}