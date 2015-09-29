mod capnp_splitter;
mod nul_splitter;
mod line_splitter;
mod syslen_splitter;

pub use self::capnp_splitter::CapnpSplitter;
pub use self::nul_splitter::NulSplitter;
pub use self::line_splitter::LineSplitter;
pub use self::syslen_splitter::SyslenSplitter;

use flowgger::decoder::Decoder;
use flowgger::encoder::Encoder;
use std::io::BufReader;
use std::sync::mpsc::SyncSender;

pub trait Splitter<T> {
    fn run(&self, buf_reader: BufReader<T>, tx: SyncSender<Vec<u8>>, decoder: Box<Decoder>, encoder: Box<Encoder>);
}
