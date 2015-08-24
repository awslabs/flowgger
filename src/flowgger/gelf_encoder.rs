
extern crate serde;
extern crate serde_json;

use flowgger::Encoder;
use flowgger::config::Config;
use flowgger::record::{Record, SDValue};
use self::serde_json::builder::ObjectBuilder;
use self::serde_json::value::Value;

#[derive(Clone)]
pub struct GelfEncoder {
    extra: Vec<(String, String)>
}

impl Encoder for GelfEncoder {
    fn new(config: &Config) -> GelfEncoder {
        let extra = match config.lookup("output.gelf_extra") {
            None => Vec::new(),
            Some(extra) => extra.as_table().expect("output.gelf_extra must be a list of key/value pairs").
                into_iter().map(|(k, v)| (k.to_owned(), v.as_str().
                expect("output.gelf_extra values mmust be strings").to_owned())).collect()
        };
        GelfEncoder { extra: extra }
    }

    fn encode(&self, record: Record) -> Result<Vec<u8>, &'static str> {
        let mut map = ObjectBuilder::new().
            insert("version".to_owned(), Value::String("1.1".to_owned())).
            insert("host".to_owned(), Value::String(record.hostname)).
            insert("short_message".to_owned(), Value::String(record.msg.unwrap_or("-".to_owned()))).
            insert("timestamp".to_owned(), Value::I64(record.ts));
        match record.pri {
            None => { },
            Some(pri) => { map = map.insert("level".to_owned(), Value::U64(pri.severity as u64)); }
        }
        match record.sd {
            None => { },
            Some(sd) => {
                if let Some(sd_id) = sd.sd_id {
                    map = map.insert("sd_id".to_owned(), Value::String(sd_id));
                }
                for (name, value) in sd.pairs {
                    let value = match value {
                        SDValue::String(value) => Value::String(value),
                        SDValue::Bool(value) => Value::Bool(value),
                        SDValue::F64(value) => Value::F64(value),
                        SDValue::I64(value) => Value::I64(value),
                        SDValue::U64(value) => Value::U64(value)
                    };
                    map = map.insert(name, value);
                }
            }
        }
        for (name, value) in self.extra.iter().cloned() {
            map = map.insert(name, Value::String(value));
        }
        let json = serde_json::to_vec(&map.unwrap());
        Ok(json)
    }
}
