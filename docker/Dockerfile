FROM jedisct1/base-dev-rust-nightly:cb40b8a2ab562f103c86a2f5ac8ac66084d2754e
MAINTAINER Frank Denis
ENV SERIAL 1

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

ENV RUST_BACKTRACE=1

EXPOSE 6514

ENTRYPOINT ["/sbin/my_init"]
