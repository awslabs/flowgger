use capnp;
use capnp::message::{Allocator, Builder};
use flowgger::config::Config;
use flowgger::record::{Record, SDValue, FACILITY_MISSING, SEVERITY_MISSING};
use flowgger::record_capnp;
use super::Encoder;

#[derive(Clone)]
pub struct CapnpEncoder;

impl CapnpEncoder {
    pub fn new(_config: &Config) -> CapnpEncoder {
        CapnpEncoder
    }
}

impl Encoder for CapnpEncoder {
    fn encode(&self, record: Record) -> Result<Vec<u8>, &'static str> {
        let mut record_msg = Builder::new_default();
        build_record(&mut record_msg, record);
        let mut bytes = Vec::new();
        try!(capnp::serialize::write_message(&mut bytes, &record_msg).
            or(Err("Unable to serialize to Cap'n Proto format")));
        Ok(bytes)
    }
}

fn build_record<T: Allocator>(record_msg: &mut capnp::message::Builder<T>, record: Record) {
    let mut root: record_capnp::record::Builder = record_msg.init_root();
    root.set_ts(record.ts);
    root.set_hostname(&record.hostname);
    match record.facility {
        Some(facility) => root.set_facility(facility),
        _ => root.set_facility(FACILITY_MISSING)
    };
    match record.severity {
        Some(severity) => root.set_severity(severity),
        _ => root.set_severity(SEVERITY_MISSING)
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
    let mut sd = root.init_sd();
    if let Some(record_sd) = record.sd {
        if let Some(sd_id) = record_sd.sd_id {
            sd.set_sd_id(&sd_id);
        }
        let pairs_count = record_sd.pairs.len();
        let mut pairs = sd.init_pairs(pairs_count as u32);
        for (i, (name, value)) in record_sd.pairs.into_iter().enumerate() {
            let mut pair = pairs.borrow().get(i as u32);
            pair.set_key(&name);
            let mut v = pair.init_value();
            match value {
                SDValue::String(value) => v.set_string(&value),
                SDValue::Bool(value) => v.set_bool(value),
                SDValue::F64(value) => v.set_f64(value),
                SDValue::I64(value) => v.set_i64(value),
                SDValue::U64(value) => v.set_u64(value),
                SDValue::Null => v.set_null(())
            };
        }
    }
}
