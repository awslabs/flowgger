extern crate capnpc;

fn main() {
    ::capnpc::compile("src", &["src/flowgger/record.capnp"]).unwrap();
}
