use chrono::{DateTime, FixedOffset, Timelike};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct PreciseTimestamp {
    ts: f64,
}

impl PreciseTimestamp {
    #[inline]
    pub fn now() -> Self {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        PreciseTimestamp { ts: now.as_secs() as f64 + now.subsec_nanos() as f64 / 1e9 }
    }

    #[inline]
    pub fn from_datetime(tsd: DateTime<FixedOffset>) -> Self {
        PreciseTimestamp { ts: tsd.timestamp() as f64 + tsd.naive_utc().nanosecond() as f64 / 1e9 }
    }

    #[inline]
    pub fn as_f64(&self) -> f64 {
        self.ts
    }
}
