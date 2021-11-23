set -ex

#main() {
    curl https://sh.rustup.rs -sSf | \
        sh -s -- -y --default-toolchain $TRAVIS_RUST_VERSION

    citarget=
    if [ $TRAVIS_OS_NAME = linux ]; then
        citarget=x86_64-unknown-linux-gnu
        sudo apt-get update

        sudo apt-get install musl-tools -y
        sudo apt-get -y install pkg-config openssl libssl-dev

        which pkg-config
        if [ -d "/usr/lib/pkgconfig" ]; then
            export PKG_CONFIG_LIBDIR=/usr/lib/pkgconfig
        else
            export PKG_CONFIG_LIBDIR=/usr/lib/aarch64-linux-gnu/pkgconfig
        fi
        ls -la $PKG_CONFIG_LIBDIR
        sort=sort
    else
        citarget=x86_64-apple-darwin
        sort=gsort  # for `sort --sort-version`, from brew's coreutils.
    fi

    # This fetches latest stable release
    cargo install cross
    # citag=$(git ls-remote --tags --refs --exit-code https://github.com/rust-embedded/cross \
    #                    | cut -d/ -f3 \
    #                    | grep -E '^v[0-9.]+$' \
    #                    | $sort --version-sort \
    #                    | tail -n1)
    # echo cross version: $citag
    # curl -LSfs https://japaric.github.io/trust/install.sh | \
    #     sh -s -- \
    #        --force \
    #        --git japaric/cross \
    #        --tag $citag \
    #        --target $citarget
#}

#main
