use super::Output;
use crate::flowgger::config::Config;
use crate::flowgger::merger::Merger;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;

const FILE_DEFAULT_BUFFER_SIZE: usize = 1024;

pub struct FileOutput {
    path: String,
    buffer_size: usize,
}

impl FileOutput {
    pub fn new(config: &Config) -> FileOutput {
        let path = config
            .lookup("output.file_path")
            .expect("output.file_path is missing")
            .as_str()
            .expect("output.file_path must be a string")
            .to_string();
        let buffer_size =
            config
                .lookup("output.file_buffer_size")
                .map_or(FILE_DEFAULT_BUFFER_SIZE, |bs| {
                    bs.as_integer()
                        .expect("output.file_buffer_size should be an integer")
                        as usize
                });
        FileOutput { path, buffer_size }
    }
}

// TODO: Add periodic rotation handling
impl Output for FileOutput {
    fn start(&self, arx: Arc<Mutex<Receiver<Vec<u8>>>>, merger: Option<Box<dyn Merger>>) {
        let merger = match merger {
            Some(merger) => Some(merger.clone_boxed()),
            None => None,
        };

        let fd = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .unwrap_or_else(|_| panic!("Cannot open file descriptor to {}", &self.path));
        let mut writer = BufWriter::with_capacity(self.buffer_size, fd);

        thread::spawn(move || loop {
            let mut bytes = match { arx.lock().unwrap().recv() } {
                Ok(line) => line,
                Err(_) => return,
            };
            if let Some(ref merger) = merger {
                merger.frame(&mut bytes);
            }

            writer
                .write_all(&bytes)
                .expect("Cannot write bytes to output file");
        });
    }
}
