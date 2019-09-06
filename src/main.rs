extern crate flowgger;

use clap::{App, Arg};
use std::io::{stderr, Write};

const DEFAULT_CONFIG_FILE: &str = "flowgger.toml";
const FLOWGGER_VERSION_STRING: &str = env!("CARGO_PKG_VERSION");

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
