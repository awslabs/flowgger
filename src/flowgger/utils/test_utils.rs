#[cfg(test)]
pub mod rfc_test_utils {
    use crate::flowgger::utils;
    use chrono::{Datelike, NaiveDateTime, Utc, DateTime};

    /// Converts a partial date to a timestamp in ms assuming the year is the current one
    #[inline]
    pub fn ts_from_partial_date_time(month: u32, day: u32, hour: u32, min: u32, sec: u32) -> f64 {
        ts_from_date_time(Utc::now().year(), month, day, hour, min, sec, 0)
    }

    /// Converts a full date to a timestamp in ms
    fn new_date_time(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        min: u32,
        sec: u32,
        msec: u32,
    )  -> NaiveDateTime {
        // Compute the timestamp we expect
        let d = chrono::NaiveDate::from_ymd(year, month, day);
        let t = chrono::NaiveTime::from_hms_milli(hour, min, sec, msec);
        NaiveDateTime::new(d, t)
    }

    /// Converts a full date to a timestamp in ms
    pub fn ts_from_date_time(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        min: u32,
        sec: u32,
        msec: u32,
    ) -> f64 {
        let dt = new_date_time(year, month, day, hour, min, sec, msec);
        utils::PreciseTimestamp::from_naive_datetime(dt).as_f64()
    }

    /// Converts a full date to a timestamp in ms
    pub fn utc_from_date_time(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        min: u32,
        sec: u32,
        msec: u32,
    ) -> DateTime<Utc> {
        let dt = new_date_time(year, month, day, hour, min, sec, msec);
        DateTime::from_utc(dt, Utc)
    }
//

}
