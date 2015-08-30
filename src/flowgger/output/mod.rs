pub mod kafka_output;

use flowgger::config::Config;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

pub trait Output {
    fn new(config: &Config) -> Self;
    fn start(&self, arx: Arc<Mutex<Receiver<Vec<u8>>>>);
}
