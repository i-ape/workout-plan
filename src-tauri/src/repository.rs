use std::fs;
use std::path::Path;
use std::sync::Mutex;
use crate::models::*;
use chrono::Datelike;
use crate::calc::Calc;

const DATA_FILE: &str = "exercise_data.json";
const BACKUP_DIR: &str = "backups";

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct AppData {
    pub exercises: Vec<Exercise>,
    pub workouts: Vec<WorkoutSession>,
    #[serde(default)]
    pub routines: Vec<Routine>,
}

pub struct Repository {
    data: Mutex<AppData>,
    data_file: String,
    backup_dir: String,
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
        Self::new_at(DATA_FILE, BACKUP_DIR)
    }

    /// Used by tests (and available generally) to point at an isolated
    /// data file / backup directory instead of the real ones.
    pub fn new_at(data_file: &str, backup_dir: &str) -> Self {
        // Clean up any leftover temp file from a crash during a previous save
        let tmp_path = format!("{}.tmp", data_file);
        if Path::new(&tmp_path).exists() {
            let _ = fs::remove_file(&tmp_path);
        }

        let data = if Path::new(data_file).exists() {
            fs::read_to_string(data_file)
                .ok()
                .and_then(|c| serde_json::from_str(&c).ok())
                .unwrap_or_default()
        } else {
            AppData::default()
        };

        Repository {
            data: Mutex::new(data),
            data_file: data_file.to_string(),
            backup_dir: backup_dir.to_string(),
        }
    }

    fn save(&self) {
        let data = self.data.lock().unwrap();
        let json = match serde_json::to_string_pretty(&*data) {
            Ok(j) => j,
            Err(_) => return, // don't touch the file if serialization somehow failed
        };
        drop(data);

        let tmp_path = format!("{}.tmp", self.data_file);

        if fs::write(&tmp_path, &json).is_err() {
            return; // couldn't even write the temp file, bail without touching the real one
        }

        // Rename is atomic on virtually all filesystems: readers see either
        // the old complete file or the new complete file, never a partial one.
        let _ = fs::rename(&tmp_path, &self.data_file);
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

    // === Lifetime Stats ===
    pub fn get_lifetime_stats(&self) -> Result<(i32, f64, i32), String> {
        let data = self.data.lock().unwrap();

        let mut total_sets = 0;
        let mut total_volume = 0.0;

        for workout in &data.workouts {
            for logged in &workout.exercises {
                total_sets += logged.sets.len() as i32;
                total_volume += Calc::calc_total_volume(&logged.sets);
            }
        }

        let total_workouts = data.workouts.len() as i32;

        Ok((total_sets, total_volume, total_workouts))
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

    // === Routines ===
    pub fn create_routine(&self, name: &str, exercise_names: Vec<String>) -> Result<Routine, String> {
        let routine = Routine::new(name, exercise_names);
        let mut data = self.data.lock().unwrap();
        data.routines.push(routine.clone());
        drop(data);
        self.save();
        Ok(routine)
    }

    pub fn get_routines(&self) -> Result<Vec<Routine>, String> {
        let data = self.data.lock().unwrap();
        Ok(data.routines.clone())
    }

    pub fn delete_routine(&self, routine_id: &str) -> Result<(), String> {
        let mut data = self.data.lock().unwrap();
        data.routines.retain(|r| r.id != routine_id);
        drop(data);
        self.save();
        Ok(())
    }

    pub fn edit_routine(&self, routine_id: &str, name: &str, exercise_names: Vec<String>) -> Result<Routine, String> {
        let mut data = self.data.lock().unwrap();

        let routine = data.routines.iter_mut()
            .find(|r| r.id == routine_id)
            .ok_or("Routine not found")?;

        routine.name = name.to_string();
        routine.exercise_names = exercise_names;
        let updated = routine.clone();

        drop(data);
        self.save();
        Ok(updated)
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

    // === Backup & Restore ===
    pub fn create_backup(&self) -> Result<String, String> {
        fs::create_dir_all(&self.backup_dir).map_err(|e| e.to_string())?;

        let timestamp = chrono::Utc::now().format("%Y-%m-%d_%H-%M-%S");
        let backup_path = format!("{}/exercise_data_{}.json", self.backup_dir, timestamp);

        let data = self.data.lock().unwrap();
        let json = serde_json::to_string_pretty(&*data).map_err(|e| e.to_string())?;
        drop(data);

        fs::write(&backup_path, json).map_err(|e| e.to_string())?;
        Ok(backup_path)
    }

    pub fn list_backups(&self) -> Result<Vec<String>, String> {
        if !Path::new(&self.backup_dir).exists() {
            return Ok(vec![]);
        }

        let mut entries: Vec<String> = fs::read_dir(&self.backup_dir)
            .map_err(|e| e.to_string())?
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| entry.file_name().into_string().ok())
            .filter(|name| name.ends_with(".json"))
            .collect();

        entries.sort();
        entries.reverse(); // most recent first
        Ok(entries)
    }

    pub fn restore_backup(&self, filename: &str) -> Result<(), String> {
        // Guard against path traversal - only allow bare filenames
        if filename.contains('/') || filename.contains("..") {
            return Err("Invalid backup filename".to_string());
        }

        let backup_path = format!("{}/{}", self.backup_dir, filename);
        let content = fs::read_to_string(&backup_path).map_err(|e| e.to_string())?;
        let restored: AppData = serde_json::from_str(&content).map_err(|e| e.to_string())?;

        // Snapshot current state before overwriting, so a bad restore can itself be undone
        let _ = self.create_backup();

        let mut data = self.data.lock().unwrap();
        *data = restored;
        drop(data);

        self.save();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Each test gets its own isolated data file/backup dir so tests never
    // collide with each other or with your real exercise_data.json.
    fn test_repo(test_name: &str) -> Repository {
        let data_file = format!("test_data_{}.json", test_name);
        let backup_dir = format!("test_backups_{}", test_name);
        let _ = fs::remove_file(&data_file);
        let _ = fs::remove_dir_all(&backup_dir);
        Repository::new_at(&data_file, &backup_dir)
    }

    fn cleanup(test_name: &str) {
        let _ = fs::remove_file(format!("test_data_{}.json", test_name));
        let _ = fs::remove_file(format!("test_data_{}.json.tmp", test_name));
        let _ = fs::remove_dir_all(format!("test_backups_{}", test_name));
    }

    #[test]
    fn log_set_creates_new_exercise_and_workout() {
        let repo = test_repo("log_new");
        let exercise = Exercise::new("Bench Press", "Chest");
        let set = Set::new(80.0, 8);

        repo.log_set(exercise, set).unwrap();

        let exercises = repo.get_all_exercises().unwrap();
        assert_eq!(exercises.len(), 1);
        assert_eq!(exercises[0].name, "Bench Press");

        let workout = repo.get_current_workout().unwrap().unwrap();
        assert_eq!(workout.exercises.len(), 1);
        assert_eq!(workout.exercises[0].sets.len(), 1);

        cleanup("log_new");
    }

    #[test]
    fn log_set_upserts_existing_exercise_by_name_case_insensitive() {
        let repo = test_repo("upsert");
        repo.log_set(Exercise::new("Squat", "Legs"), Set::new(100.0, 5)).unwrap();
        repo.log_set(Exercise::new("squat", "Legs"), Set::new(105.0, 5)).unwrap();

        let exercises = repo.get_all_exercises().unwrap();
        assert_eq!(exercises.len(), 1, "should not create a duplicate exercise for different casing");

        let workout = repo.get_current_workout().unwrap().unwrap();
        assert_eq!(workout.exercises.len(), 1);
        assert_eq!(workout.exercises[0].sets.len(), 2, "both sets should land under the same exercise");

        cleanup("upsert");
    }

    #[test]
    fn delete_set_removes_exercise_when_last_set_removed() {
        let repo = test_repo("delete_last");
        repo.log_set(Exercise::new("Deadlift", "Back"), Set::new(120.0, 3)).unwrap();

        repo.delete_set("Deadlift", 0).unwrap();

        let workout = repo.get_current_workout().unwrap();
        let has_deadlift = workout
            .map(|w| w.exercises.iter().any(|e| e.exercise.name == "Deadlift"))
            .unwrap_or(false);
        assert!(!has_deadlift, "exercise should be removed once its only set is deleted");

        cleanup("delete_last");
    }

    #[test]
    fn delete_set_out_of_range_returns_error() {
        let repo = test_repo("delete_oob");
        repo.log_set(Exercise::new("Row", "Back"), Set::new(60.0, 10)).unwrap();

        let result = repo.delete_set("Row", 5);
        assert!(result.is_err());

        cleanup("delete_oob");
    }

    #[test]
    fn edit_set_updates_values_in_place() {
        let repo = test_repo("edit_set");
        repo.log_set(Exercise::new("OHP", "Shoulders"), Set::new(40.0, 6)).unwrap();

        repo.edit_set("OHP", 0, Set::new(45.0, 5)).unwrap();

        let workout = repo.get_current_workout().unwrap().unwrap();
        let set = &workout.exercises[0].sets[0];
        assert_eq!(set.weight, 45.0);
        assert_eq!(set.reps, 5);

        cleanup("edit_set");
    }

    #[test]
    fn create_routine_and_get_routines_roundtrip() {
        let repo = test_repo("routine_crud");
        let routine = repo.create_routine("Push Day", vec!["Bench".into(), "OHP".into()]).unwrap();

        let routines = repo.get_routines().unwrap();
        assert_eq!(routines.len(), 1);
        assert_eq!(routines[0].id, routine.id);
        assert_eq!(routines[0].exercise_names, vec!["Bench", "OHP"]);

        cleanup("routine_crud");
    }

    #[test]
    fn edit_routine_updates_existing_by_id() {
        let repo = test_repo("routine_edit");
        let routine = repo.create_routine("Leg Day", vec!["Squat".into()]).unwrap();

        repo.edit_routine(&routine.id, "Leg Day v2", vec!["Squat".into(), "Lunge".into()]).unwrap();

        let routines = repo.get_routines().unwrap();
        assert_eq!(routines.len(), 1, "editing should not create a duplicate");
        assert_eq!(routines[0].name, "Leg Day v2");
        assert_eq!(routines[0].exercise_names.len(), 2);

        cleanup("routine_edit");
    }

    #[test]
    fn edit_routine_missing_id_returns_error() {
        let repo = test_repo("routine_edit_missing");
        let result = repo.edit_routine("nonexistent-id", "X", vec![]);
        assert!(result.is_err());

        cleanup("routine_edit_missing");
    }

    #[test]
    fn delete_routine_removes_it() {
        let repo = test_repo("routine_delete");
        let routine = repo.create_routine("Temp", vec!["X".into()]).unwrap();

        repo.delete_routine(&routine.id).unwrap();

        let routines = repo.get_routines().unwrap();
        assert!(routines.is_empty());

        cleanup("routine_delete");
    }

    #[test]
    fn backup_and_restore_roundtrip() {
        let repo = test_repo("backup_restore");
        repo.log_set(Exercise::new("Curl", "Arms"), Set::new(15.0, 12)).unwrap();

        let backup_path = repo.create_backup().unwrap();
        let filename = Path::new(&backup_path).file_name().unwrap().to_str().unwrap().to_string();

        // Change data after the backup
        repo.log_set(Exercise::new("Curl", "Arms"), Set::new(17.5, 10)).unwrap();
        assert_eq!(repo.get_current_workout().unwrap().unwrap().exercises[0].sets.len(), 2);

        // Restore should bring it back to the 1-set state
        repo.restore_backup(&filename).unwrap();
        assert_eq!(repo.get_current_workout().unwrap().unwrap().exercises[0].sets.len(), 1);

        cleanup("backup_restore");
    }

    #[test]
    fn restore_backup_rejects_path_traversal() {
        let repo = test_repo("path_traversal");
        let result = repo.restore_backup("../../etc/passwd");
        assert!(result.is_err());

        cleanup("path_traversal");
    }
}