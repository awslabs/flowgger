#[cfg(feature = "capnp-recompile")]
mod capnp_encoder;
#[cfg(feature = "gelf")]
mod gelf_encoder;
#[cfg(feature = "ltsv")]
mod ltsv_encoder;
#[cfg(feature = "passthrough")]
mod passthrough_encoder;
#[cfg(feature = "rfc3164")]
mod rfc3164_encoder;
#[cfg(feature = "rfc5424")]
mod rfc5424_encoder;

#[cfg(feature = "capnp-recompile")]
pub use self::capnp_encoder::CapnpEncoder;
#[cfg(feature = "gelf")]
pub use self::gelf_encoder::GelfEncoder;
#[cfg(feature = "ltsv")]
pub use self::ltsv_encoder::LTSVEncoder;
#[cfg(feature = "passthrough")]
pub use self::passthrough_encoder::PassthroughEncoder;
#[cfg(feature = "rfc3164")]
pub use self::rfc3164_encoder::RFC3164Encoder;
#[cfg(feature = "rfc5424")]
pub use self::rfc5424_encoder::RFC5424Encoder;

use crate::flowgger::config::Config;
use crate::flowgger::record::Record;
use time::{format_description, OffsetDateTime};

pub trait CloneBoxedEncoder {
    fn clone_boxed<'a>(&self) -> Box<dyn Encoder + Send + 'a>
    where
        Self: 'a;
}

impl<T: Encoder + Clone + Send> CloneBoxedEncoder for T {
    fn clone_boxed<'a>(&self) -> Box<dyn Encoder + Send + 'a>
    where
        Self: 'a,
    {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Encoder> {
    fn clone(&self) -> Box<dyn Encoder> {
        self.clone_boxed()
    }
}

pub trait Encoder: CloneBoxedEncoder {
    fn encode(&self, record: Record) -> Result<Vec<u8>, &'static str>;
}

pub fn config_get_prepend_ts(config: &Config) -> Option<String> {
    config
        .lookup("output.syslog_prepend_timestamp")
        .map_or(None, |bs| {
            Some(
                bs.as_str()
                    .expect("output.syslog_prepend_timestamp should be a string")
                    .to_string(),
            )
        })
}

pub fn build_prepend_ts(format_str: &str) -> Result<String, &'static str> {
    let current_time = OffsetDateTime::now_utc();
    let format_item = match format_description::parse(format_str) {
        Ok(item) => item,
        Err(_) => return Err("Failed to format date"),
    };

    match current_time.format(&format_item) {
        Ok(date_str) => Ok(date_str),
        Err(_) => Err("Failed to format date"),
    }
}
