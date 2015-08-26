extern crate chrono;

use flowgger::Decoder;
use flowgger::config::Config;
use flowgger::record::{Record, StructuredData, SDValue};
use self::chrono::DateTime;

#[derive(Clone)]
pub struct LTSVDecoder;

impl LTSVDecoder {
    pub fn new(config: &Config) -> LTSVDecoder {
        let _ = config;
        LTSVDecoder
    }
}

impl Decoder for LTSVDecoder {
    fn decode(&self, line: &str) -> Result<Record, &'static str> {
        let mut sd = StructuredData::new(None);
        let mut ts = None;
        let mut hostname = None;
        let mut msg = None;
        let mut severity = None;

        for part in line.split('\t') {
            let mut pair = part.splitn(2, ':');
            let name = try!(pair.next().ok_or("Missing name in an LTSV record"));
            let value = try!(pair.next().ok_or("Missing value in an LTSV record"));
            match name {
                "time" => {
                    let ts_s = if value.starts_with('[') && value.ends_with(']') {
                        &value[1..(value.len() -1)]
                    } else {
                        value
                    };
                    ts = Some(try!(parse_ts(ts_s)));
                },
                "host" => hostname = Some(value.to_owned()),
                "message" => msg = Some(value.to_owned()),
                "level" => {
                    let level: u8 = try!(value.parse().or(Err("Invalid severity level")));
                    if level > 7 {
                        return Err("Severity level should be <= 7")
                    }
                    severity = Some(level);
                },
                name @ _ => sd.pairs.push((format!("_{}", name), SDValue::String(value.to_owned())))
            };
        }
        let record = Record {
            ts: try!(ts.ok_or("Missing timestamp")),
            hostname: try!(hostname.ok_or("Missing hostname")),
            facility: None,
            severity: severity,
            appname: None,
            procid: None,
            msgid: None,
            sd: if sd.pairs.is_empty() { None } else { Some(sd) },
            msg: msg,
            full_msg: None
        };
        Ok(record)
    }
}

fn rfc3339_to_unix(rfc3339: &str) -> Result<i64, &'static str> {
    match DateTime::parse_from_rfc3339(rfc3339) {
        Ok(date) => Ok(date.timestamp()),
        Err(_) => Err("Unable to parse the date")
    }
}

fn english_time_to_unix(et: &str) -> Result<i64, &'static str> {
    match DateTime::parse_from_str(et, "%e/%b/%Y:%H:%M:%S %z") {
        Ok(date) => Ok(date.timestamp()),
        Err(_) => Err("Unable to parse the date")
    }
}

fn parse_ts(line: &str) -> Result<i64, &'static str> {
    rfc3339_to_unix(line).or(english_time_to_unix(line))
}

#[test]
fn test_ltsv() {
    let msg = "time:[2015-08-05T15:53:45.637824Z]\thost:testhostname\tname1:value1\tname 2: value 2\tn3:v3";
    let res = LTSVDecoder.decode(msg).unwrap();
    assert!(res.ts == 1438790025);

    let msg = "time:[10/Oct/2000:13:55:36 -0700]\tlevel:3\thost:testhostname\tname1:value1\tname 2: value 2\tn3:v3\tmessage:this is a test";
    let res = LTSVDecoder.decode(msg).unwrap();
    assert!(res.ts == 971211336);
    assert!(res.severity.unwrap() == 3);

    assert!(res.hostname == "testhostname");
    assert!(res.msg.unwrap() == "this is a test");
    let sd = res.sd.unwrap();
    let pairs = sd.pairs;
    assert!(pairs.iter().cloned().any(|(k, v)|
        if let SDValue::String(v) = v { k == "_name1" && v == "value1" } else { false }
    ));
    assert!(pairs.iter().cloned().any(|(k, v)|
        if let SDValue::String(v) = v { k == "_name 2" && v == " value 2" } else { false }
    ));
    assert!(pairs.iter().cloned().any(|(k, v)|
        if let SDValue::String(v) = v { k == "_n3" && v == "v3" } else { false }
    ));
}
