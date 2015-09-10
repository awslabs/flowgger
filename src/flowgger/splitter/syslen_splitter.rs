use flowgger::decoder::Decoder;
use flowgger::encoder::Encoder;
use std::io::{stderr, Read, Write, BufRead, BufReader};
use std::str;
use std::sync::mpsc::SyncSender;
use super::Splitter;

pub struct SyslenSplitter<T: Read> {
    buf_reader: BufReader<T>,
    tx: SyncSender<Vec<u8>>,
    decoder: Box<Decoder>,
    encoder: Box<Encoder>
}

impl<T: Read> Splitter for SyslenSplitter<T> {
    fn run(self) {
        let tx = self.tx;
        let (decoder, encoder) = (self.decoder, self.encoder);
        let mut buf_reader = self.buf_reader;
        loop {
            if let Err(e) = read_msglen(&mut buf_reader) {
                let _ = writeln!(stderr(), "{}", e);
                return;
            }
            let mut line = String::new();
            if buf_reader.read_line(&mut line).is_err() {
                println!("err");
                return;
            }
            if let Err(e) = handle_line(&line, &tx, &decoder, &encoder) {
                let _ = writeln!(stderr(), "{}: [{}]", e, line.trim());
            }
        }
    }
}

impl<T: Read> SyslenSplitter<T> {
    pub fn new(buf_reader: BufReader<T>, tx: SyncSender<Vec<u8>>, decoder: Box<Decoder>, encoder: Box<Encoder>) -> SyslenSplitter<T> {
        SyslenSplitter {
            buf_reader: buf_reader,
            tx: tx,
            decoder: decoder,
            encoder: encoder
        }
    }
}

fn read_msglen(reader: &mut BufRead) -> Result<usize, &'static str> {
    let mut nbytes_v = Vec::with_capacity(16);
    let nbytes_vl = match reader.read_until(b' ', &mut nbytes_v) {
        Err(_) | Ok(0) | Ok(1) => return Err("EOF"),
        Ok(nbytes_vl) => nbytes_vl
    };
    let nbytes_s = match str::from_utf8(&nbytes_v[..nbytes_vl - 1]) {
        Err(_) => return Err("Invalid or missing message length. Disable framing, maybe?"),
        Ok(nbytes_s) => nbytes_s
    };
    let nbytes: usize = match nbytes_s.parse() {
        Err(_) => return Err("Invalid message length. Disable framing, maybe?"),
        Ok(nbytes) => nbytes
    };
    Ok(nbytes)
}

fn handle_line(line: &String, tx: &SyncSender<Vec<u8>>, decoder: &Box<Decoder>, encoder: &Box<Encoder>) -> Result<(), &'static str> {
    let decoded = try!(decoder.decode(&line));
    let reencoded = try!(encoder.encode(decoded));
    tx.send(reencoded).unwrap();
    Ok(())
}
