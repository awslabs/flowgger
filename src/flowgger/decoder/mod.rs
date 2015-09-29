pub mod gelf_decoder;
pub mod invalid_decoder;
pub mod ltsv_decoder;
pub mod rfc5424_decoder;

use flowgger::record::Record;

pub trait CloneBoxedDecoder {
    fn clone_boxed<'a>(&self) -> Box<Decoder + Send + 'a> where Self: 'a;
}

impl<T: Decoder + Clone + Send> CloneBoxedDecoder for T {
    fn clone_boxed<'a>(&self) -> Box<Decoder + Send + 'a> where Self: 'a {
        Box::new(self.clone())
    }
}

impl Clone for Box<Decoder> {
    fn clone(&self) -> Box<Decoder> {
        self.clone_boxed()
    }
}

pub trait Decoder : CloneBoxedDecoder {
    fn decode(&self, line: &str) -> Result<Record, &'static str>;
}
