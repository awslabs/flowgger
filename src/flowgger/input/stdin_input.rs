use flowgger::config::Config;
use flowgger::decoder::Decoder;
use flowgger::encoder::Encoder;
use flowgger::splitter::{Splitter, CapnpSplitter, LineSplitter, NulSplitter, SyslenSplitter};
use std::io::{stdin, BufReader};
use std::sync::mpsc::SyncSender;
use super::Input;

const DEFAULT_FRAMING: &'static str = "line";

#[derive(Clone)]
pub struct StdinConfig {
    framing: String
}

pub struct StdinInput {
    stdin_config: StdinConfig
}

impl StdinInput {
    pub fn new(config: &Config) -> StdinInput {
        let framing = config.lookup("input.framing").map_or(DEFAULT_FRAMING, |x| x.as_str().
            expect(r#"input.framing must be a string set to "line", "nul" or "syslen""#)).to_owned();
        let stdin_config = StdinConfig {
            framing: framing
        };
        StdinInput {
            stdin_config: stdin_config
        }
    }
}

impl Input for StdinInput {
    fn accept(&self, tx: SyncSender<Vec<u8>>, decoder: Box<Decoder + Send>, encoder: Box<Encoder + Send>) {
        let reader = BufReader::new(stdin());
        let splitter = match &self.stdin_config.framing as &str {
            "capnp" => Box::new(CapnpSplitter) as Box<Splitter<_>>,
            "line" => Box::new(LineSplitter) as Box<Splitter<_>>,
            "syslen" => Box::new(SyslenSplitter) as Box<Splitter<_>>,
            "nul" => Box::new(NulSplitter) as Box<Splitter<_>>,
            _ => panic!("Unsupported framing scheme")
        };
        splitter.run(reader, tx, decoder, encoder);
    }
}
