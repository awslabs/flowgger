#[cfg(feature = "capnp-recompile")]
extern crate capnpc;

#[cfg(feature = "capnp-recompile")]
fn main() {
    ::capnpc::compile("src", &["record.capnp"]).unwrap();
}

#[cfg(not(feature = "capnp-recompile"))]
fn main() { }
