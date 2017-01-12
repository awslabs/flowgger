# This script takes care of testing your crate

set -ex

# TODO This is the "test phase", tweak it as you see fit
main() {
    cross build --target $TARGET --features="without_kafka coio"
    cross build --target $TARGET --release --features="without_kafka coio"

    if [ -n $DISABLE_TESTS ]; then
        return
    fi

    cross test --target $TARGET --features="without_kafka coio"
    cross test --target $TARGET --release --features="without_kafka coio"

    cross run --target $TARGET --features="without_kafka coio"
    cross run --target $TARGET --release --features="without_kafka coio"
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
