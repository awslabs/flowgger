use flowgger::config::Config;
use flowgger::decoder::Decoder;
use flowgger::encoder::Encoder;
use flowgger::splitter::Splitter;
use flowgger::splitter::line_splitter::LineSplitter;
use std::io::BufReader;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::SyncSender;
use std::thread;
use super::Input;

const DEFAULT_LISTEN: &'static str = "0.0.0.0:6514";

pub struct TcpInput {
    listen: String
}

impl TcpInput {
    pub fn new(config: &Config) -> TcpInput {
        let listen = config.lookup("input.listen").map_or(DEFAULT_LISTEN, |x|x.as_str().
            expect("input.listen must be an ip:port string")).to_owned();
        TcpInput {
            listen: listen
        }
    }
}

impl Input for TcpInput {
    fn accept(&self, tx: SyncSender<Vec<u8>>, decoder: Box<Decoder + Send>, encoder: Box<Encoder + Send>) {
        let listener = TcpListener::bind(&self.listen as &str).unwrap();
        for client in listener.incoming() {
            match client {
                Ok(client) => {
                    let tx = tx.clone();
                    let (decoder, encoder) = (decoder.clone_boxed(), encoder.clone_boxed());
                    thread::spawn(move|| {
                        handle_client(client, tx, decoder, encoder);
                    });
                }
                Err(_) => { }
            }
        }
    }
}

fn handle_client(client: TcpStream, tx: SyncSender<Vec<u8>>, decoder: Box<Decoder>, encoder: Box<Encoder>) {
    if let Ok(peer_addr) = client.peer_addr() {
        println!("Connection over TCP from [{}]", peer_addr);
    }
    let reader = BufReader::new(client);
    let splitter = Box::new(LineSplitter::new(tx, decoder, encoder)) as Box<Splitter<_>>;
    splitter.run(reader);
}
