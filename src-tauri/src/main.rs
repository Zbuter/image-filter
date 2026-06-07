#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod image_decoder;

use commands::{filesystem, image, updater};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            filesystem::list_drives,
            filesystem::list_directory,
            image::scan_images,
            image::get_raw_preview,
            image::export_images,
            updater::check_for_updates,
            updater::install_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
