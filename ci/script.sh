# This script takes care of testing your crate

set -ex

# TODO This is the "test phase", tweak it as you see fit
main() {
    cross build --target $TARGET --no-default-features --features="coio"
    cross build --target $TARGET --release --no-default-features --features="coio"

    if [ -n $DISABLE_TESTS ]; then
        return
    fi

    cross test --target $TARGET --no-default-features --features="coio"
    cross test --target $TARGET --release --no-default-features --features="coio"

    cross run --target $TARGET --no-default-features --features="coio"
    cross run --target $TARGET --release --no-default-features --features="coio"
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
