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
}#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epley_at_one_rep_returns_weight_unchanged() {
        assert_eq!(Calc::calc_one_rm_epley(100.0, 1), 100.0);
    }

    #[test]
    fn epley_scales_up_with_reps() {
        let result = Calc::calc_one_rm_epley(100.0, 10);
        assert!((result - 133.333).abs() < 0.01);
    }

    #[test]
    fn brzycki_at_one_rep_returns_weight_unchanged() {
        assert_eq!(Calc::calc_one_rm_brzycki(100.0, 1), 100.0);
    }

    #[test]
    fn calc_volume_multiplies_weight_by_reps() {
        assert_eq!(Calc::calc_volume(50.0, 10), 500.0);
    }

    #[test]
    fn calc_total_volume_sums_across_sets() {
        let sets = vec![
            Set { weight: 100.0, reps: 5, rpe: None },
            Set { weight: 100.0, reps: 5, rpe: None },
        ];
        assert_eq!(Calc::calc_total_volume(&sets), 1000.0);
    }

    #[test]
    fn calc_best_set_picks_highest_volume() {
        let sets = vec![
            Set { weight: 100.0, reps: 5, rpe: None },  // 500
            Set { weight: 120.0, reps: 5, rpe: None },  // 600 - best
            Set { weight: 80.0, reps: 8, rpe: None },   // 640 - actually best
        ];
        let best = Calc::calc_best_set(&sets).unwrap();
        assert_eq!(best.weight, 80.0);
        assert_eq!(best.reps, 8);
    }

    #[test]
    fn calc_best_set_empty_returns_none() {
        let sets: Vec<Set> = vec![];
        assert!(Calc::calc_best_set(&sets).is_none());
    }

    #[test]
    fn calc_training_max_is_ninety_percent_rounded() {
        assert_eq!(Calc::calc_training_max(100.0), 90.0);
    }

    #[test]
    fn calc_progress_percent_zero_previous_returns_zero() {
        assert_eq!(Calc::calc_progress_percent(100.0, 0.0), 0.0);
    }

    #[test]
    fn calc_progress_percent_computes_correctly() {
        let result = Calc::calc_progress_percent(110.0, 100.0);
        assert!((result - 10.0).abs() < 0.001);
    }

}