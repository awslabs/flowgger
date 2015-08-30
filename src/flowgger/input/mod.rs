pub mod tcp_input;
pub mod tls_input;

use flowgger::decoder::Decoder;
use flowgger::encoder::Encoder;
use std::sync::mpsc::SyncSender;

pub trait CloneBoxedInput {
    fn clone_boxed<'a>(&self) -> Box<Input + Send + 'a> where Self: 'a;
}

impl<T: Input + Clone + Send> CloneBoxedInput for T {
    fn clone_boxed<'a>(&self) -> Box<Input + Send + 'a> where Self: 'a {
        Box::new(self.clone())
    }
}

impl Clone for Box<Input> {
    fn clone(&self) -> Box<Input> {
        self.clone_boxed()
    }
}

pub trait Input : CloneBoxedInput {
    fn accept(&self, tx: SyncSender<Vec<u8>>, decoder: Box<Decoder + Send>, encoder: Box<Encoder + Send>);
}
