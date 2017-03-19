use flowgger::config::Config;
use flowgger::decoder::Decoder;
use flowgger::encoder::Encoder;
use flowgger::splitter::{Splitter, CapnpSplitter, LineSplitter, NulSplitter, SyslenSplitter};
use std::io::BufReader;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::SyncSender;
use std::thread;
use std::time::Duration;
use super::*;

pub struct TcpInput {
    listen: String,
    tcp_config: TcpConfig,
    timeout: Option<Duration>,
}

impl TcpInput {
    pub fn new(config: &Config) -> TcpInput {
        let (tcp_config, listen, timeout) = config_parse(&config);
        TcpInput {
            listen: listen,
            tcp_config: tcp_config,
            timeout: Some(Duration::from_secs(timeout)),
        }
    }
}

impl Input for TcpInput {
    fn accept(&self,
              tx: SyncSender<Vec<u8>>,
              decoder: Box<Decoder + Send>,
              encoder: Box<Encoder + Send>) {
        let listener = TcpListener::bind(&self.listen as &str).unwrap();
        for client in listener.incoming() {
            match client {
                Ok(client) => {
                    let _ = client.set_read_timeout(self.timeout);
                    let tx = tx.clone();
                    let tcp_config = self.tcp_config.clone();
                    let (decoder, encoder) = (decoder.clone_boxed(), encoder.clone_boxed());
                    thread::spawn(move || {
                                      handle_client(client, tx, decoder, encoder, tcp_config);
                                  });
                }
                Err(_) => {}
            }
        }
    }
}

fn handle_client(client: TcpStream,
                 tx: SyncSender<Vec<u8>>,
                 decoder: Box<Decoder>,
                 encoder: Box<Encoder>,
                 tcp_config: TcpConfig) {
    if let Ok(peer_addr) = client.peer_addr() {
        println!("Connection over TCP from [{}]", peer_addr);
    }
    let reader = BufReader::new(client);
    let splitter = match &tcp_config.framing as &str {
        "capnp" => Box::new(CapnpSplitter) as Box<Splitter<_>>,
        "line" => Box::new(LineSplitter) as Box<Splitter<_>>,
        "syslen" => Box::new(SyslenSplitter) as Box<Splitter<_>>,
        "nul" => Box::new(NulSplitter) as Box<Splitter<_>>,
        _ => panic!("Unsupported framing scheme"),
    };
    splitter.run(reader, tx, decoder, encoder);
}
