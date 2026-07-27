use std::fs;
use std::path::Path;
use std::sync::Mutex;
use crate::models::*;

const DATA_FILE: &str = "exercise_data.json";

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct AppData {
    pub exercises: Vec<Exercise>,
    pub workouts: Vec<WorkoutSession>,
}

pub struct Repository {
    data: Mutex<AppData>,
}

impl Repository {
    pub fn new() -> Self {
        let data = if Path::new(DATA_FILE).exists() {
            fs::read_to_string(DATA_FILE)
                .ok()
                .and_then(|c| serde_json::from_str(&c).ok())
                .unwrap_or_default()
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

    // === Existing methods ===
    pub fn create_exercise(&self, ex: &mut Exercise) -> Result<(), String> {
        if ex.id.is_empty() {
            ex.id = uuid::Uuid::new_v4().to_string();
        }
        let mut data = self.data.lock().unwrap();
        data.exercises.push(ex.clone());
        drop(data);
        self.save();
        Ok(())
    }

    pub fn get_all_exercises(&self) -> Result<Vec<Exercise>, String> {
        let data = self.data.lock().unwrap();
        Ok(data.exercises.clone())
    }

     // === Current Workout ===
    pub fn get_current_workout(&self) -> Result<Option<WorkoutSession>, String> {
        let data = self.data.lock().unwrap();
        let today = chrono::Utc::now().date_naive();

        let current = data.workouts.iter()
            .find(|w| w.date.date_naive() == today)
            .cloned();

        Ok(current)
    }

    // === Logging ===
    pub fn log_set(&self, mut exercise: Exercise, set: Set) -> Result<(), String> {
        let mut data = self.data.lock().unwrap();

        // Upsert into the master exercise list, keyed by name
        if let Some(existing) = data.exercises.iter()
            .find(|e| e.name.to_lowercase() == exercise.name.to_lowercase())
        {
            exercise.id = existing.id.clone();
        } else {
            exercise.id = uuid::Uuid::new_v4().to_string();
            data.exercises.push(exercise.clone());
        }

        // Get or create today's workout
        let today = chrono::Utc::now().date_naive();
        let mut workout = data.workouts.iter_mut()
            .find(|w| w.date.date_naive() == today);

        if workout.is_none() {
            let new_workout = WorkoutSession::new();
            data.workouts.push(new_workout);
            workout = data.workouts.last_mut();
        }

        let workout = workout.unwrap();

        // Add or update exercise in the workout
        if let Some(existing) = workout.exercises.iter_mut()
            .find(|e| e.exercise.name.to_lowercase() == exercise.name.to_lowercase()) 
        {
            existing.sets.push(set);
        } else {
            workout.exercises.push(LoggedExercise {
                exercise,
                sets: vec![set],
            });
        }

        drop(data);
        self.save();
        Ok(())
    }

    pub fn get_workout_history(&self) -> Result<Vec<WorkoutSession>, String> {
        let data = self.data.lock().unwrap();
        Ok(data.workouts.clone())
    }
   
}