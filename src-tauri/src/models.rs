use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Exercise {
    pub id: String,
    pub name: String,
    pub category: String,
    #[serde(default)]
    pub notes: Option<String>,
}

impl Exercise {
    pub fn new(name: impl Into<String>, category: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            category: category.into(),
            notes: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Set {
    pub reps: i32,
    pub weight: f64,
    #[serde(default)]
    pub rpe: Option<f64>,
}

impl Set {
    pub fn new(weight: f64, reps: i32) -> Self {
        Self { reps, weight, rpe: None }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoggedExercise {
    pub exercise: Exercise,
    pub sets: Vec<Set>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkoutSession {
    pub id: String,
    pub date: DateTime<Utc>,
    pub exercises: Vec<LoggedExercise>,
    #[serde(default)]
    pub notes: Option<String>,
}

impl WorkoutSession {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            date: Utc::now(),
            exercises: vec![],
            notes: None,
        }
    }
}

impl Default for WorkoutSession {
    fn default() -> Self {
        Self::new()
    }
}