#![plugin(serde_macros)]

extern crate serde;
extern crate serde_json;

use flowgger::Encoder;
use flowgger::config::Config;
use flowgger::record::Record;
use self::serde_json::builder::ObjectBuilder;
use self::serde_json::value::Value;

#[derive(Clone)]
pub struct Gelf {
    extra: Vec<(String, String)>
}

impl Encoder for Gelf {
    fn new(config: &Config) -> Gelf {
        let extra = Vec::new();
        Gelf { extra: extra }
    }

    fn encode(&self, record: Record) -> Result<Vec<u8>, &'static str> {
        let mut map = ObjectBuilder::new().
            insert("version".to_string(), Value::String("1.1".to_string())).
            insert("host".to_string(), Value::String(record.hostname)).
            insert("short_message".to_string(), Value::String(record.msg.unwrap_or("-".to_string()))).
            insert("timestamp".to_string(), Value::I64(record.ts));
        match record.pri {
            None => { },
            Some(pri) => { map = map.insert("level".to_string(), Value::U64(pri.severity as u64)); }
        }
        match record.sd {
            None => { },
            Some(sd) => {
                map = map.insert("sd_id".to_string(), Value::String(sd.sd_id));
                for (name, value) in sd.pairs {
                    map = map.insert(name, Value::String(value));
                }
            }
        }
        let json = serde_json::to_vec(&map.unwrap());
        Ok(json)
    }
}
