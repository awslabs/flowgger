pub mod tcp_input;
pub mod tls_input;

use flowgger::decoder::Decoder;
use flowgger::encoder::Encoder;
use std::sync::mpsc::SyncSender;

pub trait Input {
    fn accept(&self, tx: SyncSender<Vec<u8>>, decoder: Box<Decoder + Send>, encoder: Box<Encoder + Send>);
}
