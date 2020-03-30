mod config;
mod decoder;
mod encoder;
mod input;
mod merger;
mod output;
mod record;
mod splitter;
mod utils;

#[cfg(feature = "capnp-recompile")]
extern crate capnp;
extern crate chrono;
extern crate chrono_tz;
extern crate clap;
extern crate flate2;
#[cfg(feature = "file")]
extern crate glob;
#[cfg(feature = "kafka-output")]
extern crate kafka;
#[cfg(feature = "file")]
extern crate notify;
#[cfg(feature = "tls")]
extern crate openssl;
extern crate rand;
#[cfg(feature = "redis-input")]
extern crate redis;
#[cfg(feature = "gelf")]
extern crate serde_json;
extern crate toml;

use self::config::Config;
#[cfg(feature = "gelf")]
use self::decoder::GelfDecoder;
#[cfg(feature = "ltsv")]
use self::decoder::LTSVDecoder;
#[cfg(feature = "rfc3164")]
use self::decoder::RFC3164Decoder;
#[cfg(feature = "rfc5424")]
use self::decoder::RFC5424Decoder;
use self::decoder::{Decoder, InvalidDecoder};
#[cfg(feature = "capnp-recompile")]
use self::encoder::CapnpEncoder;
use self::encoder::Encoder;
#[cfg(feature = "gelf")]
use self::encoder::GelfEncoder;
#[cfg(feature = "ltsv")]
use self::encoder::LTSVEncoder;
#[cfg(feature = "rfc3164")]
use self::encoder::RFC3164Encoder;
#[cfg(feature = "rfc5424")]
use self::encoder::RFC5424Encoder;
#[cfg(feature = "passthrough")]
use self::encoder::PassthroughEncoder;
#[cfg(feature = "file")]
use self::input::FileInput;
#[cfg(feature = "redis-input")]
use self::input::RedisInput;
#[cfg(feature = "tls")]
use self::input::TlsInput;
use self::input::{Input, StdinInput};
#[cfg(feature = "coroutines")]
use self::input::{TcpCoInput, TlsCoInput};
#[cfg(feature = "syslog")]
use self::input::{TcpInput, UdpInput};
use self::merger::{LineMerger, Merger, NulMerger, SyslenMerger};
#[cfg(feature = "file")]
use self::output::FileOutput;
#[cfg(feature = "kafka-output")]
use self::output::KafkaOutput;
#[cfg(feature = "tls")]
use self::output::TlsOutput;
use self::output::{DebugOutput, Output};
use std::error::Error;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::{Arc, Mutex};

const DEFAULT_INPUT_FORMAT: &str = "rfc5424";
const DEFAULT_INPUT_TYPE: &str = "syslog-tls";
const DEFAULT_OUTPUT_FORMAT: &str = "gelf";
const DEFAULT_OUTPUT_FRAMING: &str = "noop";
#[cfg(feature = "kafka-output")]
const DEFAULT_OUTPUT_TYPE: &str = "kafka";
#[cfg(not(feature = "kafka-output"))]
const DEFAULT_OUTPUT_TYPE: &str = "tls";
const DEFAULT_QUEUE_SIZE: usize = 10_000_000;

#[cfg(feature = "coroutines")]
fn get_input_tlsco(config: &Config) -> Box<dyn Input> {
    Box::new(TlsCoInput::new(&config)) as Box<dyn Input>
}

#[cfg(not(feature = "coroutines"))]
fn get_input_tlsco(_config: &Config) -> ! {
    panic!("Support for coroutines is not compiled in")
}

#[cfg(feature = "coroutines")]
fn get_input_tcpco(config: &Config) -> Box<dyn Input> {
    Box::new(TcpCoInput::new(&config)) as Box<dyn Input>
}

#[cfg(not(feature = "coroutines"))]
fn get_input_tcpco(_config: &Config) -> ! {
    panic!("Support for coroutines is not compiled in")
}

#[cfg(feature = "redis-input")]
fn get_input_redis(config: &Config) -> Box<dyn Input> {
    Box::new(RedisInput::new(&config)) as Box<dyn Input>
}

#[cfg(not(feature = "redis-input"))]
fn get_input_redis(_config: &Config) -> ! {
    panic!("Support for redis is not compiled in")
}

#[cfg(feature = "tls")]
fn get_input_tls(config: &Config) -> Box<dyn Input> {
    Box::new(TlsInput::new(&config)) as Box<dyn Input>
}

#[cfg(not(feature = "tls"))]
fn get_input_tls(_config: &Config) -> ! {
    panic!("Support for tls is not compiled in")
}

#[cfg(feature = "syslog")]
fn get_input_tcp(config: &Config) -> Box<dyn Input> {
    Box::new(TcpInput::new(&config)) as Box<dyn Input>
}

#[cfg(not(feature = "syslog"))]
fn get_input_tcp(_config: &Config) -> ! {
    panic!("Support for syslog is not compiled in")
}

#[cfg(feature = "syslog")]
fn get_input_udp(config: &Config) -> Box<dyn Input> {
    Box::new(UdpInput::new(&config)) as Box<dyn Input>
}

#[cfg(not(feature = "syslog"))]
fn get_input_udp(_config: &Config) -> ! {
    panic!("Support for syslog is not compiled in")
}

#[cfg(feature = "file")]
fn get_input_file(config: &Config) -> Box<dyn Input> {
    Box::new(FileInput::new(&config)) as Box<dyn Input>
}

#[cfg(not(feature = "file"))]
fn get_input_file(_config: &Config) -> ! {
    panic!("Support for file is not compiled in")
}

fn get_input(input_type: &str, config: &Config) -> Box<dyn Input> {
    match input_type {
        "redis" => get_input_redis(config),
        "stdin" => Box::new(StdinInput::new(config)) as Box<dyn Input>,
        "tcp" | "syslog-tcp" => get_input_tcp(config),
        "tcp_co" | "tcpco" | "syslog-tcp_co" | "syslog-tcpco" => get_input_tcpco(config),
        "tls" | "syslog-tls" => get_input_tls(config),
        "tls_co" | "tlsco" | "syslog-tls_co" | "syslog-tlsco" => get_input_tlsco(config),
        "udp" => get_input_udp(config),
        "file" => get_input_file(config),
        _ => panic!("Invalid input type: {}", input_type),
    }
}

#[cfg(feature = "kafka-output")]
fn get_output_kafka(config: &Config) -> Box<dyn Output> {
    Box::new(KafkaOutput::new(config)) as Box<dyn Output>
}

#[cfg(not(feature = "kafka-output"))]
fn get_output_kafka(_config: &Config) -> ! {
    panic!("Support for Kafka hasn't been compiled in")
}

#[cfg(feature = "file")]
fn get_output_file(config: &Config) -> Box<dyn Output> {
    Box::new(FileOutput::new(config)) as Box<dyn Output>
}

#[cfg(not(feature = "file"))]
fn get_output_file(_config: &Config) -> ! {
    panic!("Support for file hasn't been compiled in")
}

#[cfg(feature = "tls")]
fn get_output_tls(config: &Config) -> Box<dyn Output> {
    Box::new(TlsOutput::new(config)) as Box<dyn Output>
}

#[cfg(not(feature = "tls"))]
fn get_output_tls(_config: &Config) -> ! {
    panic!("Support for tls hasn't been compiled in")
}

fn get_output(output_type: &str, config: &Config) -> Box<dyn Output> {
    match output_type {
        "stdout" | "debug" => Box::new(DebugOutput::new(config)) as Box<dyn Output>,
        "kafka" => get_output_kafka(config),
        "tls" | "syslog-tls" => get_output_tls(config),
        "file" => get_output_file(config),
        _ => panic!("Invalid output type: {}", output_type),
    }
}

#[cfg(feature = "capnp-recompile")]
fn get_capnp_encoder(config: &Config) -> Box<dyn Encoder + Send> {
    Box::new(CapnpEncoder::new(config)) as Box<dyn Encoder + Send>
}

#[cfg(not(feature = "capnp-recompile"))]
fn get_capnp_encoder(_config: &Config) -> ! {
    panic!("Support for CapNProto hasn't been compiled in")
}

#[cfg(feature = "gelf")]
fn get_gelf_encoder(config: &Config) -> Box<dyn Encoder + Send> {
    Box::new(GelfEncoder::new(config)) as Box<dyn Encoder + Send>
}

#[cfg(not(feature = "gelf"))]
fn get_gelf_encoder(_config: &Config) -> ! {
    panic!("Support for Gelf hasn't been compiled in")
}

#[cfg(feature = "gelf")]
fn get_gelf_decoder(config: &Config) -> Box<dyn Decoder + Send> {
    Box::new(GelfDecoder::new(config)) as Box<dyn Decoder + Send>
}

#[cfg(not(feature = "gelf"))]
fn get_gelf_decoder(_config: &Config) -> ! {
    panic!("Support for Gelf hasn't been compiled in")
}

#[cfg(feature = "ltsv")]
fn get_ltvs_encoder(config: &Config) -> Box<dyn Encoder + Send> {
    Box::new(LTSVEncoder::new(config)) as Box<dyn Encoder + Send>
}

#[cfg(not(feature = "ltsv"))]
fn get_ltvs_encoder(_config: &Config) -> ! {
    panic!("Support for Gelf hasn't been compiled in")
}

#[cfg(feature = "ltsv")]
fn get_ltvs_decoder(config: &Config) -> Box<dyn Decoder + Send> {
    Box::new(LTSVDecoder::new(config)) as Box<dyn Decoder + Send>
}

#[cfg(not(feature = "ltsv"))]
fn get_ltvs_decoder(_config: &Config) -> ! {
    panic!("Support for Gelf hasn't been compiled in")
}

#[cfg(feature = "rfc5424")]
fn get_decoder_rfc5424(config: &Config) -> Box<dyn Decoder + Send> {
    Box::new(RFC5424Decoder::new(config)) as Box<dyn Decoder + Send>
}

#[cfg(feature = "rfc5424")]
fn get_encoder_rfc5424(config: &Config) -> Box<dyn Encoder + Send> {
    Box::new(RFC5424Encoder::new(config)) as Box<dyn Encoder + Send>
}

#[cfg(feature = "passthrough")]
fn get_encoder_passthrough(config: &Config) -> Box<dyn Encoder + Send> {
    Box::new(PassthroughEncoder::new(config)) as Box<dyn Encoder + Send>
}

#[cfg(feature = "rfc3164")]
fn get_decoder_rfc3164(config: &Config) -> Box<dyn Decoder + Send> {
    Box::new(RFC3164Decoder::new(config)) as Box<dyn Decoder + Send>
}

#[cfg(feature = "rfc3164")]
fn get_encoder_rfc3164(config: &Config) -> Box<dyn Encoder + Send> {
    Box::new(RFC3164Encoder::new(config)) as Box<dyn Encoder + Send>
}

#[cfg(not(feature = "rfc5424"))]
fn get_decoder_rfc5424(_config: &Config) -> ! {
    panic!("Support for rfc5424 hasn't been compiled in")
}

#[cfg(not(feature = "rfc3164"))]
fn get_decoder_rfc3164(_config: &Config) -> ! {
    panic!("Support for rfc3164 hasn't been compiled in")
}

#[cfg(not(feature = "rfc3164"))]
fn get_encoder_rfc3164(_config: &Config) -> ! {
    panic!("Support for rfc3164 hasn't been compiled in")
}

#[cfg(not(feature = "rfc3164"))]
fn get_encoder_rfc5424(_config: &Config) -> ! {
    panic!("Support for rfc3164 hasn't been compiled in")
}

#[cfg(not(feature = "passthrough"))]
fn get_encoder_passthrough(_config: &Config) -> ! {
    panic!("Support for passthrough hasn't been compiled in")
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
            Box::new(InvalidDecoder::new(&config)) as Box<dyn Decoder + Send>
        }
        "gelf" => get_gelf_decoder(&config),
        "ltsv" => get_ltvs_decoder(&config),
        "rfc5424" => get_decoder_rfc5424(&config),
        "rfc3164" => get_decoder_rfc3164(&config),
        _ => panic!("Unknown input format: {}", input_format),
    };

    let output_format = config
        .lookup("output.format")
        .map_or(DEFAULT_OUTPUT_FORMAT, |x| {
            x.as_str().expect("output.format must be a string")
        });
    let encoder = match output_format {
        "capnp" => get_capnp_encoder(&config),
        "gelf" | "json" => get_gelf_encoder(&config),
        "ltsv" => get_ltvs_encoder(&config),
        "rfc3164" => get_encoder_rfc3164(&config),
        "rfc5424" => get_encoder_rfc5424(&config),
        "passthrough" => get_encoder_passthrough(&config),
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
    let merger: Option<Box<dyn Merger>> = match output_framing {
        "noop" | "nop" | "none" => None,
        "capnp" => None,
        "line" => Some(Box::new(LineMerger::new(&config)) as Box<dyn Merger>),
        "nul" => Some(Box::new(NulMerger::new(&config)) as Box<dyn Merger>),
        "syslen" => Some(Box::new(SyslenMerger::new(&config)) as Box<dyn Merger>),
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
