FROM jedisct1/base-dev-rust-nightly:1b5d52122e4dfa1cfd9d8c5afab931150269a7ff
MAINTAINER Frank Denis
ENV SERIAL 3

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

COPY flowgger.toml.in /opt/flowgger/etc/
COPY flowgger.pem /etc/ssl/private/
COPY flowgger.sh /etc/service/flowgger/run

ENV QUEUE_SIZE 5000000
ENV KAFKA_BROKERS '["172.16.205.129:9092"]'
ENV KAFKA_TOPIC test
ENV KAFKA_THREADS 1
ENV KAFKA_COALESCE 10000
ENV KAFKA_TIMEOUT 300
ENV GELF_EXTRA_1 'X-TEST-TOKEN-X = "Y"'

EXPOSE 6514

ENTRYPOINT ["/sbin/my_init"]
