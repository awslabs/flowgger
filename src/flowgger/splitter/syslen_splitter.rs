use flowgger::decoder::Decoder;
use flowgger::encoder::Encoder;
use std::io::{stderr, ErrorKind, Read, Write, BufRead, BufReader};
use std::str;
use std::sync::mpsc::SyncSender;
use super::Splitter;

pub struct SyslenSplitter;

impl<T: Read> Splitter<T> for SyslenSplitter {
    fn run(&self, buf_reader: BufReader<T>, tx: SyncSender<Vec<u8>>, decoder: Box<Decoder>, encoder: Box<Encoder>) {
        let mut buf_reader = buf_reader;
        loop {
            if let Err(e) = read_msglen(&mut buf_reader) {
                let _ = writeln!(stderr(), "{}", e);
                return;
            }
            let mut line = String::new();
            match buf_reader.read_line(&mut line) {
                Ok(_) => { },
                Err(e) => match e.kind() {
                    ErrorKind::Interrupted => continue,
                    ErrorKind::InvalidInput | ErrorKind::InvalidData => {
                        let _ = writeln!(stderr(), "Invalid UTF-8 input");
                        continue;
                    },
                    ErrorKind::WouldBlock => {
                        let _ = writeln!(stderr(), "Client hasn't sent any data for a while - Closing idle connection");
                        return
                    },
                    _ => return
                }
            }
            if let Err(e) = handle_line(&line, &tx, &decoder, &encoder) {
                let _ = writeln!(stderr(), "{}: [{}]", e, line.trim());
            }
        }
    }
}

fn read_msglen(reader: &mut BufRead) -> Result<usize, &'static str> {
    let mut nbytes_v = Vec::with_capacity(16);
    let nbytes_vl = match reader.read_until(b' ', &mut nbytes_v) {
        Err(_) | Ok(0) | Ok(1) => return Err("Connection closed"),
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
