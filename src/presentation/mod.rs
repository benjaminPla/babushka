pub mod courses;
pub mod students;
pub mod teachers;

use chrono::{DateTime, FixedOffset, Utc};

fn art() -> FixedOffset {
    FixedOffset::west_opt(3 * 3600).unwrap()
}

pub fn fmt_dt(dt: DateTime<Utc>) -> String {
    dt.with_timezone(&art()).format("%d-%m-%Y %H:%M").to_string()
}
