# This script takes care of building your crate and packaging it for release

set -ex

main() {
    local release_dir="$(pwd)/release"
    mkdir -p $release_dir

    test -f Cargo.lock || cargo generate-lockfile

    cross rustc --bin flowgger --target $TARGET --release --no-default-features -- -C lto

    cp target/$TARGET/release/flowgger $release_dir/flowgger_$TARGET
}

main
