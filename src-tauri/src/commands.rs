use crate::models::*;
use crate::repository::Repository;
use tauri::State;
use std::sync::Mutex;

// We use a global state for the repository
type RepoState = State<'static, Mutex<Repository>>;

#[tauri::command]
pub fn get_all_exercises(state: RepoState) -> Result<Vec<Exercise>, String> {
    let repo = state.lock().map_err(|e| e.to_string())?;
    repo.get_all_exercises()
}

#[tauri::command]
pub fn create_exercise(state: RepoState, mut exercise: Exercise) -> Result<Exercise, String> {
    let repo = state.lock().map_err(|e| e.to_string())?;
    repo.create_exercise(&mut exercise)?;
    Ok(exercise)
}

#[tauri::command]
pub fn log_set(state: RepoState, exercise: Exercise, set: Set) -> Result<(), String> {
    let repo = state.lock().map_err(|e| e.to_string())?;
    repo.log_set(exercise, set)
}

// Future commands
#[tauri::command]
pub fn get_workout_history(state: RepoState) -> Result<Vec<WorkoutSession>, String> {
    // TODO: implement later
    Ok(vec![])
}