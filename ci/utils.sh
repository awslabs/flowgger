mktempd() {
    echo $(mktemp -d 2>/dev/null || mktemp -d -t tmp)
}

host() {
    case "$TRAVIS_OS_NAME" in
        linux)
            echo x86_64-unknown-linux-gnu
            ;;
        osx)
            echo x86_64-apple-darwin
            ;;
    esac
}

gcc_prefix() {
    case "$TARGET" in
        aarch64-unknown-linux-gnu)
            echo aarch64-linux-gnu-
            ;;
        arm*-gnueabihf)
            echo arm-linux-gnueabihf-
            ;;
        *-musl)
            echo musl-
            ;;
        *)
            return
            ;;
    esac
}

dostrip() {
    local stu=strip prefix=$(gcc_prefix)
    if which ${prefix}strip > /dev/null; then
        stu=${prefix}strip
    fi
    if strip --version 2>/dev/null | fgrep GNU >/dev/null ; then
        $stu -s $1
    else
        $stu $1
    fi
}

dobin() {
    [ -z $MAKE_DEB ] && die 'dobin: $MAKE_DEB not set'
    [ $# -lt 1 ] && die "dobin: at least one argument needed"

    local f prefix=$(gcc_prefix)
    for f in "$@"; do
        install -m0755 $f $dtd/debian/usr/bin/
        dostrip $dtd/debian/usr/bin/$(basename $f)
    done
}

architecture() {
    case $1 in
        x86_64-unknown-linux-gnu|x86_64-unknown-linux-musl)
            echo amd64
            ;;
        i686-unknown-linux-gnu|i686-unknown-linux-musl)
            echo i386
            ;;
        arm*-unknown-linux-gnueabihf)
            echo armhf
            ;;
        *)
            die "architecture: unexpected target $TARGET"
            ;;
    esac
}
