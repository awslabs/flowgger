FROM jedisct1/base-dev-rust-nightly:1b5d52122e4dfa1cfd9d8c5afab931150269a7ff
MAINTAINER Frank Denis
ENV SERIAL 5

ENV DEBIAN_FRONTEND noninteractive

RUN apt-get install -y \
    libssl-dev \
    libsnappy-dev

RUN set -x && \
    cd /tmp && \
    git clone https://github.com/jedisct1/flowgger.git && \
    cd flowgger && \
    cargo build --release && \
    mkdir -p /opt/flowgger/etc /opt/flowgger/bin && \
    mv target/release/flowgger /opt/flowgger/bin/ && \
    rm -fr /tmp/flowgger

COPY flowgger.sh /etc/service/flowgger/run

EXPOSE 6514
