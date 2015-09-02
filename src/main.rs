
#![feature(plugin)]
#![plugin(clippy)]

#[macro_use]
extern crate kafka;
extern crate log;
extern crate openssl;
extern crate toml;

mod flowgger;

const DEFAULT_CONFIG_FILE: &'static str = "flowgger.toml";
const FLOWGGER_VERSION_STRING: &'static str = "0.1.2";

fn main() {
    println!("Flowgger v{}", FLOWGGER_VERSION_STRING);
    let config_file = std::env::args().skip(1).next().unwrap_or(DEFAULT_CONFIG_FILE.to_owned());
    flowgger::start(&config_file);
}
