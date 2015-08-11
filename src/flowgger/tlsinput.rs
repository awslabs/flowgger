
use flowgger::config::Config;
use flowgger::{Decoder, Encoder, Input};
use openssl::ssl::*;
use openssl::ssl::SslMethod::*;
use openssl::x509::X509FileType;
use std::io::{stderr, Write, BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::mpsc::SyncSender;
use std::thread;

const DEFAULT_CERT_FILE: &'static str = "flowgger.pem";
const DEFAULT_CIPHER_LIST: &'static str = "ECDHE-RSA-CHACHA20-POLY1305:ECDHE-ECDSA-CHACHA20-POLY1305:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES256-GCM-SHA384:ECDHE-ECDSA-AES256-GCM-SHA384:DHE-RSA-AES128-GCM-SHA256:DHE-DSS-AES128-GCM-SHA256:kEDH+AESGCM:ECDHE-RSA-AES128-SHA256:ECDHE-ECDSA-AES128-SHA256:ECDHE-RSA-AES128-SHA:ECDHE-ECDSA-AES128-SHA:ECDHE-RSA-AES256-SHA384:ECDHE-ECDSA-AES256-SHA384:ECDHE-RSA-AES256-SHA:ECDHE-ECDSA-AES256-SHA:DHE-RSA-AES128-SHA256:DHE-RSA-AES128-SHA:DHE-DSS-AES128-SHA256:DHE-RSA-AES256-SHA256:DHE-DSS-AES256-SHA:DHE-RSA-AES256-SHA:AES128-GCM-SHA256:AES256-GCM-SHA384:AES128-SHA256:AES256-SHA256:AES128-SHA:AES256-SHA:AES:CAMELLIA:DES-CBC3-SHA:!aNULL:!eNULL:!EXPORT:!DES:!RC4:!MD5:!PSK:!aECDH:!EDH-DSS-DES-CBC3-SHA:!EDH-RSA-DES-CBC3-SHA:!KRB5-DES-CBC3-SH";
const DEFAULT_KEY_FILE: &'static str = "flowgger.pem";
const DEFAULT_LISTEN: &'static str = "0.0.0.0:6514";

pub struct TlsInput {
    listen: String
}

impl Input for TlsInput {
    fn new(config: &Config) -> TlsInput {
        let listen = config.lookup("input.listen").map_or(DEFAULT_LISTEN, |x| x.as_str().unwrap()).to_string();
        TlsInput {
            listen: listen
        }
    }

    fn accept<TD, TE>(&self, tx: SyncSender<Vec<u8>>, decoder: TD, encoder: TE) where TD: Decoder + Clone + Send + 'static, TE: Encoder + Clone + Send + 'static {
        let listener = TcpListener::bind(&self.listen as &str).unwrap();
        for client in listener.incoming() {
            match client {
                Ok(client) => {
                    let tx = tx.clone();
                    let (decoder, encoder) = (decoder.clone(), encoder.clone());
                    thread::spawn(move|| {
                        handle_client(client, tx, decoder, encoder);
                    });
                }
                Err(_) => { }
            }
        }
    }
}

fn handle_line<TD, TE>(line: &String, tx: &SyncSender<Vec<u8>>, decoder: &TD, encoder: &TE) -> Result<(), &'static str> where TD: Decoder, TE: Encoder {
    let decoded = try!(decoder.decode(&line));
    let reencoded = try!(encoder.encode(decoded));
    tx.send(reencoded).unwrap();
    Ok(())
}

fn handle_client<TD, TE>(client: TcpStream, tx: SyncSender<Vec<u8>>, decoder: TD, encoder: TE) where TD: Decoder, TE: Encoder {
    let mut ctx = SslContext::new(Tlsv1_2).unwrap();
    ctx.set_verify(SSL_VERIFY_PEER, None); // Sigh
    ctx.set_options(SSL_OP_NO_COMPRESSION | SSL_OP_CIPHER_SERVER_PREFERENCE | SSL_OP_NO_SESSION_RESUMPTION_ON_RENEGOTIATION);
    ctx.set_certificate_file(&Path::new(DEFAULT_CERT_FILE), X509FileType::PEM).unwrap();
    ctx.set_private_key_file(&Path::new(DEFAULT_KEY_FILE), X509FileType::PEM).unwrap();
    ctx.set_cipher_list(DEFAULT_CIPHER_LIST).unwrap();
    let sslclient = SslStream::accept(&ctx, client).unwrap();
    let reader = BufReader::new(sslclient);
    for line in reader.lines() {
        let line = match line {
            Err(_) => return,
            Ok(line) => line
        };
        match handle_line(&line, &tx, &decoder, &encoder) {
            Err(e) => { let _ = writeln!(stderr(), "{}: [{}]", e, line.trim()); }
            _ => { }
        }
    }
}
