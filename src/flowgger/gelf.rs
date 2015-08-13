
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
        let extra = match config.lookup("output.gelf_extra") {
            None => Vec::new(),
            Some(extra) => extra.as_table().unwrap().into_iter().
                map(|(k, v)| (k.to_owned(), v.as_str().unwrap().to_owned())).collect()
        };
        Gelf { extra: extra }
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
                map = map.insert("sd_id".to_owned(), Value::String(sd.sd_id));
                for (name, value) in sd.pairs {
                    map = map.insert(name, Value::String(value));
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
