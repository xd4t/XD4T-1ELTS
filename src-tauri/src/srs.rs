use chrono::{Duration, NaiveDate};

#[derive(Debug, Clone, PartialEq)]
pub struct Sm2Outcome {
    pub ease: f64,
    pub interval_days: i64,
    pub reps: i64,
    pub due_date: String,
}

/// Applies the SM-2 spaced-repetition update for a vocab card.
/// `quality`: 0 = Forgot (10m / today), 1 = Hard (1d), 2 = Good (3d+), 3 = Easy (7d+).
pub fn apply_sm2(
    ease: f64,
    interval_days: i64,
    reps: i64,
    quality: u8,
    today: NaiveDate,
) -> Sm2Outcome {
    let clamped_ease = ease.max(1.3);

    let (new_ease, new_interval, new_reps) = match quality {
        0 => ((clamped_ease - 0.20).max(1.3), 0, 0),
        1 => ((clamped_ease - 0.15).max(1.3), 1, reps.max(1)),
        2 => {
            let next_interval = if reps == 0 {
                3
            } else {
                ((interval_days.max(1) as f64) * clamped_ease).round() as i64
            };
            (clamped_ease, next_interval.max(1), reps + 1)
        }
        _ => {
            let next_interval = if reps == 0 {
                7
            } else {
                ((interval_days.max(1) as f64) * clamped_ease * 1.3).round() as i64
            };
            ((clamped_ease + 0.10).min(3.0), next_interval.max(1), reps + 1)
        }
    };

    let due = today + Duration::days(new_interval);
    Sm2Outcome {
        ease: (new_ease * 100.0).round() / 100.0,
        interval_days: new_interval,
        reps: new_reps,
        due_date: due.format("%Y-%m-%d").to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sm2_forgot_resets_reps_and_keeps_today() {
        let today = NaiveDate::from_ymd_opt(2026, 11, 25).unwrap();
        let out = apply_sm2(2.5, 7, 3, 0, today);
        assert_eq!(out.reps, 0);
        assert_eq!(out.interval_days, 0);
        assert_eq!(out.due_date, "2026-11-25");
        assert!((out.ease - 2.3).abs() < 0.01);
    }

    #[test]
    fn sm2_good_and_easy_progress_intervals() {
        let today = NaiveDate::from_ymd_opt(2026, 11, 25).unwrap();
        let first_good = apply_sm2(2.5, 0, 0, 2, today);
        assert_eq!(first_good.interval_days, 3);
        assert_eq!(first_good.reps, 1);
        assert_eq!(first_good.due_date, "2026-11-28");

        let second_good = apply_sm2(first_good.ease, first_good.interval_days, first_good.reps, 2, today);
        assert_eq!(second_good.interval_days, 8);
        assert_eq!(second_good.reps, 2);

        let first_easy = apply_sm2(2.5, 0, 0, 3, today);
        assert_eq!(first_easy.interval_days, 7);
        assert_eq!(first_easy.reps, 1);
        assert!((first_easy.ease - 2.6).abs() < 0.01);
    }

    #[test]
    fn sm2_ease_never_drops_below_floor() {
        let today = NaiveDate::from_ymd_opt(2026, 11, 25).unwrap();
        let out = apply_sm2(1.3, 3, 2, 0, today);
        assert_eq!(out.ease, 1.3);
    }
}
