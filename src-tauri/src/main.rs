#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod face_detector;
mod image_decoder;
mod quality_analyzer;
mod wedding_analyzer;
mod waste_detector;

use commands::{filesystem, image, updater};
use waste_detector::WasteDetectorState;

fn main() {
    let waste_state = WasteDetectorState::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(waste_state)
        .invoke_handler(tauri::generate_handler![
            filesystem::list_drives,
            filesystem::list_directory,
            image::scan_images,
            image::get_raw_preview,
            image::export_images,
            waste_detector::init_waste_detector,
            waste_detector::analyze_waste_images,
            waste_detector::mark_waste_feedback,
            waste_detector::get_waste_feedback_count,
            waste_detector::get_waste_config,
            waste_detector::update_waste_config,
            updater::check_for_updates,
            updater::install_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
