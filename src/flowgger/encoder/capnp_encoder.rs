use capnp;
use capnp::message::{Allocator, Builder};
use flowgger::config::Config;
use flowgger::record::{Record, SDValue, FACILITY_MISSING, SEVERITY_MISSING};
use flowgger::record_capnp;
use super::Encoder;

#[derive(Clone)]
pub struct CapnpEncoder {
    extra: Vec<(String, String)>,
}

impl CapnpEncoder {
    pub fn new(config: &Config) -> CapnpEncoder {
        let extra = match config.lookup("output.capnp_extra") {
            None => Vec::new(),
            Some(extra) => {
                extra.as_table()
                    .expect("output.capnp_extra must be a list of key/value pairs")
                    .into_iter()
                    .map(|(k, v)| {
                        (k.to_owned(),
                         v.as_str()
                             .expect("output.capnp_extra values must be strings")
                             .to_owned())
                    })
                    .collect()
            }
        };
        CapnpEncoder { extra: extra }
    }
}

impl Encoder for CapnpEncoder {
    fn encode(&self, record: Record) -> Result<Vec<u8>, &'static str> {
        let mut record_msg = Builder::new_default();
        build_record(&mut record_msg, record, &self.extra);
        let mut bytes = Vec::new();
        try!(capnp::serialize::write_message(&mut bytes, &record_msg)
            .or(Err("Unable to serialize to Cap'n Proto format")));
        Ok(bytes)
    }
}

fn build_record<T: Allocator>(record_msg: &mut capnp::message::Builder<T>,
                              record: Record,
                              extra: &Vec<(String, String)>) {
    let mut root: record_capnp::record::Builder = record_msg.init_root();
    root.set_ts(record.ts);
    root.set_hostname(&record.hostname);
    match record.facility {
        Some(facility) => root.set_facility(facility),
        _ => root.set_facility(FACILITY_MISSING),
    };
    match record.severity {
        Some(severity) => root.set_severity(severity),
        _ => root.set_severity(SEVERITY_MISSING),
    };
    if let Some(appname) = record.appname {
        root.set_appname(&appname);
    }
    if let Some(procid) = record.procid {
        root.set_procid(&procid);
    }
    if let Some(msgid) = record.msgid {
        root.set_msgid(&msgid);
    }
    if let Some(msg) = record.msg {
        root.set_msg(&msg);
    }
    if let Some(full_msg) = record.full_msg {
        root.set_full_msg(&full_msg);
    }
    if record.sd.is_none() && extra.is_empty() {
        return;
    }
    let sd_pairs_len = match record.sd {
        Some(ref sd) => sd.pairs.len(),
        None => 0,
    };
    let pairs_count = sd_pairs_len + extra.len();
    record.sd
        .as_ref()
        .and_then(|x| x.sd_id.as_ref())
        .and_then(|sd_id| Some(root.set_sd_id(sd_id)));
    let mut pairs = root.init_pairs(pairs_count as u32);
    if let Some(sd) = record.sd {
        for (i, (name, value)) in sd.pairs.into_iter().enumerate() {
            let mut pair = pairs.borrow().get(i as u32);
            pair.set_key(&name);
            let mut v = pair.init_value();
            match value {
                SDValue::String(value) => v.set_string(&value),
                SDValue::Bool(value) => v.set_bool(value),
                SDValue::F64(value) => v.set_f64(value),
                SDValue::I64(value) => v.set_i64(value),
                SDValue::U64(value) => v.set_u64(value),
                SDValue::Null => v.set_null(()),
            };
        }
    }
    for (i, &(ref name, ref value)) in extra.into_iter().enumerate() {
        let mut pair = pairs.borrow().get((sd_pairs_len + i) as u32);
        pair.set_key(&name);
        let mut v = pair.init_value();
        v.set_string(&value)
    }
}
