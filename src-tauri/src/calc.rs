use crate::models::*;

pub struct Calc;

impl Calc {
    // Existing functions (keep them)
    pub fn calc_1rm(weight: f64, reps: i32) -> f64 {
        if reps <= 1 {
            weight
        } else {
            weight * (1.0 + reps as f64 / 30.0) // Epley formula
        }
    }

    pub fn calc_volume(weight: f64, reps: i32) -> f64 {
        weight * reps as f64
    }

    pub fn calc_total_volume(sets: &[Set]) -> f64 {
        sets.iter().map(|s| s.weight * s.reps as f64).sum()
    }

    pub fn find_best_set(sets: &[Set]) -> Option<&Set> {
        sets.iter().max_by(|a, b| 
            (a.weight * a.reps as f64)
                .partial_cmp(&(b.weight * b.reps as f64))
                .unwrap_or(std::cmp::Ordering::Equal)
        )
    }

    // === New Functions ===

    // Brzycki 1RM formula (alternative to Epley - often more accurate for low reps)
    pub fn calc_1rm_brzycki(weight: f64, reps: i32) -> f64 {
        if reps == 1 {
            weight
        } else {
            weight * (36.0 / (37.0 - reps as f64))
        }
    }

    // Estimated Training Max (90% of 1RM - common in programs)
    pub fn calc_training_max(one_rm: f64) -> f64 {
        (one_rm * 0.9).round()
    }

    // RPE-based weight suggestion (if you add RPE later)
    pub fn suggest_weight_for_rpe(one_rm: f64, reps: i32, target_rpe: f64) -> f64 {
        // Simple approximation
        one_rm * (1.0 - (target_rpe - 10.0) * 0.025)
    }

    // Weekly volume trend (simple)
    pub fn calc_weekly_volume(volumes: &[f64]) -> f64 {
        if volumes.is_empty() {
            0.0
        } else {
            volumes.iter().sum::<f64>() / volumes.len() as f64
        }
    }
}