mod kafkapool;
mod rfc5424;
mod gelf;
mod config;
mod record;
mod tcpinput;

use std::sync::mpsc::{sync_channel, SyncSender, Receiver};
use std::sync::{Arc, Mutex};
use self::config::Config;
use self::kafkapool::KafkaPool;
use self::rfc5424::RFC5424;
use self::gelf::Gelf;
use self::record::Record;
use self::tcpinput::TcpInput;

const DEFAULT_QUEUE_SIZE: usize = 10_000_000;
const DEFAULT_CONFIG_FILE: &'static str = "flowgger.toml";

pub trait Input {
    fn new() -> Self;
    fn accept<TD, TE>(&self, config: Config, tx: SyncSender<Vec<u8>>, decoder: TD, encoder: TE) where TD: Decoder + Clone + Send + 'static, TE: Encoder + Clone + Send + 'static;
}

pub trait Decoder {
    fn new() -> Self;
    fn decode(&self, line: &str) -> Result<Record, &'static str>;
}

pub trait Encoder {
    fn new(config: &Config) -> Self;
    fn encode(&self, record: Record) -> Result<Vec<u8>, &'static str>;
}

pub trait Output {
    fn new() -> Self;
    fn start(&self, arx: Arc<Mutex<Receiver<Vec<u8>>>>, config: &Config);
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

    let input = TcpInput::new();
    let decoder = RFC5424::new();
    let encoder = Gelf::new(&config);
    let output = KafkaPool::new();

    let queue_size = config.lookup("input.queuesize").
        map_or(DEFAULT_QUEUE_SIZE, |x| x.as_integer().unwrap() as usize);

    let (tx, rx): (SyncSender<Vec<u8>>, Receiver<Vec<u8>>) = sync_channel(queue_size);
    let arx = Arc::new(Mutex::new(rx));
    output.start(arx, &config);
    input.accept(config, tx, decoder, encoder);
}
