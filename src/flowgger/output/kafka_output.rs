use flowgger::config::Config;
use flowgger::merger::Merger;
use kafka::client::{Compression, KafkaClient};
use kafka::utils::ProduceMessage;
use std::io::{stderr, Write};
use std::process::exit;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;
use super::Output;

const KAFKA_DEFAULT_ACKS: i16 = 0;
const KAFKA_DEFAULT_COALESCE: usize = 1;
const KAFKA_DEFAULT_COMPRESSION: &'static str = "none";
const KAFKA_DEFAULT_THREADS: u32 = 1;
const KAFKA_DEFAULT_TIMEOUT: i32 = 60000;

pub struct KafkaOutput {
    config: KafkaConfig,
    threads: u32
}

#[derive(Clone)]
struct KafkaConfig {
    acks: i16,
    brokers: Vec<String>,
    topic: String,
    timeout: i32,
    coalesce: usize,
    compression: Compression
}

struct KafkaWorker {
    arx: Arc<Mutex<Receiver<Vec<u8>>>>,
    client: KafkaClient,
    config: KafkaConfig,
    queue: Vec<ProduceMessage>
}

impl KafkaWorker {
    fn new(arx: Arc<Mutex<Receiver<Vec<u8>>>>, config: KafkaConfig) -> KafkaWorker {
        let mut client = KafkaClient::new(config.brokers.clone());
        match client.load_metadata_all() {
            Ok(_) => {},
            Err(e) => {
                println!("Unable to connect to Kafka: [{}]", e);
                exit(1);
            }
        }
        client.set_compression(config.compression);
        let queue = Vec::with_capacity(config.coalesce);
        KafkaWorker {
            arx: arx,
            client: client,
            config: config,
            queue: queue
        }
    }

    fn run_nocoalesce(mut self) {
        loop {
            let bytes = match { self.arx.lock().unwrap().recv() } {
                Ok(line) => line,
                Err(_) => return
            };
            match self.client.send_message(self.config.acks, self.config.timeout, self.config.topic.clone(), bytes) {
                Ok(_) => {},
                Err(e) => {
                    println!("Kafka not responsive: [{}]", e);
                    exit(1);
                }
            }
        }
    }

    fn run_coalesce(mut self) {
        loop {
            let bytes = match { self.arx.lock().unwrap().recv() } {
                Ok(line) => line,
                Err(_) => return
            };
            let message = ProduceMessage {
                topic: self.config.topic.clone(),
                message: bytes
            };
            let mut queue = &mut self.queue;
            queue.push(message);
            if queue.len() >= self.config.coalesce {
                match self.client.send_messages(self.config.acks, self.config.timeout, queue.clone()) {
                    Ok(_) => {},
                    Err(e) => {
                        println!("Kafka not responsive: [{}]", e);
                        exit(1);
                    }
                }
                queue.clear();
            }
        }
    }

    fn run(self) {
        if self.config.coalesce <= 1 {
            self.run_nocoalesce()
        } else {
            self.run_coalesce()
        }
    }
}

impl KafkaOutput {
    pub fn new(config: &Config) -> KafkaOutput {
        let acks = config.lookup("output.kafka_acks").
            map_or(KAFKA_DEFAULT_ACKS, |x| x.as_integer().
            expect("output.kafka_acks must be a 16-bit integer") as i16);
        let brokers = config.lookup("output.kafka_brokers").expect("output.kafka_brokers is required").
            as_slice().expect("Invalid list of Kafka brokers").to_vec();
        let brokers = brokers.iter().map(|x| x.as_str().
            expect("output.kafka_brokers must be a list of strings").to_owned()).collect();
        let topic = config.lookup("output.kafka_topic").
            expect("output.kafka_topic must be a string").as_str().
            expect("output.kafka_topic must be a string").to_owned();
        let timeout = config.lookup("output.kafka_timeout").
            map_or(KAFKA_DEFAULT_TIMEOUT, |x| x.as_integer().
                expect("output.kafka_timeout must be a 32-bit integer") as i32);
        let threads = config.lookup("output.kafka_threads").
            map_or(KAFKA_DEFAULT_THREADS, |x| x.as_integer().
                expect("output.kafka_threads must be a 32-bit integer") as u32);
        let coalesce = config.lookup("output.kafka_coalesce").
            map_or(KAFKA_DEFAULT_COALESCE, |x| x.as_integer().
                expect("output.kafka_coalesce must be a size integer") as usize);
        let compression = match config.lookup("output.kafka_compression").
            map_or(KAFKA_DEFAULT_COMPRESSION, |x| x.as_str().
            expect("output.kafka_compresion must be a string")).to_lowercase().as_ref() {
            "none" => Compression::NONE,
            "gzip" => Compression::GZIP,
            "snappy" => Compression::SNAPPY,
            _ => panic!("Unsupported compression method")
        };
        let kafka_config = KafkaConfig {
            acks: acks,
            brokers: brokers,
            topic: topic,
            timeout: timeout,
            coalesce: coalesce,
            compression: compression
        };
        KafkaOutput {
            config: kafka_config,
            threads: threads
        }
    }
}

impl Output for KafkaOutput {
    fn start(&self, arx: Arc<Mutex<Receiver<Vec<u8>>>>, merger: Option<Box<Merger>>) {
        if merger.is_some() {
            let _ = writeln!(stderr(), "Output framing is ignored with the Kafka output");
        }
        for _ in 0..self.threads {
            let arx = arx.clone();
            let config = self.config.clone();
            thread::spawn(move || {
                let worker = KafkaWorker::new(arx, config);
                worker.run();
            });
        }
    }
}
