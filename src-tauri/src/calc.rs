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
        let rpe_factor = 1.0 - (10.0 - target_rpe) * 0.025;
        (one_rm * reps_factor * rpe_factor).max(0.0)
    }

    /// Generates a warm-up ramp leading up to a target working weight.
    /// Returns (weight, reps) pairs, typically 3-4 ramp sets before the work set.
    pub fn calc_warmup_sets(working_weight: f64, bar_weight: f64) -> Vec<(f64, i32)> {
        if working_weight <= bar_weight {
            return vec![(bar_weight, 5)];
        }

        let percentages = [(0.4, 8), (0.6, 5), (0.8, 3), (0.9, 1)];
        let mut sets = vec![(bar_weight, 5)];

        for (pct, reps) in percentages {
            let weight = (working_weight * pct).round();
            if weight > bar_weight {
                sets.push((weight, reps));
            }
        }

        sets
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

#[cfg(test)]
mod tests {
    use super::*;

    // --- 1RM Calculations ---

    #[test]
    fn epley_1rm_at_one_rep_equals_weight() {
        assert_eq!(Calc::calc_one_rm_epley(100.0, 1), 100.0);
    }

    #[test]
    fn epley_1rm_increases_with_reps() {
        let result = Calc::calc_one_rm_epley(100.0, 10);
        assert!((result - 133.333).abs() < 0.01);
    }

    #[test]
    fn brzycki_1rm_at_one_rep_equals_weight() {
        assert_eq!(Calc::calc_one_rm_brzycki(100.0, 1), 100.0);
    }

    #[test]
    fn brzycki_1rm_increases_with_reps() {
        let result = Calc::calc_one_rm_brzycki(100.0, 5);
        assert!((result - 112.5).abs() < 0.01);
    }

    // --- Volume Calculations ---

    #[test]
    fn volume_multiplies_weight_and_reps() {
        assert_eq!(Calc::calc_volume(50.0, 10), 500.0);
    }

    #[test]
    fn total_volume_sums_all_sets() {
        let sets = vec![
            Set { weight: 50.0, reps: 10, rpe: None },
            Set { weight: 60.0, reps: 8, rpe: None },
        ];
        assert_eq!(Calc::calc_total_volume(&sets), 500.0 + 480.0);
    }

    #[test]
    fn weekly_volume_averages_correctly() {
        let volumes = vec![100.0, 200.0, 300.0];
        assert_eq!(Calc::calc_weekly_volume(&volumes), 200.0);
    }

    #[test]
    fn weekly_volume_empty_returns_zero() {
        let volumes: Vec<f64> = vec![];
        assert_eq!(Calc::calc_weekly_volume(&volumes), 0.0);
    }

    // --- Best Set & Records ---

    #[test]
    fn best_set_picks_highest_volume() {
        let sets = vec![
            Set { weight: 50.0, reps: 10, rpe: None }, // volume 500
            Set { weight: 100.0, reps: 8, rpe: None }, // volume 800
            Set { weight: 40.0, reps: 5, rpe: None },  // volume 200
        ];
        let best = Calc::calc_best_set(&sets).unwrap();
        assert_eq!(best.weight, 100.0);
        assert_eq!(best.reps, 8);
    }

    #[test]
    fn best_set_empty_returns_none() {
        let sets: Vec<Set> = vec![];
        assert!(Calc::calc_best_set(&sets).is_none());
    }

    // --- Training & Planning ---

    #[test]
    fn training_max_is_ninety_percent_rounded() {
        assert_eq!(Calc::calc_training_max(150.0), 135.0);
    }

    #[test]
    fn warmup_sets_starts_with_bar() {
        let sets = Calc::calc_warmup_sets(100.0, 20.0);
        assert_eq!(sets[0], (20.0, 5));
    }

    #[test]
    fn warmup_sets_below_bar_weight_returns_just_bar() {
        let sets = Calc::calc_warmup_sets(15.0, 20.0);
        assert_eq!(sets, vec![(20.0, 5)]);
    }

    #[test]
    fn warmup_sets_ramps_upward() {
        let sets = Calc::calc_warmup_sets(100.0, 20.0);
        for i in 1..sets.len() {
            assert!(sets[i].0 >= sets[i - 1].0);
        }
    }

        #[test]
    fn suggest_weight_at_target_rpe_ten_returns_full_weight() {
        // reps=1 and rpe=10 should apply no reduction at all
        let result = Calc::calc_suggest_weight(100.0, 1, 10.0);
        assert!((result - 100.0).abs() < 0.01);
    }

    #[test]
    fn suggest_weight_decreases_with_more_reps() {
        let fewer_reps = Calc::calc_suggest_weight(100.0, 1, 8.0);
        let more_reps = Calc::calc_suggest_weight(100.0, 5, 8.0);
        assert!(more_reps < fewer_reps, "higher rep targets should suggest a lighter weight");
    }

            #[test]
    fn suggest_weight_decreases_with_lower_target_rpe() {
        let higher_rpe = Calc::calc_suggest_weight(100.0, 5, 9.0);
        let lower_rpe = Calc::calc_suggest_weight(100.0, 5, 6.0);
        assert!(lower_rpe < higher_rpe, "a lower target RPE should suggest a lighter weight, matching real RPE convention");
    }

    #[test]
    fn suggest_weight_never_goes_negative() {
        // Extreme inputs shouldn't produce a nonsensical negative weight
        let result = Calc::calc_suggest_weight(100.0, 50, 1.0);
        assert!(result >= 0.0);
    }

    // --- Progress & Trends ---

    #[test]
    fn progress_percent_calculates_increase() {
        let result = Calc::calc_progress_percent(110.0, 100.0);
        assert!((result - 10.0).abs() < 0.01);
    }

    #[test]
    fn progress_percent_previous_zero_returns_zero() {
        assert_eq!(Calc::calc_progress_percent(50.0, 0.0), 0.0);
    }
}