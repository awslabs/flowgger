
![Flowgger](https://raw.github.com/jedisct1/flowgger/master/flowgger.png)

Flowgger
========

Flowgger is a fast, simple and lightweight data collector written in Rust.

It reads log entries over a given protocol, decodes them using a given
format, reencodes them into a different format, and asynchronously pushes the
result into a remote data store.

While not providing the same set of features and flexibility as tools such as
[Fluentd](http://www.fluentd.org/) or Logstash, it is orders of
magnitude faster and doesn't require a JVM.

Compilation and installation
----------------------------

The current code is written for rust-nightly (1.4.0), although it can work on
rust-stable with minor changes.

In addition to rust-nightly, the openssl-dev system package (or LibreSSL) is
required for TLS support.

After having intalled rust-nightly, compile with the usual:

    cargo build --release

And copy the `target/release/flowgger` file anywhere; this is the only file
you need.

Configuration
-------------

Flowgger reads its configuration from a file given as an argument:

    flowgger flowgger.toml

The configuration file consists of two main sections: `[input]` and `[output]`.

Input section
-------------

```toml
[input]
type = "syslog-tls"
listen = "0.0.0.0:6514"
format = "rfc5424"
tls_cert = "flowgger.pem"
tls_key = "flowgger.pem"
tls_ciphers = "ECDHE-RSA-CHACHA20-POLY1305:ECDHE-RSA-AES128-GCM-SHA256"
queuesize = 1000000
```

The currently supported values for the input `type` are `syslog-tcp` (text-based
syslog messages over a TCP socket) and `syslog-tls` (text-based syslog messages
over TLS).

When using TLS, `tls_ciphers` is optional and defaults to a safe suite, but
`tls_cert` and `tls_key` are required.

The only supported `format` for now are `rfc5424` and `ltsv`.

Flowgger supports the [RFC 5424](https://tools.ietf.org/html/rfc5424)
and [LTSV](http://ltsv.org) formats, that support structured data
(key-value pairs).

Messages are assumed to be on a single line and use the UTF8 encoding.

LTSV is especially designed for structured data, and is faster to
parse than RFC 5424. Timestamps (the `time` property) can be in RFC
3339 format (preferred) or in English format.

Up to `queuesize` messages can be buffered in memory if the final datastore
cannot keep up with the input rate.

Output section
--------------

```toml
[output]
type = "kafka"
format = "gelf"
kafka_brokers = [ "192.168.59.103:9092" ]
kafka_topic = "test"
kafka_threads = 1
kafka_coalesce = 1000
kafka_timeout = 300
kafka_acks = 0

[output.gelf_extra]
x-header1 = "zok"
x-header2 = "zik"
```

After having been decoded, records are reencoded in `format`. Currently, only
Greylog's [`gelf` format](https://www.graylog.org/resources/gelf-2/) is supported.

Structured data from RFC5424 records show up in Gelf data as additional fields.

Optionally, additional properties can be added to every Gelf record, by
providing a table in a `[output.gelf_extra]` section. If no additional properties
are required, this section doesn't have to be present in the configuration file.

The only `type` of data store currently supported by Flowgger is `kafka`.

The output data is dispatched to a pool of `kafka_threads` workers.

You probably want to keep the number of Kafka threads low. However, increasing
`kafka_coalesce` can drastically improve performance.
With `kafka_coalesce` set to `N`, writes to Kafka will happen in batches of
`N` records.
If your traffic rate is fairly high, setting this to `10000` is a reasonable
ballpark figure. However, Flowgger always waits for a full batch to be buffered
before sending it to Kafka. So, if your incoming traffic rate is low, you
should disable coalescing by setting `kafka_coalesce` to `1`.

What the use cases for this?
----------------------------

Currently: injecting massive amounts of non-critical syslog data to an
ElasticSearch cluster, possibly via [Graylog](https://www.graylog.org/).

Other protocols and codecs will be implemented later on.

How efficient is RFC5424?
-------------------------

It's absolutely terrible. If you can, opt for [LTSV](http://ltsv.org) or binary
formats such as [Cap'n Proto](https://capnproto.org/) instead.
