use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_f64() -> f64 {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    return now.as_secs() as f64 + now.subsec_nanos() as f64 / 1e9;
}
