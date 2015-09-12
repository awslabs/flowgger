use flowgger::config::Config;
use flowgger::decoder::Decoder;
use flowgger::encoder::Encoder;
use flowgger::splitter::Splitter;
use flowgger::splitter::line_splitter::LineSplitter;
use flowgger::splitter::nul_splitter::NulSplitter;
use flowgger::splitter::syslen_splitter::SyslenSplitter;
use std::io::BufReader;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::SyncSender;
use std::thread;
use super::Input;

const DEFAULT_LISTEN: &'static str = "0.0.0.0:6514";
const DEFAULT_FRAMING: &'static str = "line";

#[derive(Clone)]
pub struct TcpConfig {
    framing: String
}

pub struct TcpInput {
    listen: String,
    tcp_config: TcpConfig
}

impl TcpInput {
    pub fn new(config: &Config) -> TcpInput {
        let listen = config.lookup("input.listen").map_or(DEFAULT_LISTEN, |x|x.as_str().
            expect("input.listen must be an ip:port string")).to_owned();
        let framing = if config.lookup("input.framed").map_or(false, |x| x.as_bool().
            expect("input.framed must be a boolean")) {
            "syslen"
        } else {
            DEFAULT_FRAMING
        };
        let framing = config.lookup("input.framing").map_or(framing, |x| x.as_str().
            expect(r#"input.framing must be a string set to "line", "nul" or "syslen""#)).to_owned();
        let tcp_config = TcpConfig {
            framing: framing
        };
        TcpInput {
            listen: listen,
            tcp_config: tcp_config
        }
    }
}

impl Input for TcpInput {
    fn accept(&self, tx: SyncSender<Vec<u8>>, decoder: Box<Decoder + Send>, encoder: Box<Encoder + Send>) {
        let listener = TcpListener::bind(&self.listen as &str).unwrap();
        for client in listener.incoming() {
            match client {
                Ok(client) => {
                    let tx = tx.clone();
                    let tcp_config = self.tcp_config.clone();
                    let (decoder, encoder) = (decoder.clone_boxed(), encoder.clone_boxed());
                    thread::spawn(move|| {
                        handle_client(client, tx, decoder, encoder, tcp_config);
                    });
                }
                Err(_) => { }
            }
        }
    }
}

fn handle_client(client: TcpStream, tx: SyncSender<Vec<u8>>, decoder: Box<Decoder>, encoder: Box<Encoder>, tcp_config: TcpConfig) {
    if let Ok(peer_addr) = client.peer_addr() {
        println!("Connection over TCP from [{}]", peer_addr);
    }
    let reader = BufReader::new(client);
    let splitter = match &tcp_config.framing as &str {
        "line" => Box::new(LineSplitter) as Box<Splitter<_>>,
        "syslen" => Box::new(SyslenSplitter) as Box<Splitter<_>>,
        "nul" => Box::new(NulSplitter) as Box<Splitter<_>>,
        _ => panic!("Unsupported framing scheme")
    };
    splitter.run(reader, tx, decoder, encoder);
}
