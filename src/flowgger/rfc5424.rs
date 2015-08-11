extern crate chrono;

use flowgger::*;
use flowgger::record::{Record, Pri, StructuredData};
use self::chrono::DateTime;

#[derive(Clone)]
pub struct RFC5424;

impl Decoder for RFC5424 {
    fn decode(&self, line: &str) -> Result<Record, &'static str> {
        let (bom, line) = match BOM::parse(line, "<") {
            Ok(bom_line) => bom_line,
            Err(err) => return Err(err)
        };
        let mut parts = line.splitn(7, ' ');
        let pri_version = try!(parse_pri_version(try!(parts.next().ok_or("Missing priority and version"))));
        let ts = try!(parse_ts(try!(parts.next().ok_or("Missing timestamp"))));
        let hostname = try!(parts.next().ok_or("Missing hostname"));
        let appname = try!(parts.next().ok_or("Missing application name"));
        let procid = try!(parts.next().ok_or("Missing process id"));
        let msgid = try!(parts.next().ok_or("Missing message id"));
        let (sd, msg) = try!(parse_data(try!(parts.next().ok_or("Missing message data"))));
        let record = Record {
            pri: Some(pri_version),
            ts: ts,
            hostname: hostname.to_string(),
            appname: Some(appname.to_string()),
            procid: Some(procid.to_string()),
            msgid: Some(msgid.to_string()),
            sd: sd,
            msg: msg
        };
        Ok(record)
    }

    fn new() -> RFC5424 {
        RFC5424
    }
}

impl Pri {
    fn new(encoded: u8) -> Pri {
        Pri {
            facility: encoded >> 3,
            severity: encoded % 7
        }
    }
}

enum BOM {
    NONE,
    UTF8
}

impl BOM {
    fn parse<'a>(line: &'a str, sep: &str) -> Result<(BOM, &'a str), &'static str> {
        if line.starts_with("\u{feff}") {
            Ok((BOM::UTF8, &line[3..]))
        } else if line.starts_with(sep) {
            Ok((BOM::NONE, line))
        } else {
            Err("Unsupported BOM")
        }
    }
}

fn parse_pri_version(line: &str) -> Result<Pri, &'static str> {
    if ! line.starts_with("<") {
        return Err("The priority should be inside brackets")
    }
    let mut parts = line[1..].splitn(2, '>');
    let pri: u8 = try!(try!(parts.next().ok_or("Empty priority")).parse().or(Err("Invalid priority")));
    let version = try!(parts.next().ok_or("Missing version"));
    if version != "1" {
        return Err("Unsupported version");
    }
    Ok(Pri::new(pri))
}

fn rfc3339_to_unix(rfc3339: &str) -> Result<i64, &'static str> {
    match DateTime::parse_from_rfc3339(rfc3339) {
        Ok(date) => Ok(date.timestamp()),
        Err(_) => Err("Unable to parse the date")
    }
}

fn parse_ts(line: &str) -> Result<i64, &'static str> {
    rfc3339_to_unix(line)
}

fn unescape_sd_value(value: &str) -> String {
    let mut res = "".to_string();
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
            m @ _ => Some(m.to_string())
        }
    }
}

fn parse_data(line: &str) -> Result<(Option<StructuredData>, Option<String>), &'static str> {
    match try!(line.chars().next().ok_or("Short message")) {
        '-' => {
            return Ok((None, parse_msg(line, 1)));
        }
        '[' => { }
        _ => return Err("Short message")
    };
    let mut parts = line[1..].splitn(2, ' ');
    let sd_id = try!(parts.next().ok_or("Missing structured data id"));
    let sd = try!(parts.next().ok_or("Missing structured data"));
    let mut in_name = false;
    let mut in_value = false;
    let mut name_start = 0;
    let mut value_start = 0;
    let mut name: Option<&str> = None;
    let mut esc = false;
    let mut after_sd: Option<usize> = None;
    let mut sd_res = StructuredData::new(sd_id);

    for (i, c) in sd.chars().enumerate() {
        let is_sd_name = match c as u32 {
            32 | 34 | 61 | 93 => false,
            33 ... 126 => true,
            _ => false
        };
        match (c, esc, is_sd_name, in_name, name.is_some(), in_value) {
            (' ', false, _, false, false, _) => { /* contextless spaces */ }
            (']', false, _, false, false, _) => {
                after_sd = Some(i + 1);
                break;
            }
            (_, false, true, false, false, _) => {
                in_name = true;
                name_start = i;
            }
            (_, _, true, true, false, _) => { /* name */ }
            ('=', false, _, true, _, _) => {
                name = Some(&sd[name_start .. i]);
                in_name = false;
            }
            ('"', false, _, _, true, false) => {
                in_value = true;
                value_start = i + 1;
            }
            ('\\', false, _, _, _, true) => esc = true,
            ('"', false, _, _, _, true) => {
                in_value = false;
                let value = unescape_sd_value(&sd[value_start .. i]);
                let pair = (format!("_{}", name.unwrap()), value);
                sd_res.pairs.push(pair);
                name = None;
            }
            (_, _, _, _, _, true) => esc = false,
            _ => return Err("Format error in the structured data")
        }
    }
    match after_sd {
        None => Err("Missing ] after structured data"),
        Some(offset) => Ok((Some(sd_res), parse_msg(sd, offset)))
    }
}
