# This script takes care of testing your crate

set -ex

main() {
    if [ -z $DISABLE_TESTS ]; then
        cross test --target $TARGET --no-default-features
    fi

    cross build --target $TARGET --release --no-default-features
}

main
