pub mod line_splitter;
pub mod syslen_splitter;

pub trait Splitter {
    fn run(self);
}
