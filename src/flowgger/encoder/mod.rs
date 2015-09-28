pub mod capnp_encoder;
pub mod gelf_encoder;

use flowgger::record::Record;

pub trait CloneBoxedEncoder {
    fn clone_boxed<'a>(&self) -> Box<Encoder + Send + 'a> where Self: 'a;
}

impl<T: Encoder + Clone + Send> CloneBoxedEncoder for T {
    fn clone_boxed<'a>(&self) -> Box<Encoder + Send + 'a> where Self: 'a {
        Box::new(self.clone())
    }
}

impl Clone for Box<Encoder> {
    fn clone(&self) -> Box<Encoder> {
        self.clone_boxed()
    }
}

pub trait Encoder : CloneBoxedEncoder {
    fn encode(&self, record: Record) -> Result<Vec<u8>, &'static str>;
}
