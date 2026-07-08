// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod models;
mod repository;
mod commands;
mod calc;

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
            commands::calc_1rm,              
            commands::calc_volume,           
            commands::calc_total_volume,
            commands::find_best_set,
            commands::calc_1rm_brzycki,
            commands::calc_training_max,
            commands::suggest_weight_for_rpe,
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