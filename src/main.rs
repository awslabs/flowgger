#[macro_use]

extern crate capnp;
extern crate chrono;
#[cfg(feature = "coroutines")]
extern crate coio;
extern crate kafka;
extern crate log;
extern crate openssl;
extern crate redis;
extern crate serde;
extern crate serde_json;
extern crate toml;

mod flowgger;
pub use flowgger::record_capnp;

const DEFAULT_CONFIG_FILE: &'static str = "flowgger.toml";
const FLOWGGER_VERSION_STRING: &'static str = "0.1.7";

fn main() {
    println!("Flowgger v{}", FLOWGGER_VERSION_STRING);
    let config_file = std::env::args().skip(1).next().unwrap_or(DEFAULT_CONFIG_FILE.to_owned());
    flowgger::start(&config_file);
}
