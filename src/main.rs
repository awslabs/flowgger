#[cfg(feature = "capnp-recompile")]
extern crate capnp;
extern crate chrono;
extern crate clap;
#[cfg(feature = "coroutines")]
extern crate coio;
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

mod flowgger;
#[cfg(feature = "capnp-recompile")]
pub use crate::flowgger::record_capnp;

use clap::{App, Arg};
use std::io::{stderr, Write};

const DEFAULT_CONFIG_FILE: &str = "flowgger.toml";
const FLOWGGER_VERSION_STRING: &str = "0.2.7";

fn main() {
    let matches = App::new("Flowgger")
        .version(FLOWGGER_VERSION_STRING)
        .about("A fast, simple and lightweight data collector")
        .arg(
            Arg::with_name("config_file")
                .help("Configuration file")
                .value_name("FILE")
                .index(1),
        )
        .get_matches();
    let config_file = matches
        .value_of("config_file")
        .unwrap_or(DEFAULT_CONFIG_FILE);
    let _ = writeln!(stderr(), "Flowgger {}", FLOWGGER_VERSION_STRING);
    flowgger::start(config_file)
}
