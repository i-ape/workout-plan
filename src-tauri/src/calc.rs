use crate::models::*;

pub struct CalcLogic;

impl CalcLogic {
    /// Calculate estimated 1RM using Epley formula
    pub fn calculate_1rm(weight: f64, reps: i32) -> f64 {
        if reps == 1 {
            weight
        } else {
            weight * (1.0 + reps as f64 / 30.0)
        }
    }

    /// Calculate total volume of a set
    pub fn calculate_volume(weight: f64, reps: i32) -> f64 {
        weight * reps as f64
    }

    /// Get best set from a list (simple PR logic)
    pub fn find_best_set(sets: &[Set]) -> Option<&Set> {
        sets.iter().max_by(|a, b| 
            (a.weight * a.reps as f64)
                .partial_cmp(&(b.weight * b.reps as f64))
                .unwrap()
        )
    }
}