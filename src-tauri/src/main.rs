#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod models;
mod repository;
mod commands;
mod calc;           // Make sure this exists

use repository::Repository;
use std::sync::Mutex;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .manage(Mutex::new(Repository::new()))
        .invoke_handler(tauri::generate_handler![
            commands::get_all_exercises,
            commands::create_exercise,
            commands::log_set,
            commands::get_current_workout,
            commands::get_workout_history,
            commands::calculate_1rm,
            commands::calculate_volume,
            commands::find_best_set
        ])
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                if let Some(window) = app.get_webview_window("main") {
                    window.open_devtools();
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}