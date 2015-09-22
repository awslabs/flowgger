pub mod redis_input;
pub use self::tls::tls_input;
#[cfg(feature = "coroutines")]
pub use self::tls::tlsco_input;
pub mod stdin_input;
pub mod tcp_input;
pub mod tls;
pub mod udp_input;

use flowgger::decoder::Decoder;
use flowgger::encoder::Encoder;
use std::sync::mpsc::SyncSender;

pub trait Input {
    fn accept(&self, tx: SyncSender<Vec<u8>>, decoder: Box<Decoder + Send>, encoder: Box<Encoder + Send>);
}
