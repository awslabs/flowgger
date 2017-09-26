extern crate capnp;
extern crate chrono;
extern crate clap;
#[cfg(feature = "coroutines")]
extern crate coio;
extern crate flate2;
#[cfg(feature = "kafka")]
extern crate kafka;
extern crate openssl;
extern crate rand;
extern crate redis;
extern crate serde_json;
extern crate toml;

mod flowgger;
pub use flowgger::record_capnp;

use clap::{App, Arg};
use std::io::{stderr, Write};

const DEFAULT_CONFIG_FILE: &'static str = "flowgger.toml";
const FLOWGGER_VERSION_STRING: &'static str = "0.2.6";

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
