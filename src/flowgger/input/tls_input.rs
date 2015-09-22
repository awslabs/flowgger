use flowgger::config::Config;
use flowgger::decoder::Decoder;
use flowgger::encoder::Encoder;
use flowgger::splitter::Splitter;
use flowgger::splitter::line_splitter::LineSplitter;
use flowgger::splitter::nul_splitter::NulSplitter;
use flowgger::splitter::syslen_splitter::SyslenSplitter;
use openssl::ssl::*;
use openssl::ssl::SslMethod::*;
use openssl::x509::X509FileType;
use std::io::{stderr, Write, BufReader};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::SyncSender;
use std::time::Duration;
use std::thread;
use super::Input;
use super::tls_common::*;

pub struct TlsInput {
    listen: String,
    timeout: Option<Duration>,
    tls_config: TlsConfig
}

impl TlsInput {
    pub fn new(config: &Config) -> TlsInput {
        let listen = config.lookup("input.listen").map_or(DEFAULT_LISTEN, |x| x.as_str().
            expect("input.listen must be an ip:port string")).to_owned();
        let cert = config.lookup("input.tls_cert").map_or(DEFAULT_CERT, |x| x.as_str().
            expect("input.tls_cert must be a path to a .pem file")).to_owned();
        let key = config.lookup("input.tls_key").map_or(DEFAULT_KEY, |x| x.as_str().
            expect("input.tls_key must be a path to a .pem file")).to_owned();
        let ciphers = config.lookup("input.tls_ciphers").map_or(DEFAULT_CIPHERS, |x| x.as_str().
            expect("input.tls_ciphers must be a string with a cipher suite")).to_owned();
        let tls_method = match config.lookup("input.tls_method").map_or(DEFAULT_TLS_METHOD, |x| x.as_str().
            expect("input.tls_method must be a string with the TLS method")).to_lowercase().as_ref() {
                "sslv23" => SslMethod::Sslv23,
                "tlsv1" | "tlsv1.0" => SslMethod::Tlsv1,
                "tlsv1.1" => SslMethod::Tlsv1_1,
                "tlsv1.2" => SslMethod::Tlsv1_2,
                _ => panic!(r#"TLS method must be "SSLv23", "TLSv1.0", "TLSv1.1" or "TLSv1.2""#)
        };
        let verify_peer = config.lookup("input.tls_verify_peer").map_or(DEFAULT_VERIFY_PEER, |x| x.as_bool().
            expect("input.tls_verify_peer must be a boolean"));
        let ca_file: Option<PathBuf> = config.lookup("input.tls_ca_file").map_or(None, |x|
            Some(PathBuf::from(x.as_str().expect("input.tls_ca_file must be a path to a file"))));
        let compression = config.lookup("input.tls_compression").map_or(DEFAULT_COMPRESSION, |x| x.as_bool().
            expect("input.tls_compression must be a boolean"));
        let timeout = config.lookup("input.timeout").map_or(DEFAULT_TIMEOUT, |x| x.as_integer().
            expect("input.timeout must be an integer") as u64);
        let framing = if config.lookup("input.framed").map_or(false, |x| x.as_bool().
            expect("input.framed must be a boolean")) {
            "syslen"
        } else {
            DEFAULT_FRAMING
        };
        let framing = config.lookup("input.framing").map_or(framing, |x| x.as_str().
            expect(r#"input.framing must be a string set to "line", "nul" or "syslen""#)).to_owned();
        let mut ctx = SslContext::new(tls_method).unwrap();
        if verify_peer == false {
            ctx.set_verify(SSL_VERIFY_NONE, None);
        } else {
            ctx.set_verify_depth(TLS_VERIFY_DEPTH);
            ctx.set_verify(SSL_VERIFY_PEER | SSL_VERIFY_FAIL_IF_NO_PEER_CERT, None);
            if let Some(ca_file) = ca_file {
                if ctx.set_CA_file(&ca_file).is_err() {
                    panic!("Unable to read the trusted CA file");
                }
            }
        }
        let mut opts = SSL_OP_CIPHER_SERVER_PREFERENCE | SSL_OP_NO_SESSION_RESUMPTION_ON_RENEGOTIATION;
        if compression == false {
            opts = opts | SSL_OP_NO_COMPRESSION;
        }
        ctx.set_options(opts);
        set_fs(&mut ctx);
        ctx.set_certificate_file(&Path::new(&cert), X509FileType::PEM).unwrap();
        ctx.set_private_key_file(&Path::new(&key), X509FileType::PEM).unwrap();
        ctx.set_cipher_list(&ciphers).unwrap();
        let arc_ctx = Arc::new(ctx);
        let tls_config = TlsConfig {
            framing: framing,
            threads: 1,
            arc_ctx: arc_ctx
        };
        TlsInput {
            listen: listen,
            tls_config: tls_config,
            timeout: Some(Duration::from_secs(timeout))
        }
    }
}

impl Input for TlsInput {
    fn accept(&self, tx: SyncSender<Vec<u8>>, decoder: Box<Decoder + Send>, encoder: Box<Encoder + Send>) {
        let listener = TcpListener::bind(&self.listen as &str).unwrap();
        for client in listener.incoming() {
            match client {
                Ok(client) => {
                    let _ = client.set_read_timeout(self.timeout);
                    let tx = tx.clone();
                    let (decoder, encoder) = (decoder.clone_boxed(), encoder.clone_boxed());
                    let tls_config = self.tls_config.clone();
                    thread::spawn(move|| {
                        handle_client(client, tx, decoder, encoder, tls_config);
                    });
                }
                Err(_) => { }
            }
        }
    }
}

fn handle_client(client: TcpStream, tx: SyncSender<Vec<u8>>, decoder: Box<Decoder>, encoder: Box<Encoder>, tls_config: TlsConfig) {
    if let Ok(peer_addr) = client.peer_addr() {
        println!("Connection over TLS from [{}]", peer_addr);
    }
    let sslclient = match SslStream::accept(&*tls_config.arc_ctx, client) {
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
