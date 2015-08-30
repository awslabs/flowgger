pub mod tcp_input;
pub mod tls_input;

use flowgger::config::Config;
use flowgger::decoder::Decoder;
use flowgger::encoder::Encoder;
use std::sync::mpsc::SyncSender;

pub trait Input {
    fn new(config: &Config) -> Self;
    fn accept<TE>(&self, tx: SyncSender<Vec<u8>>, decoder: Box<Decoder + Send>, encoder: TE) where TE: Encoder + Clone + Send + 'static;
}
