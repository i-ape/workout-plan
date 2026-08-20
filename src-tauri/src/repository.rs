use std::fs;
use std::path::Path;
use std::sync::Mutex;
use crate::models::*;
use chrono::Datelike;
use crate::calc::Calc;

const DATA_FILE: &str = "exercise_data.json";

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct AppData {
    pub exercises: Vec<Exercise>,
    pub workouts: Vec<WorkoutSession>,
}

pub struct Repository {
    data: Mutex<AppData>,
}

fn escape_csv_field(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
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

    // === Exercises ===
    pub fn create_exercise(&self, ex: &mut Exercise) -> Result<(), String> {
        if ex.id.is_empty() {
            *ex = Exercise::new(ex.name.clone(), ex.category.clone());
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

    // === Current Workout & Logging ===
    pub fn get_current_workout(&self) -> Result<Option<WorkoutSession>, String> {
        let data = self.data.lock().unwrap();
        let today = chrono::Utc::now().date_naive();

        let current = data.workouts.iter()
            .find(|w| w.date.date_naive() == today)
            .cloned();

        Ok(current)
    }

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

    // === Editing & Deleting (today's workout only) ===
    pub fn delete_set(&self, exercise_name: &str, set_index: usize) -> Result<(), String> {
        let mut data = self.data.lock().unwrap();
        let today = chrono::Utc::now().date_naive();

        let workout = data.workouts.iter_mut()
            .find(|w| w.date.date_naive() == today)
            .ok_or("No workout logged today")?;

        let logged = workout.exercises.iter_mut()
            .find(|e| e.exercise.name.to_lowercase() == exercise_name.to_lowercase())
            .ok_or("Exercise not found in today's workout")?;

        if set_index >= logged.sets.len() {
            return Err("Set index out of range".to_string());
        }

        logged.sets.remove(set_index);

        // If that was the last set for this exercise, drop the exercise entry too
        if logged.sets.is_empty() {
            workout.exercises.retain(|e| e.exercise.name.to_lowercase() != exercise_name.to_lowercase());
        }

        drop(data);
        self.save();
        Ok(())
    }

    pub fn edit_set(&self, exercise_name: &str, set_index: usize, set: Set) -> Result<(), String> {
        let mut data = self.data.lock().unwrap();
        let today = chrono::Utc::now().date_naive();

        let workout = data.workouts.iter_mut()
            .find(|w| w.date.date_naive() == today)
            .ok_or("No workout logged today")?;

        let logged = workout.exercises.iter_mut()
            .find(|e| e.exercise.name.to_lowercase() == exercise_name.to_lowercase())
            .ok_or("Exercise not found in today's workout")?;

        if set_index >= logged.sets.len() {
            return Err("Set index out of range".to_string());
        }

        logged.sets[set_index] = set;

        drop(data);
        self.save();
        Ok(())
    }

    // === Calendar / Date-based Lookups ===
    pub fn get_workout_by_date(&self, date: chrono::NaiveDate) -> Result<Option<WorkoutSession>, String> {
        let data = self.data.lock().unwrap();
        let workout = data.workouts.iter()
            .find(|w| w.date.date_naive() == date)
            .cloned();
        Ok(workout)
    }

    pub fn get_workout_dates(&self) -> Result<Vec<String>, String> {
        let data = self.data.lock().unwrap();
        let dates: Vec<String> = data.workouts.iter()
            .map(|w| w.date.date_naive().to_string())
            .collect();
        Ok(dates)
    }

    pub fn get_workout_history(&self) -> Result<Vec<WorkoutSession>, String> {
        let data = self.data.lock().unwrap();
        Ok(data.workouts.clone())
    }

    // === Exercise Progress ===
    pub fn get_exercise_progress(&self, exercise_name: &str) -> Result<Vec<(String, f64)>, String> {
        let data = self.data.lock().unwrap();
        let name_lower = exercise_name.to_lowercase();
        let mut progress = vec![];

        for workout in &data.workouts {
            for logged in &workout.exercises {
                if logged.exercise.name.to_lowercase() == name_lower {
                    if let Some(best) = Calc::calc_best_set(&logged.sets) {
                        let one_rm = Calc::calc_one_rm_epley(best.weight, best.reps);
                        let date = workout.date.date_naive().to_string();
                        progress.push((date, one_rm));
                    }
                }
            }
        }

        Ok(progress)
    }

    // === Weekly Volume Trend ===
    pub fn get_weekly_volume_trend(&self) -> Result<Vec<(String, f64)>, String> {
        let data = self.data.lock().unwrap();
        use std::collections::BTreeMap;

        // BTreeMap keeps weeks in chronological order for free
        let mut weekly: BTreeMap<(i32, u32), f64> = BTreeMap::new();

        for workout in &data.workouts {
            let iso = workout.date.iso_week();
            let key = (iso.year(), iso.week());
            let session_volume: f64 = workout.exercises.iter()
                .map(|logged| Calc::calc_total_volume(&logged.sets))
                .sum();
            *weekly.entry(key).or_insert(0.0) += session_volume;
        }

        let result = weekly.into_iter()
            .map(|((year, week), volume)| (format!("{}-W{:02}", year, week), volume))
            .collect();

        Ok(result)
    }

    // === Body Part Focus ===
    pub fn get_category_volume(&self) -> Result<Vec<(String, f64)>, String> {
        let data = self.data.lock().unwrap();
        let mut totals: std::collections::HashMap<String, f64> = std::collections::HashMap::new();

        for workout in &data.workouts {
            for logged in &workout.exercises {
                let volume = Calc::calc_total_volume(&logged.sets);
                *totals.entry(logged.exercise.category.clone()).or_insert(0.0) += volume;
            }
        }

        let mut result: Vec<(String, f64)> = totals.into_iter().collect();
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        Ok(result)
    }

    pub fn get_last_trained_by_category(&self) -> Result<Vec<(String, String)>, String> {
        let data = self.data.lock().unwrap();
        let mut last_trained: std::collections::HashMap<String, chrono::NaiveDate> = std::collections::HashMap::new();

        for workout in &data.workouts {
            let date = workout.date.date_naive();
            for logged in &workout.exercises {
                let category = logged.exercise.category.clone();
                last_trained.entry(category)
                    .and_modify(|d| if date > *d { *d = date; })
                    .or_insert(date);
            }
        }

        let mut result: Vec<(String, String)> = last_trained.into_iter()
            .map(|(cat, date)| (cat, date.to_string()))
            .collect();
        result.sort_by(|a, b| b.1.cmp(&a.1));
        Ok(result)
    }

    // === Export ===
    pub fn export_to_csv(&self) -> Result<String, String> {
        let data = self.data.lock().unwrap();
        let mut csv = String::from("date,exercise,category,reps,weight,rpe\n");

        for workout in &data.workouts {
            let date = workout.date.date_naive().to_string();
            for logged in &workout.exercises {
                for set in &logged.sets {
                    let rpe_str = set.rpe.map(|r| r.to_string()).unwrap_or_default();
                    csv.push_str(&format!(
                        "{},{},{},{},{},{}\n",
                        date,
                        escape_csv_field(&logged.exercise.name),
                        escape_csv_field(&logged.exercise.category),
                        set.reps,
                        set.weight,
                        rpe_str
                    ));
                }
            }
        }

        Ok(csv)
    }
}