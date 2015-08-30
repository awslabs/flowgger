FROM jedisct1/base-dev-rust-nightly:7bb0d0d74ca23cb871fd058a3fc6d935b7e47ae7
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

COPY 10-keepalive.conf /etc/sysctl.d/

RUN set -x && \
    service procps start

COPY flowgger.sh /etc/service/flowgger/run

EXPOSE 6514

ENTRYPOINT ["/sbin/my_init"]
