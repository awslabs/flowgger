use flowgger::config::Config;
use flowgger::decoder::Decoder;
use flowgger::encoder::Encoder;
use flate2::{Decompress, Flush, Status};
use std::io::{stderr, Write};
use std::net::UdpSocket;
use std::str;
use std::sync::mpsc::SyncSender;
use super::Input;

const DEFAULT_LISTEN: &'static str = "0.0.0.0:514";
const MAX_UDP_PACKET_SIZE: usize = 65527;
const MAX_COMPRESSION_RATIO: usize = 5;

pub struct UdpInput {
    listen: String,
}

impl UdpInput {
    pub fn new(config: &Config) -> UdpInput {
        let listen = config.lookup("input.listen")
            .map_or(DEFAULT_LISTEN,
                    |x| x.as_str().expect("input.listen must be an ip:port string"))
            .to_owned();
        UdpInput { listen: listen }
    }
}

impl Input for UdpInput {
    fn accept(&self,
              tx: SyncSender<Vec<u8>>,
              decoder: Box<Decoder + Send>,
              encoder: Box<Encoder + Send>) {
        let socket = UdpSocket::bind(&self.listen as &str)
            .expect(&format!("Unable to listen to {}", self.listen));
        let tx = tx.clone();
        let (decoder, encoder): (Box<Decoder>, Box<Encoder>) = (decoder.clone_boxed(),
                                                                encoder.clone_boxed());
        let mut buf = [0; MAX_UDP_PACKET_SIZE];
        loop {
            let (length, _src) = match socket.recv_from(&mut buf) {
                Ok(res) => res,
                Err(_) => continue,
            };
            let line = &buf[..length];
            if let Err(e) = handle_record_maybe_compressed(&line, &tx, &decoder, &encoder) {
                let _ = writeln!(stderr(), "{}", e);
            }
        }
    }
}

fn handle_record_maybe_compressed(line: &[u8],
                                  tx: &SyncSender<Vec<u8>>,
                                  decoder: &Box<Decoder>,
                                  encoder: &Box<Encoder>)
                                  -> Result<(), &'static str> {
    if line.len() > 2 && line[0] == 0x78 &&
       (line[1] == 0x01 || line[1] == 0x9c || line[1] == 0xda) {
        let mut decompressed = Vec::with_capacity(MAX_UDP_PACKET_SIZE * MAX_COMPRESSION_RATIO);
        match Decompress::new(true).decompress_vec(line, &mut decompressed, Flush::Finish) {
            Ok(Status::Ok) |
            Ok(Status::StreamEnd) => handle_record(&decompressed, tx, decoder, encoder),
            _ => return Err("Corrupted compressed record"),
        }
    } else {
        handle_record(line, tx, decoder, encoder)
    }
}

fn handle_record(line: &[u8],
                 tx: &SyncSender<Vec<u8>>,
                 decoder: &Box<Decoder>,
                 encoder: &Box<Encoder>)
                 -> Result<(), &'static str> {
    let line = match str::from_utf8(line) {
        Err(_) => return Err("Invalid UTF-8 input"),
        Ok(line) => line,
    };
    let decoded = try!(decoder.decode(line));
    let reencoded = try!(encoder.encode(decoded));
    tx.send(reencoded).unwrap();
    Ok(())
}
