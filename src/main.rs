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
use clap::App;

mod flowgger;
pub use flowgger::record_capnp;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    // Calling .unwrap() is safe here because "INPUT" is required (if "INPUT" wasn't
    // required we could have used an 'if let' to conditionally get the value)
    println!("Using input file: {}", matches.value_of("INPUT").unwrap());

    let config_file = matches.value_of("INPUT").unwrap();

    flowgger::start(config_file)

}
