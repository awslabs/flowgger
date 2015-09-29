mod debug_output;
mod kafka_output;

pub use self::debug_output::DebugOutput;
pub use self::kafka_output::KafkaOutput;

use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

pub trait Output {
    fn start(&self, arx: Arc<Mutex<Receiver<Vec<u8>>>>);
}
