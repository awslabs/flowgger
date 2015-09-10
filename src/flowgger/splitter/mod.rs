pub mod nul_splitter;
pub mod line_splitter;
pub mod syslen_splitter;

use std::io::BufReader;

pub trait Splitter<T> {
    fn run(&self, buf_reader: BufReader<T>);
}
