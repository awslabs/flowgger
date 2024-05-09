use flowgger;
use quickcheck;

use quickcheck::QuickCheck;

use flowgger::flowgger::config::Config;
use flowgger::flowgger::encoder::Encoder;
use flowgger::flowgger::decoder::Decoder;
use flowgger::flowgger::merger;
use flowgger::flowgger::output;

use std::sync::mpsc::{Receiver, sync_channel, SyncSender};

use flowgger::flowgger::get_decoder_rfc3164;
use flowgger::flowgger::get_encoder_rfc3164;
use flowgger::flowgger::input::udp_input::handle_record_maybe_compressed;

use self::merger::{LineMerger, Merger};
use self::output::FileOutput;
use self::output::Output;

use std::sync::{Arc, Mutex};
use toml::Value;
use std::fs;
use std::{thread, time};

const DEFAULT_CONFIG_FILE: &str = "flowgger.toml";
const DEFAULT_OUTPUT_FILEPATH: &str = "output.log";
const DEFAULT_QUEUE_SIZE: usize = 10_000_000;

const DEFAULT_OUTPUT_FORMAT: &str = "gelf";
const DEFAULT_OUTPUT_FRAMING: &str = "noop";
const DEFAULT_OUTPUT_TYPE: &str = "file";

const DEFAULT_FUZZED_MESSAGE_COUNT: u64 = 500;

fn get_file_output(config: &Config) -> Box<dyn Output> {
    Box::new(FileOutput::new(config)) as Box<dyn Output>
}

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

pub fn get_config() -> Config {
    let mut config = match Config::from_path(DEFAULT_CONFIG_FILE) {
        Ok(config) => config,
        Err(e) => panic!(
            "Unable to read the config file [{}]: {}",
            "flowgger.toml",
            e.to_string()
        ),
    };

    if let Some(entry) = config.config.get_mut("output").unwrap().get_mut("file_rotation_time"){
        *entry = Value::Integer(0);
    }else{
        panic!("Failed to find config entry");
    }

    return config;
}

pub fn remove_output_file(file_output_path: &str){
    fs::remove_file(file_output_path);
}

pub fn fuzz_target_rfc3164(data: &[u8]) {
    let config = get_config();
    let file_output_path = config.lookup("output.file_path").map_or(DEFAULT_OUTPUT_FILEPATH, |x| {
        x.as_str().expect("File output path missing in config")
    });
    remove_output_file(&file_output_path);

    if let Ok(s) = std::str::from_utf8(data) {
        let (tx, rx): (SyncSender<Vec<u8>>, Receiver<Vec<u8>>) = sync_channel(DEFAULT_QUEUE_SIZE);
        start_file_output(&config, rx);

        let encoder = get_encoder_rfc3164(&config);
        let decoder = get_decoder_rfc3164(&config);
        let (decoder, encoder): (Box<dyn Decoder>, Box<dyn Encoder>) =
            (decoder.clone_boxed(), encoder.clone_boxed());
        let result = handle_record_maybe_compressed(s.as_bytes(), &tx, &decoder, &encoder);

        match result {
            Ok(_) => {
                drop(tx);
                thread::sleep(time::Duration::from_millis(100));
                
                let file_contents = match fs::read_to_string(file_output_path){
                    Ok(contents) => contents,
                    Err(_) => {
                        println!("Failed to read file");
                        "".to_string()
                    }
                };
                
                let split_file_content: Vec<&str> = file_contents.split(" ").filter(|s| !s.is_empty()).collect();
                let split_input: Vec<&str> = s.split(" ").filter(|s| !s.is_empty()).collect();

                let hostnames_match = split_file_content[3].trim() == split_input[3].trim();
                let appnames_match = split_file_content[4].trim() == split_input[4].trim();
                
                if !(hostnames_match && appnames_match){
                    panic!("Log output invalid");
                }
            }
            Err(_) => {
            }
        }


    }
}


#[test]
fn test_fuzzer(){
    let config = get_config();
    let fuzzed_message_count = match config.lookup("test.fuzzed_message_count"){
        Some(count) => count.as_integer().unwrap() as u64,
        None => DEFAULT_FUZZED_MESSAGE_COUNT,
    };

    fn fuzz(data: String){
        fuzz_target_rfc3164(data.as_bytes());
    }
    QuickCheck::new().max_tests(fuzzed_message_count).quickcheck(fuzz as fn(String));
}