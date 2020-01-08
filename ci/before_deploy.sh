# This script takes care of building your crate and packaging it for release

set -ex

main() {
    local release_dir="$(pwd)/release"
    mkdir -p $release_dir

    test -f Cargo.lock || cargo generate-lockfile

    cross rustc --bin flowgger --target $TARGET --release --no-default-features --features "${FLOWGGER_FEATURES}" -- -C lto
    cp target/$TARGET/release/flowgger $release_dir/flowgger
    cp flowgger.toml $release_dir/
    zip -jr flowgger_$TARGET.zip $release_dir
}

main
