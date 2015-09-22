use flowgger::config::Config;

pub mod tcp_input;

pub use super::Input;

pub const DEFAULT_FRAMING: &'static str = "line";
pub const DEFAULT_LISTEN: &'static str = "0.0.0.0:6514";
pub const DEFAULT_TIMEOUT: u64 = 3600;

#[derive(Clone)]
pub struct TcpConfig {
    framing: String
}

pub fn config_parse(config: &Config) -> (TcpConfig, String, u64) {
    let listen = config.lookup("input.listen").map_or(DEFAULT_LISTEN, |x|x.as_str().
        expect("input.listen must be an ip:port string")).to_owned();
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
    let tcp_config = TcpConfig {
        framing: framing
    };
    (tcp_config, listen, timeout)
}
