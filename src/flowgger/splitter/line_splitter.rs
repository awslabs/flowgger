use flowgger::decoder::Decoder;
use flowgger::encoder::Encoder;
use std::io::{stderr, Read, Write, BufRead, BufReader};
use std::sync::mpsc::SyncSender;

pub struct LineSplitter<T: Read> {
    buf_reader: BufReader<T>,
    tx: SyncSender<Vec<u8>>,
    decoder: Box<Decoder>,
    encoder: Box<Encoder>
}

impl<T: Read> LineSplitter<T> {
    pub fn new(buf_reader: BufReader<T>, tx: SyncSender<Vec<u8>>, decoder: Box<Decoder>, encoder: Box<Encoder>) -> LineSplitter<T> {
        LineSplitter {
            buf_reader: buf_reader,
            tx: tx,
            decoder: decoder,
            encoder: encoder
        }
    }

    pub fn run(self) {
        let tx = self.tx;
        let (decoder, encoder) = (self.decoder, self.encoder);
        for line in self.buf_reader.lines() {
            let line = match line {
                Err(_) => {
                    let _ = writeln!(stderr(), "Invalid UTF-8 input");
                    continue;
                }
                Ok(line) => line
            };
            if let Err(e) = handle_line(&line, &tx, &decoder, &encoder) {
                let _ = writeln!(stderr(), "{}: [{}]", e, line.trim());
            }
        }
    }
}

fn handle_line(line: &String, tx: &SyncSender<Vec<u8>>, decoder: &Box<Decoder>, encoder: &Box<Encoder>) -> Result<(), &'static str> {
    let decoded = try!(decoder.decode(&line));
    let reencoded = try!(encoder.encode(decoded));
    tx.send(reencoded).unwrap();
    Ok(())
}
