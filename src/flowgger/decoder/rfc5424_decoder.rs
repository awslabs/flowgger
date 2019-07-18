use super::Decoder;
use crate::flowgger::config::Config;
use crate::flowgger::record::{Record, SDValue, StructuredData};
use crate::flowgger::utils;
use chrono::DateTime;

#[derive(Clone)]
pub struct RFC5424Decoder;

impl RFC5424Decoder {
    pub fn new(config: &Config) -> RFC5424Decoder {
        let _ = config;
        RFC5424Decoder
    }
}

impl Decoder for RFC5424Decoder {
    fn decode(&self, line: &str) -> Result<Record, &'static str> {
        let (_bom, line) = match BOM::parse(line, "<") {
            Ok(bom_line) => bom_line,
            Err(err) => return Err(err),
        };
        let mut parts = line.splitn(7, ' ');
        let pri_version = parse_pri_version(parts.next().ok_or("Missing priority and version")?)?;
        let ts = parse_ts(parts.next().ok_or("Missing timestamp")?)?;
        let hostname = parts.next().ok_or("Missing hostname")?;
        let appname = parts.next().ok_or("Missing application name")?;
        let procid = parts.next().ok_or("Missing process id")?;
        let msgid = parts.next().ok_or("Missing message id")?;
        let (sd, msg) = parse_data(parts.next().ok_or("Missing message data")?)?;
        let record = Record {
            ts,
            hostname: hostname.to_owned(),
            facility: Some(pri_version.facility),
            severity: Some(pri_version.severity),
            appname: Some(appname.to_owned()),
            procid: Some(procid.to_owned()),
            msgid: Some(msgid.to_owned()),
            sd,
            msg,
            full_msg: None,
        };
        Ok(record)
    }
}

struct Pri {
    facility: u8,
    severity: u8,
}

enum BOM {
    NONE,
    UTF8,
}

impl BOM {
    fn parse<'a>(line: &'a str, sep: &str) -> Result<(BOM, &'a str), &'static str> {
        if line.starts_with('\u{feff}') {
            Ok((BOM::UTF8, &line[3..]))
        } else if line.starts_with(sep) {
            Ok((BOM::NONE, line))
        } else {
            Err("Unsupported BOM")
        }
    }
}

fn parse_pri_version(line: &str) -> Result<Pri, &'static str> {
    println!("line: {}", line);
    if !line.starts_with('<') {
        return Err("The priority should be inside brackets");
    }
    let mut parts = line[1..].splitn(2, '>');
    let pri_encoded: u8 = parts
        .next()
        .ok_or("Empty priority")?
        .parse()
        .or(Err("Invalid priority"))?;
    let version = parts.next().ok_or("Missing version")?;
    println!("version: {}", version);
    if version != "1" {
        return Err("Unsupported version");
    }
    Ok(Pri {
        facility: pri_encoded >> 3,
        severity: pri_encoded & 7,
    })
}

fn rfc3339_to_unix(rfc3339: &str) -> Result<f64, &'static str> {
    match DateTime::parse_from_rfc3339(rfc3339) {
        Ok(date) => Ok(utils::PreciseTimestamp::from_datetime(date).as_f64()),
        Err(_) => Err("Unable to parse the date"),
    }
}

fn parse_ts(line: &str) -> Result<f64, &'static str> {
    rfc3339_to_unix(line)
}

fn unescape_sd_value(value: &str) -> String {
    let mut res = "".to_owned();
    let mut esc = false;

    for c in value.chars() {
        match (c, esc) {
            ('\\', false) => esc = true,
            (_, false) => res.push(c),
            ('"', true) | ('\\', true) | (']', true) => {
                res.push(c);
                esc = false;
            }
            (_, true) => {
                res.push('\\');
                res.push(c);
                esc = false;
            }
        }
    }
    res
}

fn parse_msg(line: &str, offset: usize) -> Option<String> {
    if offset > line.len() {
        None
    } else {
        match line[offset..].trim() {
            "" => None,
            m => Some(m.to_owned()),
        }
    }
}

fn parse_data(line: &str) -> Result<(Option<StructuredData>, Option<String>), &'static str> {
    match line.chars().next().ok_or("Short message")? {
        '-' => {
            return Ok((None, parse_msg(line, 1)));
        }
        '[' => {}
        _ => return Err("Short message"),
    };
    let mut parts = line[1..].splitn(2, ' ');
    let sd_id = parts.next().ok_or("Missing structured data id")?;
    let sd = parts.next().ok_or("Missing structured data")?;
    let mut in_name = false;
    let mut in_value = false;
    let mut name_start = 0;
    let mut value_start = 0;
    let mut name: Option<&str> = None;
    let mut esc = false;
    let mut after_sd: Option<usize> = None;
    let mut sd_res = StructuredData::new(Some(sd_id));

    for (i, c) in sd.char_indices() {
        let is_sd_name = match c as u32 {
            32 | 34 | 61 | 93 => false,
            33..=126 => true,
            _ => false,
        };
        match (c, esc, is_sd_name, in_name, name.is_some(), in_value) {
            (' ', false, _, false, false, _) => {
                // contextless spaces
            }
            (']', false, _, false, false, _) => {
                after_sd = Some(i + 1);
                break;
            }
            (_, false, true, false, false, _) => {
                in_name = true;
                name_start = i;
            }
            (_, _, true, true, false, _) => {
                // name
            }
            ('=', false, _, true, ..) => {
                name = Some(&sd[name_start..i]);
                in_name = false;
            }
            ('"', false, _, _, true, false) => {
                in_value = true;
                value_start = i + 1;
            }
            ('\\', false, _, _, _, true) => esc = true,
            ('"', false, _, _, _, true) => {
                in_value = false;
                let value = unescape_sd_value(&sd[value_start..i]);
                let pair = (
                    "_".to_owned()
                        + name.expect(
                            "Name in structured data contains an invalid UTF-8 \
                             sequence",
                        ),
                    SDValue::String(value),
                );
                sd_res.pairs.push(pair);
                name = None;
            }
            (_, _, _, _, _, true) => esc = false,
            ('"', false, _, false, false, _) => {
                // tolerate bogus entries with extra "
            }
            _ => return Err("Format error in the structured data"),
        }
    }
    match after_sd {
        None => Err("Missing ] after structured data"),
        Some(offset) => Ok((Some(sd_res), parse_msg(sd, offset))),
    }
}

#[test]
fn test_rfc5424() {
    let msg = r#"<23>1 2015-08-05T15:53:45.637824Z testhostname appname 69 42 [origin@123 software="te\st sc\"ript" swVersion="0.0.1"] test message"#;
    let res = RFC5424Decoder.decode(msg).unwrap();
    assert!(res.facility.unwrap() == 2);
    assert!(res.severity.unwrap() == 7);
    assert!(res.ts == 1438790025.637824);
    assert!(res.hostname == "testhostname");
    assert!(res.appname == Some("appname".to_owned()));
    assert!(res.procid == Some("69".to_owned()));
    assert!(res.msgid == Some("42".to_owned()));
    assert!(res.msg == Some("test message".to_owned()));
    let sd = res.sd.unwrap();
    assert!(sd.sd_id == Some("origin@123".to_owned()));
    let pairs = sd.pairs;

    assert!(pairs
        .iter()
        .cloned()
        .any(|(k, v)| if let SDValue::String(v) = v {
            k == "_software" && v == "te\\st sc\"ript"
        } else {
            false
        }));
    assert!(pairs
        .iter()
        .cloned()
        .any(|(k, v)| if let SDValue::String(v) = v {
            k == "_swVersion" && v == "0.0.1"
        } else {
            false
        }));
}
