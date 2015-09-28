extern crate capnpc;

fn main() {
    ::capnpc::compile("src", &["record.capnp"]).unwrap();
}
