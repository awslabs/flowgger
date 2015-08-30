pub mod kafka_output;

use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

pub trait Output {
    fn start(&self, arx: Arc<Mutex<Receiver<Vec<u8>>>>);
}
