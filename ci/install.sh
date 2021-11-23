set -ex

main() {
    curl https://sh.rustup.rs -sSf | \
        sh -s -- -y --default-toolchain $TRAVIS_RUST_VERSION

    local target=
    if [ $TRAVIS_OS_NAME = linux ]; then
        target=x86_64-unknown-linux-gnu
        # sudo apt-get update
        #
        # sudo apt-get install musl-tools -y
        # sudo apt-get -y install pkg-config openssl libssl-dev

        # pkfconfig is now installed in usr/lib in some installs
        # if [ -d "/usr/lib/pkgconfig" ]; then
        #     export PKG_CONFIG_LIBDIR=/usr/lib/pkgconfig
        # else
        #     export PKG_CONFIG_LIBDIR=/usr/lib/aarch64-linux-gnu/pkgconfig
        # fi
        sort=sort
    else
        target=x86_64-apple-darwin
        sort=gsort  # for `sort --sort-version`, from brew's coreutils.
    fi

    # Cross removed support of openssl, which breaks in musl. We're using an older version.
    # Eventually we may want to create a specific docker file for cross as mentionned here
    # https://github.com/rust-embedded/cross/issues/229
    cargo install --version 0.1.16 cross
}

main
