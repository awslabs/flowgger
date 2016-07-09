FROM jedisct1/base-dev-rust-nightly:0ae54a4a27506ef9f08e
MAINTAINER Frank Denis
ENV SERIAL 1

ENV DEBIAN_FRONTEND noninteractive

ENV BUILD_DEPS \
    autoconf \
    file \
    gcc \
    git \
    libc-dev \
    make \
    pkg-config

RUN set -x && \
    apt-get install -y \
        $BUILD_DEPS \
        libsnappy-dev \
        --no-install-recommends && \
    apt-get clean && \
    rm -fr /tmp/* /var/tmp/*

ENV LIBRESSL_VERSION 2.4.1
ENV LIBRESSL_SHA256 121922b13169cd47a85e3e77f0bc129f8d04247193b42491cb1fab9074e80477
ENV LIBRESSL_DOWNLOAD_URL http://ftp.openbsd.org/pub/OpenBSD/LibreSSL/libressl-${LIBRESSL_VERSION}.tar.gz

RUN set -x && \
    mkdir -p /tmp/src && \
    cd /tmp/src && \
    curl -sSL $LIBRESSL_DOWNLOAD_URL -o libressl.tar.gz && \
    echo "${LIBRESSL_SHA256} *libressl.tar.gz" | sha256sum -c - && \
    tar xzf libressl.tar.gz && \
    rm -f libressl.tar.gz && \
    cd libressl-${LIBRESSL_VERSION} && \
    ./configure --disable-shared --with-pic --disable-dependency-tracking --prefix=/opt/libressl && \
    make check && make install && \
    rm -fr /opt/libressl/share/man && \
    echo /opt/libressl/lib > /etc/ld.so.conf.d/libressl.conf && ldconfig && \
    rm -fr /tmp/*

ENV OPENSSL_LIB_DIR=/opt/libressl/lib
ENV OPENSSL_INCLUDE_DIR=/opt/libressl/include

RUN set -x && \
    cd /tmp && \
    git clone https://github.com/jedisct1/flowgger.git && \
    cd flowgger && \
    cargo build --release --features='ecdh coroutines' && \
    mkdir -p /opt/flowgger/etc /opt/flowgger/bin && \
    strip target/release/flowgger && \
    mv target/release/flowgger /opt/flowgger/bin/ && \
    rm -fr /tmp/flowgger

COPY flowgger.sh /etc/service/flowgger/run

EXPOSE 6514

ENTRYPOINT ["/sbin/my_init"]
