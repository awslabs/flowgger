use super::Decoder;
use crate::flowgger::config::Config;
use crate::flowgger::record::Record;
use crate::flowgger::utils;
use chrono::{Datelike, NaiveDateTime, Utc};

#[derive(Clone)]
pub struct RFC3164Decoder {}

impl RFC3164Decoder {
    pub fn new(_config: &Config) -> RFC3164Decoder {
        RFC3164Decoder {}
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
        let (pri, _msg) = parse_strip_pri(line)?;

        // The event may have several consecutive spaces as separator
        let tokens_vec = _msg.split_whitespace().collect::<Vec<&str>>();

        // If we have less than 4 tokens, the input can't be valid
        if tokens_vec.len() > 3 {
            // Date is made of the first 3 space separated tokens, rebuild it
            let _date_str = tokens_vec[0..3].join(" ");
            let _hostname = tokens_vec[3];

            // All that remains is the message that may contain several spaces, so rebuild it
            let _message = tokens_vec[4..].join(" ");

            let ts = parse_ts(&_date_str)?;
            let record = Record {
                ts,
                hostname: _hostname.to_owned(),
                facility: pri.facility,
                severity: pri.severity,
                appname: None,
                procid: None,
                msgid: None,
                msg: Some(_message.to_owned()),
                full_msg: Some(line.to_owned()),
                sd: None,
            };
            Ok(record)
        } else {
            Err("Malformed RFC3164 event: Invalid timestamp or hostname")
        }
    }
}

struct Pri {
    facility: Option<u8>,
    severity: Option<u8>,
}

fn parse_strip_pri(event: &str) -> Result<(Pri, &str), &'static str> {
    if event.starts_with('<') {
        let pri_end_index = event
            .find('>')
            .ok_or("Malformed RFC3164 event: Invalid priority")?;
        let (pri, msg) = event.split_at(pri_end_index + 1);
        let npri: u8 = pri
            .trim_start_matches('<')
            .trim_end_matches('>')
            .parse()
            .or(Err("Invalid priority"))?;
        Ok((
            Pri {
                facility: Some(npri >> 3),
                severity: Some(npri & 7),
            },
            msg,
        ))
    } else {
        Ok((
            Pri {
                facility: None,
                severity: None,
            },
            event,
        ))
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
fn test_rfc3164_decode() {
    let msg = r#"Aug  6 11:15:24 testhostname appname 69 42 [origin@123 software="te\st sc\"ript" swVersion="0.0.1"] test message"#;
    let cfg = Config::from_string("[input]\n[input.ltsv_schema]\nformat = \"rfc3164\"\n").unwrap();
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
fn test_rfc3164_decode_with_pri() {
    let msg = r#"<13>Aug  6 11:15:24 testhostname appname 69 42 [origin@123 software="te\st sc\"ript" swVersion="0.0.1"] test message"#;
    let cfg = Config::from_string("[input]\n[input.ltsv_schema]\nformat = \"rfc3164\"\n").unwrap();
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
