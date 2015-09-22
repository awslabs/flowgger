mod config;
mod decoder;
mod encoder;
mod input;
mod output;
mod record;
mod splitter;

use self::config::Config;
use self::decoder::Decoder;
use self::decoder::gelf_decoder::GelfDecoder;
use self::decoder::ltsv_decoder::LTSVDecoder;
use self::decoder::rfc5424_decoder::RFC5424Decoder;
use self::encoder::Encoder;
use self::encoder::gelf_encoder::GelfEncoder;
use self::input::Input;
use self::input::redis_input::RedisInput;
use self::input::stdin_input::StdinInput;
use self::input::tcp_input::TcpInput;
use self::input::tls_input::TlsInput;
#[cfg(feature = "coroutines")]
use self::input::tlsco_input::TlsCoInput;
use self::input::udp_input::UdpInput;
use self::output::Output;
use self::output::debug_output::DebugOutput;
use self::output::kafka_output::KafkaOutput;
use std::sync::mpsc::{sync_channel, SyncSender, Receiver};
use std::sync::{Arc, Mutex};

const DEFAULT_INPUT_FORMAT: &'static str = "rfc5424";
const DEFAULT_INPUT_TYPE: &'static str = "syslog-tls";
const DEFAULT_OUTPUT_TYPE: &'static str = "kafka";
const DEFAULT_QUEUE_SIZE: usize = 10_000_000;

#[cfg(feature = "coroutines")]
fn get_input_tlsco(config: &Config) -> Box<Input> {
    Box::new(TlsCoInput::new(&config)) as Box<Input>
}

#[cfg(not(feature = "coroutines"))]
fn get_input_tlsco(_config: &Config) -> ! {
    panic!("Support for coroutines is not compiled in")
}

fn get_input(input_type: &str, config: &Config) -> Box<Input> {
    match input_type {
        "redis" => Box::new(RedisInput::new(&config)) as Box<Input>,
        "stdin" => Box::new(StdinInput::new(&config)) as Box<Input>,
        "tcp" | "syslog-tcp" => Box::new(TcpInput::new(&config)) as Box<Input>,
        "tls" | "syslog-tls" => Box::new(TlsInput::new(&config)) as Box<Input>,
        "tls_co" | "tlsco" | "syslog-tls_co" | "syslog-tlsco" => get_input_tlsco(&config),
        "udp" => Box::new(UdpInput::new(&config)) as Box<Input>,
        _ => panic!("Invalid input type: {}", input_type)
    }
}

pub fn start(config_file: &str) {
    let config = match Config::from_path(config_file) {
        Ok(config) => config,
        Err(_) => panic!("Unable to read the config file [{}]", config_file)
    };
    let input_format = config.lookup("input.format").
        map_or(DEFAULT_INPUT_FORMAT, |x| x.as_str().expect("input.format must be a string"));
    let input_type = config.lookup("input.type").
        map_or(DEFAULT_INPUT_TYPE, |x| x.as_str().expect("input.type must be a string"));
    let input = get_input(input_type, &config);
    let decoder = match input_format {
        "rfc5424" => Box::new(RFC5424Decoder::new(&config)) as Box<Decoder + Send>,
        "ltsv" => Box::new(LTSVDecoder::new(&config)) as Box<Decoder + Send>,
        "gelf" => Box::new(GelfDecoder::new(&config)) as Box<Decoder + Send>,
        _ => panic!("Unknown input format: {}", input_format)
    };
    let encoder = Box::new(GelfEncoder::new(&config)) as Box<Encoder + Send>;
    let output_type = config.lookup("output.type").
        map_or(DEFAULT_OUTPUT_TYPE, |x| x.as_str().expect("output.type must be a string"));
    let output = match output_type {
        "debug" => Box::new(DebugOutput::new(&config)) as Box<Output>,
        "kafka" => Box::new(KafkaOutput::new(&config)) as Box<Output>,
        _ => panic!("Invalid output type: {}", output_type)
    };

    let queue_size = config.lookup("input.queuesize").
        map_or(DEFAULT_QUEUE_SIZE, |x| x.as_integer().
        expect("input.queuesize must be a size integer") as usize);
    let (tx, rx): (SyncSender<Vec<u8>>, Receiver<Vec<u8>>) = sync_channel(queue_size);
    let arx = Arc::new(Mutex::new(rx));

    output.start(arx);
    input.accept(tx, decoder, encoder);
}
