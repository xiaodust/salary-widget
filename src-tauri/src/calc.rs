use chrono::{Datelike, Local, NaiveTime, Timelike};
use serde::Serialize;

use crate::config::Config;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkStatus {
    BeforeWork,
    Working,
    Lunch,
    AfterWork,
    RestDay,
}

#[derive(Debug, Clone, Serialize)]
pub struct Snapshot {
    pub now: String,
    pub status: WorkStatus,
    pub earned_today: f64,
    pub work_seconds: u64,
    pub remaining_seconds: Option<u64>,
    pub rate_per_second: f64,
    pub progress: f64,
    pub day_total_seconds: u64,
    pub lunch_seconds: u64,
}

fn parse_minutes(s: &str) -> Option<u32> {
    NaiveTime::parse_from_str(s, "%H:%M")
        .ok()
        .map(|t| t.num_seconds_from_midnight() / 60)
}

pub fn snapshot(cfg: &Config) -> Snapshot {
    let now = Local::now();
    let date = now.date_naive();
    let hour = now.hour() as u32;
    let minute = now.minute() as u32;
    let second = now.second() as u32;
    let today_min = hour * 60 + minute;
    let secs_today = hour as u64 * 3600 + minute as u64 * 60 + second as u64;

    let weekday = date.weekday().number_from_monday(); // Mon=1 ... Sun=7
    let is_workday = cfg.workdays.contains(&weekday);

    let rate_ps = cfg.effective_hourly() / 3600.0;

    let start = parse_minutes(&cfg.work_start).unwrap_or(9 * 60);
    let end = parse_minutes(&cfg.work_end).unwrap_or(18 * 60);
    let lunch_s = if cfg.lunch_enabled {
        parse_minutes(&cfg.lunch_start).unwrap_or(12 * 60)
    } else {
        end
    };
    let lunch_e = if cfg.lunch_enabled {
        parse_minutes(&cfg.lunch_end).unwrap_or(13 * 60)
    } else {
        end
    };
    let lunch_seconds = lunch_e.saturating_sub(lunch_s) as u64 * 60;
    let day_total_seconds =
        (end.saturating_sub(start) as u64 * 60).saturating_sub(lunch_seconds);

    let now_str = || format!("{hour:02}:{minute:02}:{second:02}");

    let empty = |status: WorkStatus| Snapshot {
        now: now_str(),
        status,
        earned_today: 0.0,
        work_seconds: 0,
        remaining_seconds: None,
        rate_per_second: rate_ps,
        progress: 0.0,
        day_total_seconds,
        lunch_seconds,
    };

    if !is_workday {
        return empty(WorkStatus::RestDay);
    }
    if today_min < start {
        return empty(WorkStatus::BeforeWork);
    }

    if cfg.lunch_enabled && today_min >= lunch_s && today_min < lunch_e {
        let work_seconds = lunch_s.saturating_sub(start) as u64 * 60;
        return Snapshot {
            now: now_str(),
            status: WorkStatus::Lunch,
            earned_today: rate_ps * work_seconds as f64,
            work_seconds,
            remaining_seconds: Some((end as u64 * 60).saturating_sub(secs_today)),
            rate_per_second: rate_ps,
            progress: work_seconds as f64 / day_total_seconds.max(1) as f64,
            day_total_seconds,
            lunch_seconds,
        };
    }

    if today_min < end {
        let lunch_elapsed = if cfg.lunch_enabled && today_min >= lunch_e {
            lunch_seconds
        } else if cfg.lunch_enabled && today_min > lunch_s {
            today_min.saturating_sub(lunch_s) as u64 * 60
        } else {
            0
        };
        let work_seconds =
            secs_today.saturating_sub(start as u64 * 60).saturating_sub(lunch_elapsed);
        Snapshot {
            now: now_str(),
            status: WorkStatus::Working,
            earned_today: rate_ps * work_seconds as f64,
            work_seconds,
            remaining_seconds: Some((end as u64 * 60).saturating_sub(secs_today)),
            rate_per_second: rate_ps,
            progress: (work_seconds as f64 / day_total_seconds.max(1) as f64).clamp(0.0, 1.0),
            day_total_seconds,
            lunch_seconds,
        }
    } else {
        let work_seconds = (end.saturating_sub(start) as u64 * 60).saturating_sub(lunch_seconds);
        Snapshot {
            now: now_str(),
            status: WorkStatus::AfterWork,
            earned_today: rate_ps * work_seconds as f64,
            work_seconds,
            remaining_seconds: None,
            rate_per_second: rate_ps,
            progress: 1.0,
            day_total_seconds,
            lunch_seconds,
        }
    }
}
