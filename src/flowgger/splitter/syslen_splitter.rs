use flowgger::decoder::Decoder;
use flowgger::encoder::Encoder;
use std::cmp;
use std::io;
use std::io::{stderr, Read, Write, BufRead, BufReader};
use std::str;
use std::sync::mpsc::SyncSender;
use super::Splitter;

pub struct SyslenSplitter {
    tx: SyncSender<Vec<u8>>,
    decoder: Box<Decoder>,
    encoder: Box<Encoder>
}

impl<T: Read> Splitter<T> for SyslenSplitter {
    fn run(&self, buf_reader: BufReader<T>) {
        let mut buf_reader = buf_reader;
        let tx = &self.tx;
        let (decoder, encoder) = (&self.decoder, &self.encoder);
        loop {
            let msg_len = match read_msglen(&mut buf_reader) {
                Err(e) => {
                    let _ = writeln!(stderr(), "{}", e);
                    return;
                }
                Ok(msg_len) => msg_len
            };
            let mut line = vec![0; msg_len];
            if read_exact(&mut buf_reader, &mut line).is_err() {
                println!("err");
                return;
            }
            let line = match str::from_utf8(&line) {
                Err(_) => {
                    let _ = writeln!(stderr(), "Invalid UTF-8 sequence");
                    return;
                }
                Ok(line) => line
            };
            if let Err(e) = handle_line(line, tx, decoder, encoder) {
                let _ = writeln!(stderr(), "{}: [{}]", e, line.trim());
            }
        }
    }
}

impl SyslenSplitter {
    pub fn new(tx: SyncSender<Vec<u8>>, decoder: Box<Decoder>, encoder: Box<Encoder>) -> SyslenSplitter {
        SyslenSplitter {
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

fn read_exact<R: BufRead + ?Sized>(reader: &mut R, buf: &mut Vec<u8>) -> io::Result<usize> {
    let len = buf.len();
    let mut to_read = len;
    buf.clear();
    while to_read > 0 {
        let used = {
            let buffer = match reader.fill_buf() {
                Ok(buffer) => buffer,
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e)
            };
            let used = cmp::min(buffer.len(), to_read);
            buf.push_all(&buffer[..used]);
            used
        };
        reader.consume(used);
        to_read -= used;
    }
    Ok(len)
}

fn handle_line(line: &str, tx: &SyncSender<Vec<u8>>, decoder: &Box<Decoder>, encoder: &Box<Encoder>) -> Result<(), &'static str> {
    let decoded = try!(decoder.decode(line));
    let reencoded = try!(encoder.encode(decoded));
    tx.send(reencoded).unwrap();
    Ok(())
}
