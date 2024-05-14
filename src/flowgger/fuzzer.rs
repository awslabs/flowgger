extern crate quickcheck;

use crate::flowgger;

use quickcheck::QuickCheck;

use std::sync::mpsc::{Receiver, sync_channel, SyncSender};
use std::sync::{Arc, Mutex};
use std::{fs, thread, time};
use std::io::{BufReader, BufRead};

use flowgger::config::Config;
use flowgger::encoder::Encoder;
use flowgger::decoder::Decoder;
use flowgger::merger;
use flowgger::output;

use flowgger::get_decoder_rfc3164;
use flowgger::get_encoder_rfc3164;
use flowgger::input::udp_input::handle_record_maybe_compressed;

use self::merger::{LineMerger, Merger};
use self::output::FileOutput;
use self::output::Output;
use toml::Value;
use lazy_static::lazy_static;


/// Fuzz testing logic defined in this module
/// This module depends on the default configuration file `flowgger.toml`
/// 
/// The test in this module hits flowgger with random input. 
/// The expected state is a failure for invalid inputs (no logs sent to output) 
/// and successfully parsed logs written to output stream for valid inputs
/// 
/// # Dependencies
/// It depend on the external crates [`QuickCheck`][https://docs.rs/quickcheck/latest/quickcheck/] and 
/// [`LazyStatic`][https://docs.rs/lazy_static/latest/lazy_static/]
/// 
/// `QuickCheck`is used to generate random string input, while LazyStatic is used to lazily intialize share resources at runtime
/// 
/// # Errors
///
/// This function will return an error if the default config does not exists,is unreadbale, or is not valid
/// toml format
#[cfg(test)]
mod tests {
    use super::*;

    const DEFAULT_CONFIG_FILE: &str = "flowgger.toml";
    const DEFAULT_OUTPUT_FILEPATH: &str = "output.log";
    const DEFAULT_QUEUE_SIZE: usize = 10_000_000;

    const DEFAULT_OUTPUT_FORMAT: &str = "gelf";
    const DEFAULT_OUTPUT_FRAMING: &str = "noop";
    const DEFAULT_OUTPUT_TYPE: &str = "file";

    const DEFAULT_FUZZED_MESSAGE_COUNT: u64 = 100;

    lazy_static! {
        static ref STATIC_CONFIG: Config = get_config();
        static ref SYNC_SENDER:Mutex<Option<SyncSender<Vec<u8>>>> = Mutex::new(None);
    }

    #[test]
    fn test_fuzzer(){
        let config = STATIC_CONFIG.clone();
        let file_output_path = config.lookup("output.file_path").map_or(DEFAULT_OUTPUT_FILEPATH, |x| {
            x.as_str().expect("File output path missing in config")
        });
        remove_output_file(&file_output_path);

        let (tx, rx): (SyncSender<Vec<u8>>, Receiver<Vec<u8>>) = sync_channel(DEFAULT_QUEUE_SIZE);
        initialize_sender(tx);
        start_file_output(&config, rx);

        QuickCheck::new().max_tests(DEFAULT_FUZZED_MESSAGE_COUNT).quickcheck(fuzz_target_rfc3164 as fn(String));
        let _ = check_result(&file_output_path);
    }

    pub fn get_config() -> Config {
        let mut config = match Config::from_path(DEFAULT_CONFIG_FILE) {
            Ok(config) => config,
            Err(e) => panic!(
                "Unable to read the config file [{}]: {}",
                "flowgger.toml",
                e.to_string()
            ),
        };
        
        update_file_rotation_defaults_in_config(&mut config);

        return config;
    }

    /// Update the default file rotation size and time in the config file
    /// This ensures output is sent to a single non-rotated file
    pub fn update_file_rotation_defaults_in_config(config: &mut Config){
        if let Some(entry) = config.config.get_mut("output").unwrap().get_mut("file_rotation_size"){
            *entry = Value::Integer(0);
        }

        if let Some(entry) = config.config.get_mut("output").unwrap().get_mut("file_rotation_time"){
            *entry = Value::Integer(0);
        }
    }

    pub fn remove_output_file(file_output_path: &str){
        let _ = fs::remove_file(file_output_path);
    }
    
    // Initialize the SyncSender which will be used to send data to the output file
    fn initialize_sender(tx: SyncSender<Vec<u8>>){
        let mut sender_guard = SYNC_SENDER.lock().unwrap();
        if sender_guard.is_none(){
            *sender_guard = Some(tx);
        }

        drop(sender_guard);
    }

    /// Start an input listener which writes data to the output file once received.
    pub fn start_file_output(config: &Config, rx: Receiver<Vec<u8>>){

        let output_format = config
            .lookup("output.format")
            .map_or(DEFAULT_OUTPUT_FORMAT, |x| {
                x.as_str().expect("output.format must be a string")
            });

        let output = get_file_output(&config);
        let output_type = config
            .lookup("output.type")
            .map_or(DEFAULT_OUTPUT_TYPE, |x| {
                x.as_str().expect("output.type must be a string")
            });

        let _output_framing = match config.lookup("output.framing") {
            Some(framing) => framing.as_str().expect("output.framing must be a string"),
            None => match (output_format, output_type) {
                ("capnp", _) | (_, "kafka") => "noop",
                (_, "debug") | ("ltsv", _) => "line",
                ("gelf", _) => "nul",
                _ => DEFAULT_OUTPUT_FRAMING,
            },
        };
        let merger: Option<Box<dyn Merger>> = Some(Box::new(LineMerger::new(&config)) as Box<dyn Merger>);

        let arx = Arc::new(Mutex::new(rx));
        output.start(arx, merger);
    }

    fn get_file_output(config: &Config) -> Box<dyn Output> {
        Box::new(FileOutput::new(config)) as Box<dyn Output>
    }
    
    pub fn fuzz_target_rfc3164(data: String) {
        let config = STATIC_CONFIG.clone();
        
        let mut sender_guard = match SYNC_SENDER.lock() {
            Ok(guard) => guard,
            Err(_poisoned_error) => {
                // Handle the poisoned Mutex
                let guard = _poisoned_error.into_inner();
                guard
            }
        };
        let tx: &mut SyncSender<Vec<u8>> = sender_guard.as_mut().unwrap();

        let encoder = get_encoder_rfc3164(&config);
        let decoder = get_decoder_rfc3164(&config);
        let (decoder, encoder): (Box<dyn Decoder>, Box<dyn Encoder>) =
            (decoder.clone_boxed(), encoder.clone_boxed());
        let _result = handle_record_maybe_compressed(data.as_bytes(), &tx, &decoder, &encoder);
    }

    // Check for the result
    // On invalid input, no logs are expected to be written to the output file
    // For valid inputs, analyze each log entry, check that the hostname and appnames are defined
    fn check_result(file_output_path: &str)-> Result<(), std::io::Error> {

        let mut sender_guard = match SYNC_SENDER.lock() {
            Ok(guard) => guard,
            Err(_poisoned_error) => {
                // Handle the poisoned Mutex
                let guard = _poisoned_error.into_inner();
                guard
            }
        };
        let tx: SyncSender<Vec<u8>> = sender_guard.take().unwrap();
        drop(tx);

        thread::sleep(time::Duration::from_millis(1000));

        let file = fs::File::open(file_output_path).expect("Unable to open output file");
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line_item: String = line?;
            let split_line_content: Vec<&str> = line_item.split(" ").filter(|data| !data.is_empty()).collect();
            let hostname = split_line_content[3].trim();
            let appname = split_line_content[4].trim();

            if hostname.is_empty() || appname.is_empty() {
                panic!("Log output invalid");
            }
        }
        Ok(())
    }
}