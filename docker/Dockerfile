FROM rust:1.38 AS builder
MAINTAINER Frank Denis, Damian Czaja

ENV LIBRESSL_VERSION 2.9.2
ENV LIBRESSL_SHA256 c4c78167fae325b47aebd8beb54b6041d6f6a56b3743f4bd5d79b15642f9d5d4
ENV LIBRESSL_DOWNLOAD_URL https://ftp.openbsd.org/pub/OpenBSD/LibreSSL/libressl-${LIBRESSL_VERSION}.tar.gz

ENV OPENSSL_LIB_DIR=/opt/libressl/lib
ENV OPENSSL_INCLUDE_DIR=/opt/libressl/include

RUN apt-get update && \
	apt-get install -y \
		capnproto && \
	curl -sSL $LIBRESSL_DOWNLOAD_URL -o libressl.tar.gz && \
    echo "${LIBRESSL_SHA256} *libressl.tar.gz" | sha256sum -c - && \
    tar xzf libressl.tar.gz && \
    cd libressl-${LIBRESSL_VERSION} && \
    ./configure --disable-shared --with-pic --disable-dependency-tracking --prefix=/opt/libressl && \
    make check && make install

COPY . /flowgger

RUN cd /flowgger && \
    cargo build --release && \
    strip target/release/flowgger


FROM debian:buster-slim

ENV OPENSSL_LIB_DIR=/opt/libressl/lib
ENV OPENSSL_INCLUDE_DIR=/opt/libressl/include

WORKDIR /opt/flowgger

RUN mkdir -p /opt/flowgger/bin /opt/flowgger/etc
COPY --from=builder /flowgger/target/release/flowgger /opt/flowgger/bin/flowgger
COPY --from=builder /opt/libressl /opt/libressl
COPY docker/entrypoint.sh /
COPY flowgger.toml /opt/flowgger/etc/flowgger.toml

ENTRYPOINT ["/entrypoint.sh"]
CMD ["/opt/flowgger/etc/flowgger.toml"]
