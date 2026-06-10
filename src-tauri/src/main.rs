#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod image_decoder;

use commands::{ai, filesystem, image, updater};
use std::sync::Mutex;

fn main() {
    let ai_state = ai::AiState {
        model: Mutex::new(None),
        classifier_weights: Mutex::new(None),
        feedback_count: Mutex::new(0),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(ai_state)
        .invoke_handler(tauri::generate_handler![
            filesystem::list_drives,
            filesystem::list_directory,
            image::scan_images,
            image::get_raw_preview,
            image::export_images,
            ai::init_ai_model,
            ai::analyze_images,
            ai::download_ai_model,
            ai::get_ai_model_dir,
            ai::check_ai_model_exists,
            ai::extract_ai_model_zip,
            ai::mark_image_feedback,
            ai::retrain_classifier,
            ai::get_feedback_count,
            ai::get_feedback_data,
            ai::detect_duplicates,
            ai::mark_duplicates_as_waste,
            ai::save_model_path,
            ai::load_model_path,
            updater::check_for_updates,
            updater::install_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
