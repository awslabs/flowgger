mod config;
mod decoder;
mod encoder;
mod input;
mod merger;
mod output;
mod record;
mod splitter;
mod utils;

pub mod record_capnp;

use self::config::Config;
use self::decoder::{Decoder, GelfDecoder, InvalidDecoder, LTSVDecoder, RFC5424Decoder};
use self::encoder::{CapnpEncoder, Encoder, GelfEncoder, LTSVEncoder};
use self::input::{Input, RedisInput, StdinInput, TcpInput, TlsInput, UdpInput};
#[cfg(feature = "coroutines")]
use self::input::{TcpCoInput, TlsCoInput};
use self::merger::{LineMerger, Merger, NulMerger, SyslenMerger};
use self::output::{DebugOutput, Output, TlsOutput};
#[cfg(feature = "kafka")]
use self::output::KafkaOutput;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};

const DEFAULT_INPUT_FORMAT: &'static str = "rfc5424";
const DEFAULT_INPUT_TYPE: &'static str = "syslog-tls";
const DEFAULT_OUTPUT_FORMAT: &'static str = "gelf";
const DEFAULT_OUTPUT_FRAMING: &'static str = "noop";
#[cfg(feature = "kafka")]
const DEFAULT_OUTPUT_TYPE: &'static str = "kafka";
#[cfg(not(feature = "kafka"))]
const DEFAULT_OUTPUT_TYPE: &'static str = "tls";
const DEFAULT_QUEUE_SIZE: usize = 10_000_000;

#[cfg(feature = "coroutines")]
fn get_input_tlsco(config: &Config) -> Box<Input> {
    Box::new(TlsCoInput::new(&config)) as Box<Input>
}

#[cfg(not(feature = "coroutines"))]
fn get_input_tlsco(_config: &Config) -> ! {
    panic!("Support for coroutines is not compiled in")
}

#[cfg(feature = "coroutines")]
fn get_input_tcpco(config: &Config) -> Box<Input> {
    Box::new(TcpCoInput::new(&config)) as Box<Input>
}

#[cfg(not(feature = "coroutines"))]
fn get_input_tcpco(_config: &Config) -> ! {
    panic!("Support for coroutines is not compiled in")
}

fn get_input(input_type: &str, config: &Config) -> Box<Input> {
    match input_type {
        "redis" => Box::new(RedisInput::new(&config)) as Box<Input>,
        "stdin" => Box::new(StdinInput::new(&config)) as Box<Input>,
        "tcp" | "syslog-tcp" => Box::new(TcpInput::new(&config)) as Box<Input>,
        "tcp_co" | "tcpco" | "syslog-tcp_co" | "syslog-tcpco" => get_input_tcpco(&config),
        "tls" | "syslog-tls" => Box::new(TlsInput::new(&config)) as Box<Input>,
        "tls_co" | "tlsco" | "syslog-tls_co" | "syslog-tlsco" => get_input_tlsco(&config),
        "udp" => Box::new(UdpInput::new(&config)) as Box<Input>,
        _ => panic!("Invalid input type: {}", input_type),
    }
}

#[cfg(feature = "kafka")]
fn get_output_kafka(config: &Config) -> Box<Output> {
    Box::new(KafkaOutput::new(&config)) as Box<Output>
}

#[cfg(not(feature = "kafka"))]
fn get_output_kafka(_config: &Config) -> ! {
    panic!("Support for Kafka hasn't been compiled in")
}

fn get_output(output_type: &str, config: &Config) -> Box<Output> {
    match output_type {
        "stdout" | "debug" => Box::new(DebugOutput::new(&config)) as Box<Output>,
        "kafka" => get_output_kafka(&config),
        "tls" | "syslog-tls" => Box::new(TlsOutput::new(&config)) as Box<Output>,
        _ => panic!("Invalid output type: {}", output_type),
    }
}

pub fn start(config_file: &str) {
    let config = match Config::from_path(config_file) {
        Ok(config) => config,
        Err(e) => panic!(
            "Unable to read the config file [{}]: {}",
            config_file,
            e.description()
        ),
    };
    let input_format = config
        .lookup("input.format")
        .map_or(DEFAULT_INPUT_FORMAT, |x| {
            x.as_str().expect("input.format must be a string")
        });
    let input_type = config.lookup("input.type").map_or(DEFAULT_INPUT_TYPE, |x| {
        x.as_str().expect("input.type must be a string")
    });
    let input = get_input(input_type, &config);
    let decoder = match input_format {
        _ if input_format == "capnp" => {
            Box::new(InvalidDecoder::new(&config)) as Box<Decoder + Send>
        }
        "gelf" => Box::new(GelfDecoder::new(&config)) as Box<Decoder + Send>,
        "ltsv" => Box::new(LTSVDecoder::new(&config)) as Box<Decoder + Send>,
        "rfc5424" => Box::new(RFC5424Decoder::new(&config)) as Box<Decoder + Send>,
        _ => panic!("Unknown input format: {}", input_format),
    };

    let output_format = config
        .lookup("output.format")
        .map_or(DEFAULT_OUTPUT_FORMAT, |x| {
            x.as_str().expect("output.format must be a string")
        });
    let encoder = match output_format {
        "capnp" => Box::new(CapnpEncoder::new(&config)) as Box<Encoder + Send>,
        "gelf" | "json" => Box::new(GelfEncoder::new(&config)) as Box<Encoder + Send>,
        "ltsv" => Box::new(LTSVEncoder::new(&config)) as Box<Encoder + Send>,
        _ => panic!("Unknown output format: {}", output_format),
    };
    let output_type = config
        .lookup("output.type")
        .map_or(DEFAULT_OUTPUT_TYPE, |x| {
            x.as_str().expect("output.type must be a string")
        });
    let output = get_output(output_type, &config);
    let output_framing = match config.lookup("output.framing") {
        Some(framing) => framing.as_str().expect("output.framing must be a string"),
        None => match (output_format, output_type) {
            ("capnp", _) | (_, "kafka") => "noop",
            (_, "debug") | ("ltsv", _) => "line",
            ("gelf", _) => "nul",
            _ => DEFAULT_OUTPUT_FRAMING,
        },
    };
    let merger: Option<Box<Merger>> = match output_framing {
        "noop" | "nop" | "none" => None,
        "capnp" => None,
        "line" => Some(Box::new(LineMerger::new(&config)) as Box<Merger>),
        "nul" => Some(Box::new(NulMerger::new(&config)) as Box<Merger>),
        "syslen" => Some(Box::new(SyslenMerger::new(&config)) as Box<Merger>),
        _ => panic!("Invalid framing type: {}", output_framing),
    };
    let queue_size = config
        .lookup("input.queuesize")
        .map_or(DEFAULT_QUEUE_SIZE, |x| {
            x.as_integer()
                .expect("input.queuesize must be a size integer") as usize
        });
    let (tx, rx): (SyncSender<Vec<u8>>, Receiver<Vec<u8>>) = sync_channel(queue_size);
    let arx = Arc::new(Mutex::new(rx));

    output.start(arx, merger);
    input.accept(tx, decoder, encoder);
}
