
use flowgger::config::Config;
use flowgger::{Decoder, Encoder, Input};
use std::io::{stderr, Write, BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::SyncSender;
use std::thread;

const DEFAULT_LISTEN: &'static str = "0.0.0.0:6514";

pub struct TcpInput {
    listen: String
}

impl Input for TcpInput {
    fn new(config: &Config) -> TcpInput {
        let listen = config.lookup("input.listen").map_or(DEFAULT_LISTEN, |x| x.as_str().unwrap()).to_string();
        TcpInput {
            listen: listen
        }
    }

    fn accept<TD, TE>(&self, tx: SyncSender<Vec<u8>>, decoder: TD, encoder: TE) where TD: Decoder + Clone + Send + 'static, TE: Encoder + Clone + Send + 'static {
        let listener = TcpListener::bind(&self.listen as &str).unwrap();
        for client in listener.incoming() {
            match client {
                Ok(client) => {
                    let tx = tx.clone();
                    let (decoder, encoder) = (decoder.clone(), encoder.clone());
                    thread::spawn(move|| {
                        handle_client(client, tx, decoder, encoder);
                    });
                }
                Err(_) => { }
            }
        }
    }
}

fn handle_line<TD, TE>(line: &String, tx: &SyncSender<Vec<u8>>, decoder: &TD, encoder: &TE) -> Result<(), &'static str> where TD: Decoder, TE: Encoder {
    let decoded = try!(decoder.decode(&line));
    let reencoded = try!(encoder.encode(decoded));
    tx.send(reencoded).unwrap();
    Ok(())
}

fn handle_client<TD, TE>(client: TcpStream, tx: SyncSender<Vec<u8>>, decoder: TD, encoder: TE) where TD: Decoder, TE: Encoder {
    let reader = BufReader::new(client);
    for line in reader.lines() {
        let line = match line {
            Err(_) => return,
            Ok(line) => line
        };
        match handle_line(&line, &tx, &decoder, &encoder) {
            Err(e) => { let _ = writeln!(stderr(), "{}: [{}]", e, line.trim()); }
            _ => { }
        }
    }
}
