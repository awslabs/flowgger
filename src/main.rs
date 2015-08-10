#![allow(dead_code, unused_variables)]
#![feature(custom_derive, plugin)]

#[macro_use]
extern crate log;
extern crate toml;
extern crate kafka;

mod flowgger;

fn main() {
    flowgger::main();
}
