#![allow(dead_code, unused_variables)]
#![feature(custom_derive, plugin)]

#[macro_use]
extern crate kafka;
extern crate log;
extern crate toml;

mod flowgger;

fn main() {
    flowgger::main();
}
