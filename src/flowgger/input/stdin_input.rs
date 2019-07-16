use super::Input;
use crate::flowgger::config::Config;
use crate::flowgger::decoder::Decoder;
use crate::flowgger::encoder::Encoder;
use crate::flowgger::splitter::{
    CapnpSplitter, LineSplitter, NulSplitter, Splitter, SyslenSplitter,
};
use std::io::{stdin, BufReader};
use std::sync::mpsc::SyncSender;

const DEFAULT_FRAMING: &str = "line";

#[derive(Clone)]
pub struct StdinConfig {
    framing: String,
}

pub struct StdinInput {
    stdin_config: StdinConfig,
}

impl StdinInput {
    pub fn new(config: &Config) -> StdinInput {
        let framing = config
            .lookup("input.framing")
            .map_or(DEFAULT_FRAMING, |x| {
                x.as_str()
                    .expect(r#"input.framing must be a string set to "line", "nul" or "syslen""#)
            })
            .to_owned();
        let stdin_config = StdinConfig { framing };
        StdinInput { stdin_config }
    }
}

impl Input for StdinInput {
    fn accept(
        &self,
        tx: SyncSender<Vec<u8>>,
        decoder: Box<dyn Decoder + Send>,
        encoder: Box<dyn Encoder + Send>,
    ) {
        let reader = BufReader::new(stdin());
        let splitter = match &self.stdin_config.framing as &str {
            "capnp" => Box::new(CapnpSplitter) as Box<dyn Splitter<_>>,
            "line" => Box::new(LineSplitter) as Box<dyn Splitter<_>>,
            "syslen" => Box::new(SyslenSplitter) as Box<dyn Splitter<_>>,
            "nul" => Box::new(NulSplitter) as Box<dyn Splitter<_>>,
            _ => panic!("Unsupported framing scheme"),
        };
        splitter.run(reader, tx, decoder, encoder);
    }
}
