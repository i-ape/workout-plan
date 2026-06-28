use crate::models::*;
use crate::repository::Repository;
use crate::calc::Calc;
use tauri::State;
use std::sync::Mutex;

#[tauri::command]
pub fn get_all_exercises(state: State<'_, Mutex<Repository>>) -> Result<Vec<Exercise>, String> {
    let repo = state.lock().map_err(|e| e.to_string())?;
    repo.get_all_exercises()
}

#[tauri::command]
pub fn create_exercise(
    state: State<'_, Mutex<Repository>>,
    mut exercise: Exercise
) -> Result<Exercise, String> {
    let repo = state.lock().map_err(|e| e.to_string())?;
    repo.create_exercise(&mut exercise)?;
    Ok(exercise)
}

#[tauri::command]
pub fn log_set(
    state: State<'_, Mutex<Repository>>,
    exercise: Exercise,
    set: Set
) -> Result<(), String> {
    let repo = state.lock().map_err(|e| e.to_string())?;
    repo.log_set(exercise, set)
}

#[tauri::command]
pub fn get_current_workout(state: State<'_, Mutex<Repository>>) -> Result<Option<WorkoutSession>, String> {
    let repo = state.lock().map_err(|e| e.to_string())?;
    repo.get_current_workout()
}

#[tauri::command]
pub fn get_workout_history(state: State<'_, Mutex<Repository>>) -> Result<Vec<WorkoutSession>, String> {
    let repo = state.lock().map_err(|e| e.to_string())?;
    repo.get_workout_history()
}

// === New Exercise Logic Functions ===

#[tauri::command]
pub fn calculate_1rm(weight: f64, reps: i32) -> Result<f64, String> {
    Ok(Calc::calculate_1rm(weight, reps))
}

#[tauri::command]
pub fn calculate_volume(weight: f64, reps: i32) -> Result<f64, String> {
    Ok(Calc::calculate_volume(weight, reps))
}

#[tauri::command]
pub fn calculate_total_volume(sets: Vec<Set>) -> Result<f64, String> {
    Ok(Calc::calculate_total_volume(&sets))
}

#[tauri::command]
pub fn find_best_set(sets: Vec<Set>) -> Result<Option<Set>, String> {
    Ok(Calc::find_best_set(&sets).cloned())
}

#[tauri::command]
pub fn get_personal_records(state: State<'_, Mutex<Repository>>) -> Result<Vec<(String, Set)>, String> {
    // Simple implementation - can be expanded
    let repo = state.lock().map_err(|e| e.to_string())?;
    let exercises = repo.get_all_exercises()?;
    let mut records = vec![];

    for ex in exercises {
        // For now, return a dummy best set - improve later
        records.push((ex.name, Set { reps: 5, weight: 100.0 }));
    }

    Ok(records)
}