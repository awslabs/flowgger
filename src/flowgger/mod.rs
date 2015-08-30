mod config;
mod decoder;
mod gelf_encoder;
mod kafkapool;
mod record;
mod tcpinput;
mod tlsinput;

use self::config::Config;
use self::decoder::Decoder;
use self::decoder::gelf_decoder::GelfDecoder;
use self::decoder::ltsv_decoder::LTSVDecoder;
use self::decoder::rfc5424_decoder::RFC5424Decoder;
use self::gelf_encoder::GelfEncoder;
use self::kafkapool::KafkaPool;
use self::record::Record;
use self::tcpinput::TcpInput;
use self::tlsinput::TlsInput;
use std::sync::mpsc::{sync_channel, SyncSender, Receiver};
use std::sync::{Arc, Mutex};

const DEFAULT_INPUT_FORMAT: &'static str = "rfc5424";
const DEFAULT_INPUT_TYPE: &'static str = "syslog-tls";
const DEFAULT_QUEUE_SIZE: usize = 10_000_000;

pub trait Input {
    fn new(config: &Config) -> Self;
    fn accept<TE>(&self, tx: SyncSender<Vec<u8>>, decoder: Box<Decoder + Send>, encoder: TE) where TE: Encoder + Clone + Send + 'static;
}

pub trait Encoder {
    fn new(config: &Config) -> Self;
    fn encode(&self, record: Record) -> Result<Vec<u8>, &'static str>;
}

pub trait Output {
    fn new(config: &Config) -> Self;
    fn start(&self, arx: Arc<Mutex<Receiver<Vec<u8>>>>);
}

pub fn start(config_file: &str) {
    let config = match Config::from_path(config_file) {
        Ok(config) => config,
        Err(_) => panic!("Unable to read the config file [{}]", config_file)
    };
    let input_format = config.lookup("input.format").
        map_or(DEFAULT_INPUT_FORMAT, |x| x.as_str().expect("input.format must be a string"));
    let decoder = match input_format {
        "rfc5424" => Box::new(RFC5424Decoder::new(&config)) as Box<Decoder + Send>,
        "ltsv" => Box::new(LTSVDecoder::new(&config)) as Box<Decoder + Send>,
        "gelf" => Box::new(GelfDecoder::new(&config)) as Box<Decoder + Send>,
        _ => panic!("Unknown input format: {}", input_format)
    };
    let encoder = GelfEncoder::new(&config);
    let output = KafkaPool::new(&config);

    let queue_size = config.lookup("input.queuesize").
        map_or(DEFAULT_QUEUE_SIZE, |x| x.as_integer().
        expect("input.queuesize must be a size integer") as usize);

    let (tx, rx): (SyncSender<Vec<u8>>, Receiver<Vec<u8>>) = sync_channel(queue_size);
    let arx = Arc::new(Mutex::new(rx));
    output.start(arx);

    let input_type = config.lookup("input.type").
        map_or(DEFAULT_INPUT_TYPE, |x| x.as_str().expect("input.type must be a string"));
    match input_type {
        "syslog-tcp" => TcpInput::new(&config).accept(tx, decoder, encoder),
        "syslog-tls" => TlsInput::new(&config).accept(tx, decoder, encoder),
        _ => panic!("Invalid input type: {}", input_type)
    }
}
