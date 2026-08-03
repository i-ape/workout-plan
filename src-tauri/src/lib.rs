pub mod repository;
pub mod models;
pub mod commands;
pub mod calc;

use commands::*;
use repository::Repository;
use std::sync::Mutex;
use tauri::Manager;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Mutex::new(Repository::new()))
        .invoke_handler(tauri::generate_handler![
            greet,
            get_all_exercises,
            create_exercise,
            log_set,
            get_current_workout,
            get_workout_history,
            calc_1rm,
            calc_volume,
            calc_total_volume,
            find_best_set,
            calc_1rm_brzycki,
            calc_training_max,
            suggest_weight_for_rpe,
            calc_weekly_volume,
            calc_progress_percent,
            get_personal_records,
            get_exercise_progress,
            get_weekly_volume_trend,
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