extern crate serde;
extern crate serde_json;

use flowgger::Decoder;
use flowgger::config::Config;
use flowgger::record::{Record, StructuredData, SDValue};
use self::serde_json::de;
use self::serde_json::value::Value;

#[derive(Clone)]
pub struct GelfDecoder;

impl GelfDecoder {
    pub fn new(config: &Config) -> GelfDecoder {
        let _ = config;
        GelfDecoder
    }
}

impl Decoder for GelfDecoder {
    fn decode(&self, line: &str) -> Result<Record, &'static str> {
        let mut sd = StructuredData::new(None);
        let mut ts = None;
        let mut hostname = None;
        let mut msg = None;
        let mut full_msg = None;
        let mut severity = None;

        let obj: Value = try!(de::from_str(line).or(Err("Invalid GELF input, unable to parse as a JSON object")));
        let obj = try!(obj.as_object().ok_or("Empty GELF input"));
        for (key, value) in obj {
            match key.as_ref() {
                "timestamp" => {
                    ts = Some(try!(value.as_f64().ok_or("Invalid GELF timestamp")) as i64);
                }
                "host" => {
                    hostname = Some(try!(value.as_string().ok_or("GELF host name must be a string")).to_owned());
                }
                "short_message" => {
                    msg = Some(try!(value.as_string().ok_or("GELF short message must be a string")).to_owned());
                }
                "full_message" => {
                    full_msg = Some(try!(value.as_string().ok_or("GELF full message must be a string")).to_owned());
                }
                "version" => {
                    match try!(value.as_string().ok_or("GELF version must be a string")) {
                        "1.0" | "1.1" => { }
                        _ => return Err("Unsupported GELF version")
                    }
                }
                "level" => {
                    let level = try!(value.as_u64().ok_or("Invalid severity level"));
                    if level > 7 {
                        return Err("Severity level should be <= 7")
                    }
                    severity = Some(level as u8)
                },
                name @ _ => {
                    let sd_value: SDValue = match *value {
                        Value::String(ref value) => SDValue::String(value.to_owned()),
                        Value::Bool(value) => SDValue::Bool(value),
                        Value::F64(value) => SDValue::F64(value),
                        Value::I64(value) => SDValue::I64(value),
                        Value::U64(value) => SDValue::U64(value),
                        Value::Null => SDValue::Null,
                        _ => return Err("Invalid value type in structured data")
                    };
                    let name = if name.starts_with("_") {
                        name.to_owned()
                    } else {
                        format!("_{}", name)
                    };
                    sd.pairs.push((name, sd_value));
                }
            }
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
            full_msg: full_msg
        };
        Ok(record)
    }
}

#[test]
fn test_gelf() {
    let msg = r#"{"version":"1.1", "host": "example.org", "short_message": "A short message that helps you identify what is going on", "full_message": "Backtrace here\n\nmore stuff", "timestamp": 1385053862.3072, "level": 1, "_user_id": 9001, "_some_info": "foo", "_some_env_var": "bar"}"#;
    let res = GelfDecoder.decode(msg).unwrap();
    assert!(res.ts == 1385053862);
    assert!(res.hostname == "example.org");
    assert!(res.msg.unwrap() == "A short message that helps you identify what is going on");
    assert!(res.full_msg.unwrap() == "Backtrace here\n\nmore stuff");
    assert!(res.severity.unwrap() == 1);

    let sd = res.sd.unwrap();
    let pairs = sd.pairs;
    assert!(pairs.iter().cloned().any(|(k, v)|
        if let SDValue::U64(v) = v { k == "_user_id" && v == 9001 } else { false }
    ));
    assert!(pairs.iter().cloned().any(|(k, v)|
        if let SDValue::String(v) = v { k == "_some_info" && v == "foo" } else { false }
    ));
    assert!(pairs.iter().cloned().any(|(k, v)|
        if let SDValue::String(v) = v { k == "_some_env_var" && v == "bar" } else { false }
    ));
}
