mod redis_input;
mod stdin_input;
mod tcp;
mod tls;
mod udp_input;
mod file;

pub use self::redis_input::RedisInput;
pub use self::stdin_input::StdinInput;
pub use self::tcp::tcp_input::TcpInput;
#[cfg(feature = "coroutines")]
pub use self::tcp::tcpco_input::TcpCoInput;
pub use self::tls::tls_input::TlsInput;
#[cfg(feature = "coroutines")]
pub use self::tls::tlsco_input::TlsCoInput;
pub use self::udp_input::UdpInput;
pub use self::file::FileInput;

use crate::flowgger::decoder::Decoder;
use crate::flowgger::encoder::Encoder;
use std::sync::mpsc::SyncSender;

pub trait Input {
    fn accept(
        &self,
        tx: SyncSender<Vec<u8>>,
        decoder: Box<Decoder + Send>,
        encoder: Box<Encoder + Send>,
    );
}
