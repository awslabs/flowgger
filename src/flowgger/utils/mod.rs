use chrono::{DateTime, FixedOffset, Timelike};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_f64() -> f64 {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    now.as_secs() as f64 + now.subsec_nanos() as f64 / 1e9
}

pub fn datetime_f64(tsd: DateTime<FixedOffset>) -> f64 {
    tsd.timestamp() as f64 + tsd.naive_utc().nanosecond() as f64 / 1e9
}
