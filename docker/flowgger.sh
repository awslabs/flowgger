#! /bin/sh

KAFKA_BROKERS=$(echo "$KAFKA_BROKERS" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g')
GELF_EXTRA_1=$(echo "$GELF_EXTRA_1" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g')
GELF_EXTRA_2=$(echo "$GELF_EXTRA_2" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g')
GELF_EXTRA_3=$(echo "$GELF_EXTRA_3" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g')

ls -l /opt/flowgger/etc/

sed -e "s/@QUEUE_SIZE@/${QUEUE_SIZE}/g" \
    -e "s/@KAFKA_BROKERS@/${KAFKA_BROKERS}/g" \
    -e "s/@KAFKA_TOPIC@/${KAFKA_TOPIC}/g" \
    -e "s/@KAFKA_THREADS@/${KAFKA_THREADS}/g" \
    -e "s/@KAFKA_COALESCE@/${KAFKA_COALESCE}/g" \
    -e "s/@KAFKA_TIMEOUT@/${KAFKA_TIMEOUT}/g" \
    -e "s/@GELF_EXTRA_1@/${GELF_EXTRA_1}/g" \
    -e "s/@GELF_EXTRA_2@/${GELF_EXTRA_2}/g" \
    -e "s/@GELF_EXTRA_3@/${GELF_EXTRA_3}/g" \
    /opt/flowgger/etc/flowgger.toml.in > \
    /opt/flowgger/etc/flowgger.toml && \
    cat /opt/flowgger/etc/flowgger.toml

exec /opt/flowgger/bin/flowgger /opt/flowgger/etc/flowgger.toml
