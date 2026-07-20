use crate::models::*;

pub struct Calc;

impl Calc {
    // --- 1RM Calculations ---

    /// Epley formula. Generally more accurate for higher rep ranges.
    pub fn calc_one_rm_epley(weight: f64, reps: i32) -> f64 {
        if reps <= 1 {
            weight
        } else {
            weight * (1.0 + reps as f64 / 30.0)
        }
    }

    /// Brzycki formula. Often more accurate for low rep ranges.
    pub fn calc_one_rm_brzycki(weight: f64, reps: i32) -> f64 {
        if reps <= 1 {
            weight
        } else {
            weight * (36.0 / (37.0 - reps as f64))
        }
    }

    // --- Volume Calculations ---

    pub fn calc_volume(weight: f64, reps: i32) -> f64 {
        weight * reps as f64
    }

    pub fn calc_total_volume(sets: &[Set]) -> f64 {
        sets.iter().map(|s| Self::calc_volume(s.weight, s.reps)).sum()
    }

    pub fn calc_weekly_volume(volumes: &[f64]) -> f64 {
        if volumes.is_empty() {
            0.0
        } else {
            volumes.iter().sum::<f64>() / volumes.len() as f64
        }
    }

    // --- Best Set & Records ---

    pub fn calc_best_set(sets: &[Set]) -> Option<&Set> {
        sets.iter().max_by(|a, b| {
            Self::calc_volume(a.weight, a.reps)
                .partial_cmp(&Self::calc_volume(b.weight, b.reps))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    // --- Training & Planning ---

    /// Estimated training max (90% of 1RM - common in programs).
    pub fn calc_training_max(one_rm: f64) -> f64 {
        (one_rm * 0.9).round()
    }

    /// RPE-based weight suggestion.
    pub fn calc_suggest_weight(one_rm: f64, reps: i32, target_rpe: f64) -> f64 {
        let reps_factor = 1.0 - (reps as f64 - 1.0) * 0.02;
        let rpe_factor = 1.0 - (target_rpe - 10.0) * 0.025;
        (one_rm * reps_factor * rpe_factor).max(0.0)
    }

    // --- Progress & Trends ---

    /// Simple progress percentage.
    pub fn calc_progress_percent(current: f64, previous: f64) -> f64 {
        if previous == 0.0 {
            0.0
        } else {
            ((current - previous) / previous) * 100.0
        }
    }
}