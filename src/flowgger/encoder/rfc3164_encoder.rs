use super::{Encoder, config_get_prepend_ts, build_prepend_ts};
use crate::flowgger::config::Config;
use crate::flowgger::record::Record;
use chrono::NaiveDateTime;

#[derive(Clone)]
pub struct RFC3164Encoder {
    header_time_format: Option<String>,
}

impl RFC3164Encoder {
    pub fn new(config: &Config) -> RFC3164Encoder {
        let header_time_format = config_get_prepend_ts(config);

        RFC3164Encoder { header_time_format }
    }
}

impl Encoder for RFC3164Encoder {
    /// Implementation of the RF3164 encoder. Encode a record object into a string
    ///
    /// # Arguments
    /// * `record` - Record object containing the log info to encode
    ///
    /// # Returns
    /// * Array of chars containing the encoded object as a string
    ///
    fn encode(&self, record: Record) -> Result<Vec<u8>, &'static str> {
        let mut res = String::new();

        // First, if specified, prepend a header
        if self.header_time_format.is_some() {
            res.push_str(&build_prepend_ts(self.header_time_format.as_ref().unwrap()));
        }

        // If a priority is specified, add it
        if record.facility.is_some() && record.severity.is_some() {
            let npri: u8 =
                ((record.facility.unwrap() << 3) & 0xF8) + (record.severity.unwrap() & 0x7);
            res.push_str(&format!("<{}>", npri));
        }

        // Add timestamp + space
        let dt = NaiveDateTime::from_timestamp(record.ts as i64, 0);
        let dt_str = dt.format("%b %e %H:%M:%S ").to_string();
        res.push_str(&dt_str);

        // Add hostname + space
        res.push_str(&record.hostname);
        res.push(' ');

        // Add appname/procid/msgid if specified
        if let Some(appname) = record.appname {
            res.push_str(&appname);
        }
        if let Some(procid) = record.procid {
            res.push_str(&format!("[{}]:", procid));
            res.push(' ');
        }
        if let Some(msgid) = record.msgid {
            res.push_str(&msgid);
            res.push(' ');
        }

        // Encode structured data is present, although not part of rfc3164
        if let Some(sd) = record.sd {
            res.push_str(&sd.to_string());
            res.push(' ');
        }

        if let Some(msg) = record.msg {
            res.push_str(&msg);
        }

        Ok(res.into_bytes())
    }
}

#[cfg(test)]
use crate::flowgger::record::{SDValue, StructuredData};
#[cfg(test)]
use crate::flowgger::utils::test_utils::rfc_test_utils::ts_from_partial_date_time;
#[cfg(test)]
use chrono::Utc;

#[test]
fn test_rfc3164_encode() {
    let expected_msg = r#"Aug  6 11:15:24 testhostname appname 69 42 [origin@123 software="te\st sc\"ript" swVersion="0.0.1"] test message"#;
    let cfg = Config::from_string("[input]\n[input.ltsv_schema]\nformat = \"rfc3164\"\n").unwrap();
    let ts = ts_from_partial_date_time(8, 6, 11, 15, 24);

    let record = Record {
        ts,
        hostname: "testhostname".to_string(),
        facility: None,
        severity: None,
        appname: None,
        procid: None,
        msgid: None,
        msg: Some(r#"appname 69 42 [origin@123 software="te\st sc\"ript" swVersion="0.0.1"] test message"#.to_string()),
        full_msg: Some(expected_msg.to_string()),
        sd: None,
    };

    let encoder = RFC3164Encoder::new(&cfg);
    let res = encoder.encode(record).unwrap();
    assert_eq!(String::from_utf8_lossy(&res), expected_msg);
}

#[test]
fn test_rfc3164_withpri_encode() {
    let expected_msg = r#"<23>Aug  6 11:15:24 testhostname appname 69 42 [origin@123 software="te\st sc\"ript" swVersion="0.0.1"] test message"#;
    let cfg = Config::from_string("[input]\n[input.ltsv_schema]\nformat = \"rfc3164\"\n").unwrap();
    let ts = ts_from_partial_date_time(8, 6, 11, 15, 24);

    let record = Record {
        ts,
        hostname: "testhostname".to_string(),
        facility: Some(2),
        severity: Some(7),
        appname: None,
        procid: None,
        msgid: None,
        msg: Some(r#"appname 69 42 [origin@123 software="te\st sc\"ript" swVersion="0.0.1"] test message"#.to_string()),
        full_msg: Some(expected_msg.to_string()),
        sd: None,
    };

    let encoder = RFC3164Encoder::new(&cfg);
    let res = encoder.encode(record).unwrap();
    assert_eq!(String::from_utf8_lossy(&res), expected_msg);
}

#[test]
fn test_rfc3164_encode_with_prepend() {
    let cfg = Config::from_string("[output]\nformat = \"rfc3164\"\nsyslog_prepend_timestamp=\"[%Y-%m-%dT%H:%MZ]\"").unwrap();
    let ts = ts_from_partial_date_time(8, 6, 11, 15, 24);
    let dt = Utc::now();
    let dt_str = dt.format("[%Y-%m-%dT%H:%MZ]").to_string();
    let expected_msg = format!(r#"{}Aug  6 11:15:24 testhostname appname 69 42 [origin@123 software="te\st sc\"ript" swVersion="0.0.1"] test message"#, dt_str);


    let record = Record {
        ts,
        hostname: "testhostname".to_string(),
        facility: None,
        severity: None,
        appname: None,
        procid: None,
        msgid: None,
        msg: Some(r#"appname 69 42 [origin@123 software="te\st sc\"ript" swVersion="0.0.1"] test message"#.to_string()),
        full_msg: Some(expected_msg.to_string()),
        sd: None,
    };

    let encoder = RFC3164Encoder::new(&cfg);
    let res = encoder.encode(record).unwrap();
    assert_eq!(String::from_utf8_lossy(&res), expected_msg);
}

#[test]
#[should_panic(expected = "output.syslog_prepend_timestamp should be a string")]
fn test_rfc3164_invalid_prepend() {
    let cfg = Config::from_string("[output]\nformat = \"rfc3164\"\nsyslog_prepend_timestamp=123").unwrap();
    let _ = RFC3164Encoder::new(&cfg);
}

#[test]
fn test_rfc3164_full_encode() {
    let expected_msg = r#"<23>Aug  6 11:15:24 testhostname appname[69]: 42 [someid a="b" c="123456"] some test message"#;
    let cfg = Config::from_string("[input]\n[input.ltsv_schema]\nformat = \"rfc3164\"\n").unwrap();
    let ts = ts_from_partial_date_time(8, 6, 11, 15, 24);

    let record = Record {
        ts,
        hostname: "testhostname".to_string(),
        facility: Some(2),
        severity: Some(7),
        appname: Some("appname".to_string()),
        procid: Some("69".to_string()),
        msgid: Some("42".to_string()),
        msg: Some(r#"some test message"#.to_string()),
        full_msg: Some(expected_msg.to_string()),
        sd: Some(StructuredData {
            sd_id: Some("someid".to_string()),
            pairs: vec![
                ("a".to_string(), SDValue::String("b".to_string())),
                ("c".to_string(), SDValue::U64(123456)),
            ],
        }),
    };

    let encoder = RFC3164Encoder::new(&cfg);
    let res = encoder.encode(record).unwrap();
    assert_eq!(String::from_utf8_lossy(&res), expected_msg);
}
