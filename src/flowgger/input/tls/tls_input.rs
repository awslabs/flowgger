use super::*;
use crate::flowgger::config::Config;
use crate::flowgger::decoder::Decoder;
use crate::flowgger::encoder::Encoder;
use crate::flowgger::splitter::{
    CapnpSplitter, LineSplitter, NulSplitter, Splitter, SyslenSplitter,
};
use std::io::{stderr, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::SyncSender;
use std::thread;
use std::time::Duration;

pub struct TlsInput {
    listen: String,
    timeout: Option<Duration>,
    tls_config: TlsConfig,
}

impl TlsInput {
    pub fn new(config: &Config) -> TlsInput {
        let (tls_config, listen, timeout) = config_parse(config);
        TlsInput {
            listen,
            tls_config,
            timeout: Some(Duration::from_secs(timeout)),
        }
    }
}

impl Input for TlsInput {
    fn accept(
        &self,
        tx: SyncSender<Vec<u8>>,
        decoder: Box<dyn Decoder + Send>,
        encoder: Box<dyn Encoder + Send>,
    ) {
        let listener = TcpListener::bind(&self.listen as &str).unwrap();
        for client in listener.incoming() {
            if let Ok(client) = client {
                let _ = client.set_read_timeout(self.timeout);
                let tx = tx.clone();
                let (decoder, encoder) = (decoder.clone_boxed(), encoder.clone_boxed());
                let tls_config = self.tls_config.clone();
                thread::spawn(move || {
                    handle_client(client, tx, decoder, encoder, tls_config);
                });
            }
        }
    }
}

fn handle_client(
    client: TcpStream,
    tx: SyncSender<Vec<u8>>,
    decoder: Box<dyn Decoder>,
    encoder: Box<dyn Encoder>,
    tls_config: TlsConfig,
) {
    if let Ok(peer_addr) = client.peer_addr() {
        println!("Connection over TLS from [{}]", peer_addr);
    }
    let sslclient = match tls_config.acceptor.accept(client) {
        Err(_) => {
            let _ = writeln!(stderr(), "SSL handshake aborted by the client");
            return;
        }
        Ok(sslclient) => sslclient,
    };
    let reader = BufReader::new(sslclient);
    let splitter = match &tls_config.framing as &str {
        "capnp" => Box::new(CapnpSplitter) as Box<dyn Splitter<_>>,
        "line" => Box::new(LineSplitter) as Box<dyn Splitter<_>>,
        "syslen" => Box::new(SyslenSplitter) as Box<dyn Splitter<_>>,
        "nul" => Box::new(NulSplitter) as Box<dyn Splitter<_>>,
        _ => panic!("Unsupported framing scheme"),
    };
    splitter.run(reader, tx, decoder, encoder);
}
