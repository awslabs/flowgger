use flowgger::config::Config;
use super::Merger;

#[derive(Clone)]
pub struct NulMerger;

impl NulMerger {
    pub fn new(_config: &Config) -> NulMerger {
        NulMerger
    }
}

impl Merger for NulMerger {
    fn frame(&self, bytes: &mut Vec<u8>) {
        bytes.push(0);
    }
}
