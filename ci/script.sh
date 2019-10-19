# This script takes care of testing your crate

set -ex

main() {
    if [ -z $DISABLE_TESTS ]; then
        cross test --target $TARGET --no-default-features --features "syslog kafka-output file redis tls gelf ltsv"
    fi
}

main
