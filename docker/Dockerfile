FROM jedisct1/base-dev-rust-nightly:f5a64a678f7e5942b42781573683640952498b28
MAINTAINER Frank Denis
ENV SERIAL 1

ENV DEBIAN_FRONTEND noninteractive

RUN apt-get install -y \
    libsnappy-dev

ENV LIBRESSL_VERSION 2.2.3
ENV LIBRESSL_SHA256 a1ccc21adf91d60e99246031b99c930c9af5e1b1b5a61b1bec87beef6f16d882
ENV LIBRESSL_DOWNLOAD_URL http://ftp.openbsd.org/pub/OpenBSD/LibreSSL/libressl-${LIBRESSL_VERSION}.tar.gz

RUN set -x && \
    mkdir -p /tmp/src && \
    cd /tmp/src && \
    curl -sSL $LIBRESSL_DOWNLOAD_URL -o libressl.tar.gz && \
    echo "${LIBRESSL_SHA256} *libressl.tar.gz" | sha256sum -c - && \
    tar xzf libressl.tar.gz && \
    rm -f libressl.tar.gz && \
    cd libressl-${LIBRESSL_VERSION} && \
    ./configure --disable-dependency-tracking --prefix=/opt/libressl && \
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
    cargo build --release --features=ecdh && \
    mkdir -p /opt/flowgger/etc /opt/flowgger/bin && \
    mv target/release/flowgger /opt/flowgger/bin/ && \
    rm -fr /tmp/flowgger

COPY 10-keepalive.conf /etc/sysctl.d/

RUN set -x && \
    service procps start

COPY flowgger.sh /etc/service/flowgger/run

EXPOSE 6514

ENTRYPOINT ["/sbin/my_init"]
