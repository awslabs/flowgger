
extern crate kafka;

use std::sync::{Arc, Mutex};
use std::sync::mpsc::Receiver;
use std::thread;
use flowgger::config::Config;
use self::kafka::client::KafkaClient;
use self::kafka::utils::ProduceMessage;

const KAFKA_DEFAULT_TIMEOUT: i32 = 60;
const KAFKA_DEFAULT_THREADS: u32 = 1;

pub struct KafkaPool {
    kafka_client: Vec<KafkaClient>
}

#[derive(Clone)]
struct KafkaConfig {
    pub brokers: Vec<String>,
    pub topic: String,
    pub timeout: i32
}

struct KafkaWorker {
    arx: Arc<Mutex<Receiver<Vec<u8>>>>,
    client: KafkaClient,
    config: KafkaConfig
}

impl KafkaWorker {
    fn new(arx: Arc<Mutex<Receiver<Vec<u8>>>>, config: KafkaConfig) -> KafkaWorker {
        let mut client = KafkaClient::new(config.brokers.clone());
        client.load_metadata_all().unwrap();
        KafkaWorker {
            arx: arx,
            client: client,
            config: config
        }
    }

    fn run(&mut self) {
        loop {
            let bytes = match { self.arx.lock().unwrap().recv() } {
                Ok(line) => line,
                Err(_) => return
            };
            self.client.send_message(1, self.config.timeout, self.config.topic.clone(), bytes).unwrap();
        }
    }
}

impl KafkaPool {
    pub fn new(arx: Arc<Mutex<Receiver<Vec<u8>>>>, config: &Config) {
        let brokers = config.lookup("output.kafka_brokers").unwrap().as_slice().unwrap().to_vec();
        let brokers = brokers.iter().map(|x| x.as_str().unwrap().to_string()).collect();
        let topic = config.lookup("output.kafka_topic").unwrap().as_str().unwrap().to_string();
        let timeout = config.lookup("output.kafka_timeout").
            map_or(KAFKA_DEFAULT_TIMEOUT, |x| x.as_integer().unwrap() as i32);
        let threads = config.lookup("output.kafka_threads").
            map_or(KAFKA_DEFAULT_THREADS, |x| x.as_integer().unwrap() as u32);
        let config = KafkaConfig {
            brokers: brokers,
            topic: topic,
            timeout: timeout
        };
        for i in 0..threads {
            let arx0 = arx.clone();
            let config0 = config.clone();
            thread::spawn(move || {
                let mut worker = KafkaWorker::new(arx0, config0);
                worker.run();
            });
        }
    }
}
