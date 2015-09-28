use flowgger::config::Config;
use std::io::{stdout, Write};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;
use super::Output;

const DEBUG_DEFAULT_LF: bool = true;

pub struct DebugOutput {
    lf: bool
}

impl DebugOutput {
    pub fn new(config: &Config) -> DebugOutput {
        let lf = config.lookup("output.debug_lf").map_or(DEBUG_DEFAULT_LF, |x| x.as_bool().
            expect("output.debug_lf must be a boolean") as bool);
        DebugOutput {
            lf: lf
        }
    }
}

impl Output for DebugOutput {
    fn start(&self, arx: Arc<Mutex<Receiver<Vec<u8>>>>) {
        let lf = self.lf;
        thread::spawn(move || {
            loop {
                let bytes = match { arx.lock().unwrap().recv() } {
                    Ok(line) => line,
                    Err(_) => return
                };
                let out = String::from_utf8_lossy(&bytes);
                if lf {
                    println!("{}", out);
                } else {
                    print!("{}", out);
                    let _ = stdout().flush();
                }
            }
        });
    }
}
