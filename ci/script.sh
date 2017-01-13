# This script takes care of testing your crate

set -ex

# TODO This is the "test phase", tweak it as you see fit
main() {
    cross build --target $TARGET --no-default-features
    cross build --target $TARGET --release --no-default-features

    if [ -n $DISABLE_TESTS ]; then
        return
    fi

    cross test --target $TARGET --no-default-features
    cross test --target $TARGET --release --no-default-features

    cross run --target $TARGET --no-default-features
    cross run --target $TARGET --release --no-default-features
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
