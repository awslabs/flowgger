mod debug_output;
#[cfg(feature = "kafka")]
mod kafka_output;
mod tls_output;

pub use self::debug_output::DebugOutput;
#[cfg(feature = "kafka")]
pub use self::kafka_output::KafkaOutput;
pub use self::tls_output::TlsOutput;

use flowgger::merger::Merger;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Receiver;

pub trait Output {
    fn start(&self, arx: Arc<Mutex<Receiver<Vec<u8>>>>, merger: Option<Box<Merger>>);
}
