mod config;
mod gelf;
mod kafkapool;
mod record;
mod rfc5424;
mod tcpinput;

use self::config::Config;
use self::gelf::Gelf;
use self::kafkapool::KafkaPool;
use self::record::Record;
use self::rfc5424::RFC5424;
use self::tcpinput::TcpInput;
use std::sync::mpsc::{sync_channel, SyncSender, Receiver};
use std::sync::{Arc, Mutex};

const DEFAULT_QUEUE_SIZE: usize = 10_000_000;
const DEFAULT_CONFIG_FILE: &'static str = "flowgger.toml";

pub trait Input {
    fn new(config: &Config) -> Self;
    fn accept<TD, TE>(&self, tx: SyncSender<Vec<u8>>, decoder: TD, encoder: TE) where TD: Decoder + Clone + Send + 'static, TE: Encoder + Clone + Send + 'static;
}

pub trait Decoder {
    fn new(config: &Config) -> Self;
    fn decode(&self, line: &str) -> Result<Record, &'static str>;
}

pub trait Encoder {
    fn new(config: &Config) -> Self;
    fn encode(&self, record: Record) -> Result<Vec<u8>, &'static str>;
}

pub trait Output {
    fn new(config: &Config) -> Self;
    fn start(&self, arx: Arc<Mutex<Receiver<Vec<u8>>>>);
}

pub fn main() {
    let config = match Config::from_path(DEFAULT_CONFIG_FILE) {
        Ok(config) => config,
        Err(e) => {
            println!("{}", e);
            return
        }
    };
    let line = "\u{feff}<23>1 2015-08-05T15:53:45.637824Z testhostname appname 69 42 [origin@123 software=\"te\\st sc\\\"ript\" swVersion=\"0.0.1\"] test message";
    println!("{}", line);

    let input = TcpInput::new(&config);
    let decoder = RFC5424::new(&config);
    let encoder = Gelf::new(&config);
    let output = KafkaPool::new(&config);

    let queue_size = config.lookup("input.queuesize").
        map_or(DEFAULT_QUEUE_SIZE, |x| x.as_integer().unwrap() as usize);

    let (tx, rx): (SyncSender<Vec<u8>>, Receiver<Vec<u8>>) = sync_channel(queue_size);
    let arx = Arc::new(Mutex::new(rx));
    output.start(arx);
    input.accept(tx, decoder, encoder);
}
