use flowgger::config::Config;
use flowgger::decoder::Decoder;
use flowgger::encoder::Encoder;
use flowgger::splitter::Splitter;
use flowgger::splitter::line_splitter::LineSplitter;
use flowgger::splitter::nul_splitter::NulSplitter;
use flowgger::splitter::syslen_splitter::SyslenSplitter;
use openssl::ssl::*;
use coio;
use coio::net::{TcpListener, TcpStream};
use std::io::{stderr, Write, BufReader};
use std::sync::mpsc::SyncSender;
use super::*;

pub struct TlsCoInput {
    listen: String,
    tls_config: TlsConfig
}

impl TlsCoInput {
    pub fn new(config: &Config) -> TlsCoInput {
        let (tls_config, listen, _timeout) = config_parse(&config);
        TlsCoInput {
            listen: listen,
            tls_config: tls_config
        }
    }
}

impl Input for TlsCoInput {
    fn accept(&self, tx: SyncSender<Vec<u8>>, decoder: Box<Decoder + Send>, encoder: Box<Encoder + Send>) {
        let listener = TcpListener::bind(&self.listen as &str).unwrap();
        let tls_config = self.tls_config.clone();
        let threads = tls_config.threads;
        coio::spawn(move|| {
            for client in listener.incoming() {
                match client {
                    Ok(client) => {
                        let tx = tx.clone();
                        let (decoder, encoder) = (decoder.clone_boxed(), encoder.clone_boxed());
                        let tls_config = tls_config.clone();
                        coio::spawn(move|| {
                            handle_client(client, tx, decoder, encoder, tls_config);
                        });
                    }
                    Err(_) => { }
                }
            }
        });
        coio::run(threads);
    }
}

fn handle_client(client: TcpStream, tx: SyncSender<Vec<u8>>, decoder: Box<Decoder>, encoder: Box<Encoder>, tls_config: TlsConfig) {
    if let Ok(peer_addr) = client.peer_addr() {
        println!("Connection over TLS<coroutines> from [{}]", peer_addr);
    }
    let sslclient = match SslStream::accept_generic(&*tls_config.arc_ctx, client) {
        Err(_) => {
            let _ = writeln!(stderr(), "SSL handshake aborted by the client");
            return
        }
        Ok(sslclient) => sslclient
    };
    let reader = BufReader::new(sslclient);
    let splitter = match &tls_config.framing as &str {
        "line" => Box::new(LineSplitter) as Box<Splitter<_>>,
        "syslen" => Box::new(SyslenSplitter) as Box<Splitter<_>>,
        "nul" => Box::new(NulSplitter) as Box<Splitter<_>>,
        _ => panic!("Unsupported framing scheme")
    };
    splitter.run(reader, tx, decoder, encoder);
}
