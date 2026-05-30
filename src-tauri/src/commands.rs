use crate::models::*;
use crate::repository::Repository;
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
pub fn get_workout_history(state: State<'_, Mutex<Repository>>) -> Result<Vec<WorkoutSession>, String> {
    let repo = state.lock().map_err(|e| e.to_string())?;
    repo.get_workout_history()
}