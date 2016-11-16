#[macro_use]

extern crate capnp;
extern crate chrono;
#[cfg(feature = "coroutines")]
extern crate coio;
extern crate flate2;
#[cfg(not(feature = "without_kafka"))]
extern crate kafka;
extern crate log;
extern crate openssl;
extern crate rand;
extern crate redis;
extern crate serde;
extern crate serde_json;
extern crate toml;

#[macro_use]
extern crate clap;
use clap::{Arg, App};

mod flowgger;
pub use flowgger::record_capnp;

const DEFAULT_CONFIG_FILE: &'static str = "flowgger.toml";
const FLOWGGER_VERSION_STRING: &'static str = "0.2.3";

fn main() {
    let matches = App::new("Flowgger")
        .version(FLOWGGER_VERSION_STRING)
        .about("A fast, simple and lightweight data collector")
        .arg(Arg::with_name("config_file").help("Configuration file").value_name("FILE").index(1))
        .get_matches();
    let config_file = matches.value_of("config_file").unwrap_or(DEFAULT_CONFIG_FILE);
    println!("Flowgger {}", FLOWGGER_VERSION_STRING);
    flowgger::start(config_file)
}
