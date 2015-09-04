extern crate redis;

use flowgger::config::Config;
use flowgger::decoder::Decoder;
use flowgger::encoder::Encoder;
use self::redis::{Commands, RedisResult};
use std::io::{stderr, Write};
use std::sync::mpsc::SyncSender;
use super::Input;

const DEFAULT_CONNECT: &'static str = "127.0.0.1";
const DEFAULT_QUEUE_KEY: &'static str = "logs";

pub struct RedisInput {
    connect: String,
    queue_key: String,
    queue_key_tmp: String
}

impl RedisInput {
    pub fn new(config: &Config) -> RedisInput {
        let connect = config.lookup("input.redis_connect").map_or(DEFAULT_CONNECT, |x|x.as_str().
            expect("input.redis_connect must be an ip:port string")).to_owned();
        let queue_key = config.lookup("input.redis_queue_key").map_or(DEFAULT_QUEUE_KEY, |x|x.as_str().
            expect("input.redis_queue_key must be a string")).to_owned();
        let queue_key_tmp = queue_key.clone() + ".tmp";
        RedisInput {
            connect: connect,
            queue_key: queue_key,
            queue_key_tmp: queue_key_tmp
        }
    }
}

impl Input for RedisInput {
    fn accept(&self, tx: SyncSender<Vec<u8>>, decoder: Box<Decoder + Send>, encoder: Box<Encoder + Send>) {
        let redis_cnx = match redis::Client::open(format!("redis://{}/", self.connect).as_ref()) {
            Err(_) => panic!("Invalid connection string for the Redis server: [{}]", self.connect),
            Ok(client) => match client.get_connection() {
                Err(_) => panic!("Unable to connect to the Redis server: [{}]", self.connect),
                Ok(redis_cnx) => redis_cnx
            }
        };
        let (queue_key, queue_key_tmp): (&str, &str) = (&self.queue_key, &self.queue_key_tmp);
        println!("Connected to Redis [{}], pulling messages from key [{}]", self.connect, queue_key);
        while {
            let dummy: RedisResult<()> = redis_cnx.rpoplpush(queue_key_tmp, queue_key);
            dummy.is_ok()
        } { };
        let (decoder, encoder): (Box<Decoder>, Box<Encoder>) = (decoder, encoder);
        loop {
            let line: String = match redis_cnx.brpoplpush(queue_key, queue_key_tmp, 0) {
                Err(_) => panic!("Redis protocol error in BRPOPLPUSH"),
                Ok(line) => line
            };
            if let Err(e) = handle_line(&line, &tx, &decoder, &encoder) {
                let _ = writeln!(stderr(), "{}: [{}]", e, line.trim());
            }
            match redis_cnx.lrem(self.queue_key.clone(), 1, line) {
                Err(_) => panic!("Redis protocol error in LREM"),
                Ok(()) => ()
            };
        }
    }
}

fn handle_line(line: &String, tx: &SyncSender<Vec<u8>>, decoder: &Box<Decoder>, encoder: &Box<Encoder>) -> Result<(), &'static str> {
    let decoded = try!(decoder.decode(&line));
    let reencoded = try!(encoder.encode(decoded));
    tx.send(reencoded).unwrap();
    Ok(())
}
