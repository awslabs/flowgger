mod kafkapool;
mod rfc5424;
mod gelf;
mod config;
mod record;

use std::io::{BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{sync_channel, SyncSender, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;
use self::config::Config;
use self::kafkapool::KafkaPool;
use self::rfc5424::RFC5424;
use self::gelf::Gelf;
use self::record::Record;

const DEFAULT_QUEUE_SIZE: usize = 10_000_000;
const DEFAULT_LISTEN: &'static str = "0.0.0.0:6514";

pub trait Decoder {
    fn new() -> Self;
    fn decode(&self, line: &str) -> Result<Record, &'static str>;
}

pub trait Encoder {
    fn new(config: &Config) -> Self;
    fn encode(&self, record: Record) -> Result<Vec<u8>, &'static str>;
}

fn handle_client<TD: Decoder, TE: Encoder>(client: TcpStream, tx: SyncSender<Vec<u8>>, decoder: TD, encoder: TE) {
    let reader = BufReader::new(client);
    let mut counter = 0;
    for line in reader.lines() {
        let line = match line {
            Err(_) => return,
            Ok(line) => line
        };
        let decoded = match decoder.decode(&line) {
            Err(e) => { debug!("{}", e) ; continue },
            Ok(res) => res
        };
        let reencoded = match encoder.encode(decoded) {
            Err(e) => { debug!("{}", e) ; continue },
            Ok(reencoded) => reencoded
        };
        tx.send(reencoded).unwrap();
        counter = counter + 1;
        if counter % 250_000 == 0 {
            println!("Counter={}", counter);
        }
    }
}

pub fn main() {
    let config = match Config::from_path("flowgger.toml") {
        Ok(config) => config,
        Err(e) => {
            println!("{}", e);
            return
        }
    };
    let line = "\u{feff}<23>1 2015-08-05T15:53:45.637824Z testhostname appname 69 42 [origin@123 software=\"te\\st sc\\\"ript\" swVersion=\"0.0.1\"] test message";
    println!("{}", line);

    let decoder = RFC5424::new();
    let encoder = Gelf::new(&config);

    let queue_size = config.lookup("input.queuesize").
        map_or(DEFAULT_QUEUE_SIZE, |x| x.as_integer().unwrap() as usize);

    let (tx, rx): (SyncSender<Vec<u8>>, Receiver<Vec<u8>>) = sync_channel(queue_size);
    let arx = Arc::new(Mutex::new(rx));
    KafkaPool::new(arx, &config);
    let listen = config.lookup("input.listen").map_or(DEFAULT_LISTEN, |x| x.as_str().unwrap());
    let listener = TcpListener::bind(listen).unwrap();

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
