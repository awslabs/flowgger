#![allow(dead_code, unused_variables)]
#![feature(custom_derive, plugin)]

#[macro_use]
extern crate kafka;
extern crate log;
extern crate openssl;
extern crate toml;

mod flowgger;

const DEFAULT_CONFIG_FILE: &'static str = "flowgger.toml";

fn main() {
    let config_file = std::env::args().skip(1).next().unwrap_or(DEFAULT_CONFIG_FILE.to_string());
    flowgger::start(&config_file);
}
