pub mod nul_splitter;
pub mod line_splitter;
pub mod syslen_splitter;

use flowgger::decoder::Decoder;
use flowgger::encoder::Encoder;
use std::io::BufReader;
use std::sync::mpsc::SyncSender;

pub trait Splitter<T> {
    fn run(&self, buf_reader: BufReader<T>, tx: SyncSender<Vec<u8>>, decoder: Box<Decoder>, encoder: Box<Encoder>);
}
