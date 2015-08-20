
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
        let listen = config.lookup("input.listen").map_or(DEFAULT_LISTEN, |x|x.as_str().
            expect("input.listen must be an ip:port string")).to_owned();
        TcpInput {
            listen: listen
        }
    }

    fn accept<TE>(&self, tx: SyncSender<Vec<u8>>, decoder: Box<Decoder + Send>, encoder: TE) where TE: Encoder + Clone + Send + 'static {
        let listener = TcpListener::bind(&self.listen as &str).unwrap();
        for client in listener.incoming() {
            match client {
                Ok(client) => {
                    let tx = tx.clone();
                    let (decoder, encoder) = (decoder.clone_boxed(), encoder.clone());
                    thread::spawn(move|| {
                        handle_client(client, tx, decoder, encoder);
                    });
                }
                Err(_) => { }
            }
        }
    }
}

fn handle_line<TE>(line: &String, tx: &SyncSender<Vec<u8>>, decoder: &Box<Decoder>, encoder: &TE) -> Result<(), &'static str> where TE: Encoder {
    let decoded = try!(decoder.decode(&line));
    let reencoded = try!(encoder.encode(decoded));
    tx.send(reencoded).unwrap();
    Ok(())
}

fn handle_client<TE>(client: TcpStream, tx: SyncSender<Vec<u8>>, decoder: Box<Decoder>, encoder: TE) where TE: Encoder {
    if let Ok(peer_addr) = client.peer_addr() {
        println!("New connection over TCP from [{}]", peer_addr);
    }
    let reader = BufReader::new(client);
    for line in reader.lines() {
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
