use std::cmp::{max};

use anyhow::Result;
use crate::model::{SchedulerState, Grade, Schedule};


pub fn schedule(previous: Option<&SchedulerState>, grade: Grade, reviewed_at: i64) -> Result<Schedule> {
    let mut next: Schedule = if let Some(state) = previous {
        Schedule::from(state)
    } else {
        Schedule::default()
    };

    match grade {
        Grade::Again => {
            next.interval_seconds = 600;
            next.repetitions = 0;
            next.ease = 1.3_f64.max(next.ease - 0.2);
        }
        Grade::Hard => {
            next.interval_seconds = if next.repetitions == 0 { 43_200 } 
                else { max(86_400, (next.interval_seconds as f64 * 1.2).round() as i64) };
            next.repetitions += 1;
            next.ease = 1.3_f64.max(next.ease - 0.15);
        }
        Grade::Good => {
            next.interval_seconds = if next.repetitions == 0 { 86_400 } 
                else { max(86_400, (next.interval_seconds as f64 * next.ease).round() as i64) };
            next.repetitions += 1;
        }
        Grade::Easy => {
            next.interval_seconds = if next.repetitions == 0 { 345_600 } 
                else { max(345_600, (next.interval_seconds as f64 * next.ease * 1.3).round() as i64) };
            next.repetitions += 1;
            next.ease = 1.3_f64.max(next.ease + 0.15);
        }
    }
    next.ease = next.ease.max(1.3_f64);
    next.due_at = reviewed_at + next.interval_seconds;

    Ok(next)
}