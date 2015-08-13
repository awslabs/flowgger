#![allow(dead_code, unused_variables, option_unwrap_used)]
#![feature(custom_derive, plugin)]
#![plugin(clippy)]

#[macro_use]
extern crate kafka;
extern crate log;
extern crate openssl;
extern crate toml;

mod flowgger;

const DEFAULT_CONFIG_FILE: &'static str = "flowgger.toml";

fn main() {
    let config_file = std::env::args().skip(1).next().unwrap_or(DEFAULT_CONFIG_FILE.to_owned());
    flowgger::start(&config_file);
}
