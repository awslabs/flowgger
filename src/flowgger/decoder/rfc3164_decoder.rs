use super::Decoder;
use crate::flowgger::config::Config;
use crate::flowgger::record::Record;
use crate::flowgger::utils;
use regex::Regex;
use chrono::{NaiveDateTime, Datelike, Utc};

#[derive(Clone)]
pub struct RFC3164Decoder {
    rfc_regex:Regex,
}

impl RFC3164Decoder {
    pub fn new(config: &Config) -> RFC3164Decoder {
        let _ = config;
        let re = Regex::new(r"^([A-Z,a-z]{3}[\s]*[0-9]*[\s]*[0-9]{2}:[0-9]{2}:[0-9]{2})[\s]*([\S]*)[\s]*(.*)").unwrap();

        RFC3164Decoder {rfc_regex:re}
    }
}

impl Decoder for RFC3164Decoder {
    /// Implementation of the RF3164 decoder. Decode a string into a record object
    ///
    /// # Arguments
    /// * `line` - String to decode
    ///
    /// # Returns
    /// * Record object containing the log info extracted
    ///
    fn decode(&self, line: &str) -> Result<Record, &'static str> {

        // Get the optional pri part and remove it from the string
        let (pri, msg) =  parse_strip_pri(line)?;

        // Regex must extract 3 groups: date/time, hostname, and message, or the entry is invalid
        // We don't try to extract app name/prod id as there is no standard as to how they are provided in rfc3164
        let caps = self.rfc_regex.captures(msg).ok_or("Malformed RFC3164 event: Invalid format")?;
        if caps.len() == 4 {

            let ts = parse_ts(caps.get(1).unwrap().as_str())?;
            let record = Record {
                ts,
                hostname: caps.get(2).unwrap().as_str().to_owned(),
                facility: pri.facility,
                severity: pri.severity,
                appname: None,
                procid: None,
                msgid: None,
                msg: Some(caps.get(3).unwrap().as_str().to_owned()),
                full_msg: Some(line.to_owned()),
                sd: None,
            };
            Ok(record)
        }
        else {
            Err("Malformed RFC3164 event: Invalid timestamp or hostname")
        }
    }
}

struct Pri {
    facility: Option<u8>,
    severity: Option<u8>,
}

fn parse_strip_pri(event: &str) -> Result<(Pri, &str), &'static str> {
    if event.starts_with("<") {
        let pri_end_index = event.find('>').ok_or("Malformed RFC3164 event: Invalid priority")?;
        let (pri, msg) = event.split_at(pri_end_index+1);
        let npri:u8 = pri.trim_start_matches('<').trim_end_matches('>').
            parse().or(Err("Invalid priority"))?;
        Ok((Pri {
            facility: Some(npri >> 3),
            severity: Some(npri & 7),
        }, msg))
    }
    else {
        Ok((Pri {facility: None, severity: None}, event))
    }
}

fn parse_ts(ts_str: &str) -> Result<f64, &'static str> {
    // Append the year to parse a full ts
    let current_year = Utc::now().year();
    let ts = format!("{} {}", current_year, ts_str);

    match NaiveDateTime::parse_from_str(&ts, "%Y %b %d %H:%M:%S") {
        Ok(date) => Ok(utils::PreciseTimestamp::from_naive_datetime(date).as_f64()),
        Err(_) => Err("Unable to parse the date"),
    }
}

#[cfg(test)]
use crate::flowgger::utils::test_utils::rfc_test_utils::ts_from_partial_date_time;

#[test]
fn test_rfc3164_encode() {
    let msg = r#"Aug  6 11:15:24 testhostname appname 69 42 [origin@123 software="te\st sc\"ript" swVersion="0.0.1"] test message"#;
    let cfg = Config::from_string("[input]\n[input.ltsv_schema]\nformat = \"rfc3164\"\n",).unwrap();
    let expected_ts = ts_from_partial_date_time(8, 6, 11, 15, 24);

    let decoder = RFC3164Decoder::new(&cfg);
    let res = decoder.decode(msg).unwrap();
    assert_eq!(res.facility, None);
    assert_eq!(res.severity, None);
    assert_eq!(res.ts, expected_ts);
    assert_eq!(res.hostname, "testhostname");
    assert_eq!(res.appname, None);
    assert_eq!(res.procid, None);
    assert_eq!(res.msgid, None);
    assert_eq!(res.msg, Some(r#"appname 69 42 [origin@123 software="te\st sc\"ript" swVersion="0.0.1"] test message"#.to_string()));
    assert_eq!(res.full_msg, Some(msg.to_string()));
}

#[test]
fn test_rfc3164_encode_with_pri() {
    let msg = r#"<13>Aug  6 11:15:24 testhostname appname 69 42 [origin@123 software="te\st sc\"ript" swVersion="0.0.1"] test message"#;
    let cfg = Config::from_string("[input]\n[input.ltsv_schema]\nformat = \"rfc3164\"\n",).unwrap();
    let expected_ts = ts_from_partial_date_time(8, 6, 11, 15, 24);

    let decoder = RFC3164Decoder::new(&cfg);
    let res = decoder.decode(msg).unwrap();
    assert_eq!(res.facility, Some(1));
    assert_eq!(res.severity, Some(5));
    assert_eq!(res.ts, expected_ts);
    assert_eq!(res.hostname, "testhostname");
    assert_eq!(res.appname, None);
    assert_eq!(res.procid, None);
    assert_eq!(res.msgid, None);
    assert_eq!(res.msg, Some(r#"appname 69 42 [origin@123 software="te\st sc\"ript" swVersion="0.0.1"] test message"#.to_string()));
    assert_eq!(res.full_msg, Some(msg.to_string()));
}
