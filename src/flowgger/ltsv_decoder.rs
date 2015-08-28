extern crate chrono;

use flowgger::Decoder;
use flowgger::config::Config;
use flowgger::record::{Record, StructuredData, SDValue, SDValueType};
use std::collections::HashMap;
use self::chrono::DateTime;

#[derive(Clone)]
pub struct LTSVDecoder {
    schema: Option<HashMap<String, SDValueType>>
}

impl LTSVDecoder {
    pub fn new(config: &Config) -> LTSVDecoder {
        let schema = match config.lookup("input.ltsv_schema") {
            None => None,
            Some(pairs) => {
                let mut schema = HashMap::new();
                for (name, sdtype) in pairs.as_table().expect("input.ltsv_schema must be a list of key/type pairs") {
                    let sdtype = match sdtype.as_str().expect("input.ltsv_schema types must be strings").to_lowercase().as_ref() {
                        "string" => SDValueType::String,
                        "bool" => SDValueType::Bool,
                        "f64" => SDValueType::F64,
                        "i64" => SDValueType::I64,
                        "u64" => SDValueType::U64,
                        _ => panic!(format!("Unsupported type in input.ltsv_schema for name [{}]", name))
                    };
                    schema.insert(name.to_owned(), sdtype);
                }
                Some(schema)
            }
        };
        LTSVDecoder {
            schema: schema
        }
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
                name @ _ => {
                    let value: SDValue = if let Some(ref schema) = self.schema {
                        match schema.get(name) {
                            None | Some(&SDValueType::String) =>
                                SDValue::String(value.to_owned()),
                            Some(&SDValueType::Bool) =>
                                SDValue::Bool(try!(value.parse::<bool>().or(Err("Type error; boolean was expected")))),
                            Some(&SDValueType::F64) =>
                                SDValue::F64(try!(value.parse::<f64>().or(Err("Type error; f64 was expected")))),
                            Some(&SDValueType::I64) =>
                                SDValue::I64(try!(value.parse::<i64>().or(Err("Type error; i64 was expected")))),
                            Some(&SDValueType::U64) =>
                                SDValue::U64(try!(value.parse::<u64>().or(Err("Type error; u64 was expected"))))
                        }
                    } else {
                        SDValue::String(value.to_owned())
                    };
                    sd.pairs.push((format!("_{}", name), value));
                }
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
    let config = Config::from_string("[input]\n[input.ltsv_schema]\ncounter = \"u64\"");
    let ltsv_decoder = LTSVDecoder::new(&config.unwrap());

    let mut msg = "time:[2015-08-05T15:53:45.637824Z]\thost:testhostname\tname1:value1\tname 2: value 2\tn3:v3";
    let mut res = ltsv_decoder.decode(msg).unwrap();
    assert!(res.ts == 1438790025);

    msg = "time:[10/Oct/2000:13:55:36 -0700]\tcounter:42\tlevel:3\thost:testhostname\tname1:value1\tname 2: value 2\tn3:v3\tmessage:this is a test";
    res = ltsv_decoder.decode(msg).unwrap();
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
    assert!(pairs.iter().cloned().any(|(k, v)|
        if let SDValue::U64(v) = v { k == "_counter" && v == 42 } else { false }
    ));
}
