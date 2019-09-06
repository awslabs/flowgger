#[cfg(feature = "capnp-recompile")]
mod capnp_encoder;
#[cfg(feature = "gelf")]
mod gelf_encoder;
#[cfg(feature = "ltsv")]
mod ltsv_encoder;
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
#[cfg(feature = "rfc3164")]
pub use self::rfc3164_encoder::RFC3164Encoder;
#[cfg(feature = "rfc5424")]
pub use self::rfc5424_encoder::RFC5424Encoder;

use crate::flowgger::record::Record;

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
