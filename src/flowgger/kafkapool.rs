
extern crate kafka;

use flowgger::Output;
use flowgger::config::Config;
use self::kafka::client::KafkaClient;
use self::kafka::utils::ProduceMessage;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;

const KAFKA_DEFAULT_COALESCE: usize = 1;
const KAFKA_DEFAULT_THREADS: u32 = 1;
const KAFKA_DEFAULT_TIMEOUT: i32 = 60;

pub struct KafkaPool {
    config: KafkaConfig,
    threads: u32
}

#[derive(Clone)]
struct KafkaConfig {
    brokers: Vec<String>,
    topic: String,
    timeout: i32,
    coalesce: usize
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
        client.load_metadata_all().unwrap();
        let queue = Vec::with_capacity(config.coalesce);
        KafkaWorker {
            arx: arx,
            client: client,
            config: config,
            queue: queue
        }
    }

    fn run_nocoalesce(&mut self) {
        loop {
            let bytes = match { self.arx.lock().unwrap().recv() } {
                Ok(line) => line,
                Err(_) => return
            };
            self.client.send_message(1, self.config.timeout, self.config.topic.clone(), bytes).unwrap();
        }
    }

    fn run_coalesce(&mut self) {
        loop {
            let bytes = match { self.arx.lock().unwrap().recv() } {
                Ok(line) => line,
                Err(_) => return
            };
            let message = ProduceMessage {
                topic: self.config.topic.clone(),
                message: bytes
            };
            self.queue.push(message);
            if self.queue.len() >= self.config.coalesce {
                self.client.send_messages(1, self.config.timeout, self.queue.clone()).unwrap();
                self.queue.clear();
            }
        }
    }

    fn run(&mut self) {
        if self.config.coalesce <= 1 {
            self.run_nocoalesce()
        } else {
            self.run_coalesce()
        }
    }
}

impl Output for KafkaPool {
    fn new(config: &Config) -> KafkaPool {
        let brokers = config.lookup("output.kafka_brokers").unwrap().as_slice().unwrap().to_vec();
        let brokers = brokers.iter().map(|x| x.as_str().unwrap().to_string()).collect();
        let topic = config.lookup("output.kafka_topic").unwrap().as_str().unwrap().to_string();
        let timeout = config.lookup("output.kafka_timeout").
            map_or(KAFKA_DEFAULT_TIMEOUT, |x| x.as_integer().unwrap() as i32);
        let threads = config.lookup("output.kafka_threads").
            map_or(KAFKA_DEFAULT_THREADS, |x| x.as_integer().unwrap() as u32);
        let coalesce = config.lookup("output.kafka_coalesce").
            map_or(KAFKA_DEFAULT_COALESCE, |x| x.as_integer().unwrap() as usize);
        let kafka_config = KafkaConfig {
            brokers: brokers,
            topic: topic,
            timeout: timeout,
            coalesce: coalesce
        };
        KafkaPool {
            config: kafka_config,
            threads: threads
        }
    }

    fn start(&self, arx: Arc<Mutex<Receiver<Vec<u8>>>>) {
        for i in 0..self.threads {
            let arx = arx.clone();
            let config = self.config.clone();
            thread::spawn(move || {
                let mut worker = KafkaWorker::new(arx, config);
                worker.run();
            });
        }
    }
}
