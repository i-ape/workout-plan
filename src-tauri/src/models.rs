use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Exercise {
    pub id: String,
    pub name: String,
    pub category: String,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Set {
    pub reps: i32,
    pub weight: f64,
    pub rpe: Option<f64>,
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