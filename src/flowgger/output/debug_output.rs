use flowgger::config::Config;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;
use super::Output;

pub struct DebugOutput;

impl DebugOutput {
    pub fn new(config: &Config) -> DebugOutput {
        let _ = config;
        DebugOutput
    }
}

impl Output for DebugOutput {
    fn start(&self, arx: Arc<Mutex<Receiver<Vec<u8>>>>) {
        thread::spawn(move || {
            loop {
                let bytes = match { arx.lock().unwrap().recv() } {
                    Ok(line) => line,
                    Err(_) => return
                };
                println!("{}", String::from_utf8_lossy(&bytes));
            }
        });
    }
}
