use super::{Encoder, config_get_prepend_ts, build_prepend_ts};
use crate::flowgger::config::Config;
use crate::flowgger::record::Record;

#[derive(Clone)]
pub struct PassthroughEncoder {
    header_time_format: Option<String>,
}

impl PassthroughEncoder {
    pub fn new(config: &Config) -> PassthroughEncoder {
        let header_time_format = config_get_prepend_ts(config);
        PassthroughEncoder { header_time_format }
    }
}

impl Encoder for PassthroughEncoder {
    /// Implementation of a passthrough encoder.
    /// Just pass the full raw messages from input without rebuilding them.
    /// This allows passing several different formats, i.e. rfc3164 can accept different formats.
    /// The actual output format is therefore the format set set as input.
    fn encode(&self, record: Record) -> Result<Vec<u8>, &'static str> {
        let mut res = String::new();

        // Only push messages where the raw message is specified
        if let Some(msg) = record.full_msg {
            // First, if specified, prepend a header
            if self.header_time_format.is_some() {
                res.push_str(&build_prepend_ts(self.header_time_format.as_ref().unwrap()));
            }

            // Pysh the message
            res.push_str(&msg);
            Ok(res.into_bytes())
        }
        else {
            Err("Cannot output empty raw message")
        }
    }
}

#[cfg(test)]
use chrono::Utc;

#[test]
fn test_passthrough_encode() {
    let expected_msg = r#"Aug  6 11:15:24 testhostname appname 69 42 [origin@123 software="te\st sc\"ript" swVersion="0.0.1"] test message"#;
    let cfg = Config::from_string("[input]\n[input.ltsv_schema]\nformat = \"passthrough\"\n").unwrap();

    let record = Record {
        ts:1.2,
        hostname: "abcd".to_string(),
        facility: None,
        severity: None,
        appname: None,
        procid: None,
        msgid: None,
        msg: Some(r#"test message"#.to_string()),
        full_msg: Some(expected_msg.to_string()),
        sd: None,
    };

    let encoder = PassthroughEncoder::new(&cfg);
    let res = encoder.encode(record).unwrap();
    assert_eq!(String::from_utf8_lossy(&res), expected_msg);
}

#[test]
fn test_passthrough_encode_with_prepend() {
    let cfg = Config::from_string("[output]\nformat = \"passthrough\"\nsyslog_prepend_timestamp=\"[%Y-%m-%dT%H:%MZ]\"").unwrap();
    let dt = Utc::now();
    let dt_str = dt.format("[%Y-%m-%dT%H:%MZ]").to_string();
    let input_msg = format!(r#"{}Aug  6 11:15:24 testhostname appname 69 42 [origin@123 software="te\st sc\"ript" swVersion="0.0.1"] test message"#, dt_str);
    let expected_msg = format!(r#"{}{}"#, dt_str, input_msg);

    let record = Record {
        ts: 1.2,
        hostname: "abcd".to_string(),
        facility: None,
        severity: None,
        appname: None,
        procid: None,
        msgid: None,
        msg: Some(r#"test message"#.to_string()),
        full_msg: Some(input_msg.to_string()),
        sd: None,
    };

    let encoder = PassthroughEncoder::new(&cfg);
    let res = encoder.encode(record).unwrap();
    assert_eq!(String::from_utf8_lossy(&res), expected_msg);
}

#[test]
#[should_panic(expected = "output.syslog_prepend_timestamp should be a string")]
fn test_passthrough_encode_invalid_prepend() {
    let cfg = Config::from_string("[output]\nformat = \"passthrough\"\nsyslog_prepend_timestamp=123").unwrap();
    let _ = PassthroughEncoder::new(&cfg);
}

#[test]
#[should_panic(expected = "Cannot output empty raw message")]
fn test_passthrough_encode_no_msg() {
    let cfg = Config::from_string("[output]\nformat = \"passthrough\"\nsyslog_prepend_timestamp=\"[%Y-%m-%dT%H:%MZ]\"").unwrap();

    let record = Record {
        ts: 1.2,
        hostname: "abcd".to_string(),
        facility: None,
        severity: None,
        appname: None,
        procid: None,
        msgid: None,
        msg: Some(r#"test message"#.to_string()),
        full_msg: None,
        sd: None,
    };

    let encoder = PassthroughEncoder::new(&cfg);
    let _ = encoder.encode(record).unwrap();
}
