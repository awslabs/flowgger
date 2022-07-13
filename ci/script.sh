# This script takes care of testing your crate

set -ex

main() {
    if [[ 0 -eq $DISABLE_TESTS ]]; then
        cross test --target $TARGET --no-default-features --features "${FLOWGGER_FEATURES}"
    fi
}

main
