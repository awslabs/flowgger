pub mod gelf_encoder;

use flowgger::config::Config;
use flowgger::record::Record;

pub trait Encoder {
    fn new(config: &Config) -> Self;
    fn encode(&self, record: Record) -> Result<Vec<u8>, &'static str>;
}
