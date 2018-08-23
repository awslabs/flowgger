mod gelf_decoder;
mod invalid_decoder;
mod ltsv_decoder;
mod rfc5424_decoder;

pub use self::gelf_decoder::GelfDecoder;
pub use self::invalid_decoder::InvalidDecoder;
pub use self::ltsv_decoder::LTSVDecoder;
pub use self::rfc5424_decoder::RFC5424Decoder;

use crate::flowgger::record::Record;

pub trait CloneBoxedDecoder {
    fn clone_boxed<'a>(&self) -> Box<Decoder + Send + 'a>
    where
        Self: 'a;
}

impl<T: Decoder + Clone + Send> CloneBoxedDecoder for T {
    fn clone_boxed<'a>(&self) -> Box<Decoder + Send + 'a>
    where
        Self: 'a,
    {
        Box::new(self.clone())
    }
}

impl Clone for Box<Decoder> {
    fn clone(&self) -> Box<Decoder> {
        self.clone_boxed()
    }
}

pub trait Decoder: CloneBoxedDecoder {
    fn decode(&self, line: &str) -> Result<Record, &'static str>;
}
