use flowgger::config::Config;
use flowgger::decoder::Decoder;
use flowgger::encoder::Encoder;
use flowgger::splitter::Splitter;
use flowgger::splitter::line_splitter::LineSplitter;
use flowgger::splitter::nul_splitter::NulSplitter;
use flowgger::splitter::syslen_splitter::SyslenSplitter;
use openssl::bn::BigNum;
use openssl::dh::DH;
use openssl::ssl::*;
use openssl::ssl::SslMethod::*;
use openssl::x509::X509FileType;
use std::io::{stderr, Write, BufReader};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::mpsc::SyncSender;
use std::time::Duration;
use std::thread;
use super::Input;

const DEFAULT_CERT: &'static str = "flowgger.pem";
const DEFAULT_CIPHERS: &'static str = "DHE-RSA-AES128-GCM-SHA256:DHE-DSS-AES128-GCM-SHA256:kEDH+AESGCM:DHE-RSA-AES128-SHA256:DHE-RSA-AES128-SHA:DHE-DSS-AES128-SHA256:DHE-RSA-AES256-SHA256:DHE-DSS-AES256-SHA:DHE-RSA-AES256-SHA:AES128-GCM-SHA256:AES256-GCM-SHA384:AES128-SHA256:AES256-SHA256:AES128-SHA:AES256-SHA:AES:CAMELLIA:DES-CBC3-SHA:!aNULL:!eNULL:!EXPORT:!DES:!RC4:!MD5:!PSK:!aECDH:!EDH-DSS-DES-CBC3-SHA:!EDH-RSA-DES-CBC3-SHA:!KRB5-DES-CBC3-SH";
const DEFAULT_FRAMING: &'static str = "line";
const DEFAULT_KEY: &'static str = "flowgger.pem";
const DEFAULT_LISTEN: &'static str = "0.0.0.0:6514";
const DEFAULT_TLS_METHOD: &'static str = "TLSv1.2";
const DEFAULT_VERIFY_PEER: bool = false;
const DEFAULT_COMPRESSION: bool = false;
const TLS_VERIFY_DEPTH: u32 = 6;

#[derive(Clone)]
struct TlsConfig {
    cert: String,
    key: String,
    ciphers: String,
    framing: String,
    tls_method: SslMethod,
    verify_peer: bool,
    ca_file: Option<PathBuf>,
    compression: bool
}

pub struct TlsInput {
    listen: String,
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
        let framing = if config.lookup("input.framed").map_or(false, |x| x.as_bool().
            expect("input.framed must be a boolean")) {
            "syslen"
        } else {
            DEFAULT_FRAMING
        };
        let framing = config.lookup("input.framing").map_or(framing, |x| x.as_str().
            expect(r#"input.framing must be a string set to "line", "nul" or "syslen""#)).to_owned();

        let tls_config = TlsConfig {
            cert: cert,
            key: key,
            ciphers: ciphers,
            framing: framing,
            tls_method: tls_method,
            verify_peer: verify_peer,
            ca_file: ca_file,
            compression: compression
        };
        TlsInput {
            listen: listen,
            tls_config: tls_config
        }
    }
}

impl Input for TlsInput {
    fn accept(&self, tx: SyncSender<Vec<u8>>, decoder: Box<Decoder + Send>, encoder: Box<Encoder + Send>) {
        let listener = TcpListener::bind(&self.listen as &str).unwrap();
        for client in listener.incoming() {
            match client {
                Ok(client) => {
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

fn set_fs(ctx: &SslContext) {
    let p = BigNum::from_hex_str("87A8E61DB4B6663CFFBBD19C651959998CEEF608660DD0F25D2CEED4435E3B00E00DF8F1D61957D4FAF7DF4561B2AA3016C3D91134096FAA3BF4296D830E9A7C209E0C6497517ABD5A8A9D306BCF67ED91F9E6725B4758C022E0B1EF4275BF7B6C5BFC11D45F9088B941F54EB1E59BB8BC39A0BF12307F5C4FDB70C581B23F76B63ACAE1CAA6B7902D52526735488A0EF13C6D9A51BFA4AB3AD8347796524D8EF6A167B5A41825D967E144E5140564251CCACB83E6B486F6B3CA3F7971506026C0B857F689962856DED4010ABD0BE621C3A3960A54E710C375F26375D7014103A4B54330C198AF126116D2276E11715F693877FAD7EF09CADB094AE91E1A1597").unwrap();
    let g = BigNum::from_hex_str("3FB32C9B73134D0B2E77506660EDBD484CA7B18F21EF205407F4793A1A0BA12510DBC15077BE463FFF4FED4AAC0BB555BE3A6C1B0C6B47B1BC3773BF7E8C6F62901228F8C28CBB18A55AE31341000A650196F931C77A57F2DDF463E5E9EC144B777DE62AAAB8A8628AC376D282D6ED3864E67982428EBC831D14348F6F2F9193B5045AF2767164E1DFC967C1FB3F2E55A4BD1BFFE83B9C80D052B985D182EA0ADB2A3B7313D3FE14C8484B1E052588B9B7D2BBD2DF016199ECD06E1557CD0915B3353BBB64E0EC377FD028370DF92B52C7891428CDC67EB6184B523D1DB246C32F63078490F00EF8D647D148D47954515E2327CFEF98C582664B4C0F6CC41659").unwrap();
    let q = BigNum::from_hex_str("8CF83642A709A097B447997640129DA299B1A47D1EB3750BA308B0FE64F5FBD3").unwrap();
    let dh = DH::from_params(p, g, q).unwrap();
    ctx.set_tmp_dh(dh).unwrap();
}

fn handle_client(client: TcpStream, tx: SyncSender<Vec<u8>>, decoder: Box<Decoder>, encoder: Box<Encoder>, tls_config: TlsConfig) {
    let mut ctx = SslContext::new(tls_config.tls_method).unwrap();
    if tls_config.verify_peer == false {
        ctx.set_verify(SSL_VERIFY_NONE, None);
    } else {
        ctx.set_verify_depth(TLS_VERIFY_DEPTH);
        ctx.set_verify(SSL_VERIFY_PEER | SSL_VERIFY_FAIL_IF_NO_PEER_CERT, None);
        if let Some(ca_file) = tls_config.ca_file {
            if ctx.set_CA_file(&ca_file).is_err() {
                panic!("Unable to read the trusted CA file");
            }
        }
    }
    let mut opts = SSL_OP_CIPHER_SERVER_PREFERENCE | SSL_OP_NO_SESSION_RESUMPTION_ON_RENEGOTIATION;
    if tls_config.compression == false {
        opts = opts | SSL_OP_NO_COMPRESSION;
    }
    ctx.set_options(opts);
    set_fs(&ctx);
    ctx.set_certificate_file(&Path::new(&tls_config.cert), X509FileType::PEM).unwrap();
    ctx.set_private_key_file(&Path::new(&tls_config.key), X509FileType::PEM).unwrap();
    ctx.set_cipher_list(&tls_config.ciphers).unwrap();
    if let Ok(peer_addr) = client.peer_addr() {
        println!("Connection over TLS from [{}]", peer_addr);
    }
    let sslclient = match SslStream::accept(&ctx, client) {
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
