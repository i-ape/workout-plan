use serde_json;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use crate::models::*;

const DATA_FILE: &str = "exercise_data.json";

pub struct Repository {
    data: Mutex<AppData>,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct AppData {
    pub exercises: Vec<Exercise>,
    pub workouts: Vec<WorkoutSession>,
}

impl Repository {
    pub fn new() -> Self {
        let data = if Path::new(DATA_FILE).exists() {
            match fs::read_to_string(DATA_FILE) {
                Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
                Err(_) => AppData::default(),
            }
        } else {
            AppData::default()
        };

        Repository { data: Mutex::new(data) }
    }

    fn save(&self) {
        let data = self.data.lock().unwrap();
        if let Ok(json) = serde_json::to_string_pretty(&*data) {
            let _ = fs::write(DATA_FILE, json);
        }
    }

    // === Public Methods ===
    pub fn create_exercise(&self, ex: &mut Exercise) -> Result<(), String> {
        if ex.id.is_empty() {
            ex.id = uuid::Uuid::new_v4().to_string();
        }

        let mut data = self.data.lock().unwrap();
        data.exercises.push(ex.clone());
        drop(data); // release lock before save
        self.save();
        Ok(())
    }

    pub fn get_all_exercises(&self) -> Result<Vec<Exercise>, String> {
        let data = self.data.lock().unwrap();
        Ok(data.exercises.clone())
    }

    pub fn log_set(&self, exercise: Exercise, set: Set) -> Result<(), String> {
        // For now: simple implementation - we'll improve this soon
        println!("Logged set: {} × {}kg", set.reps, set.weight);
        // TODO: Add to current workout or create new one
        Ok(())
    }
}