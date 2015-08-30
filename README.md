
![Flowgger](https://raw.github.com/jedisct1/flowgger/master/flowgger.png)

Flowgger
========

Flowgger is a fast, simple and lightweight data collector written in Rust.

It reads log entries over a given protocol, decodes them using a given
format, reencodes them into a different format, and asynchronously pushes the
result into a remote data store.

While not providing the same set of features and flexibility as tools
such as [Fluentd](http://www.fluentd.org/) or Logstash, it is orders
of magnitude faster and doesn't require a JVM.

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
tls_compression = false
tls_verify_peer = false
tls_method = "TLSv1.2"
tls_ciphers = "ECDHE-RSA-CHACHA20-POLY1305:ECDHE-RSA-AES128-GCM-SHA256"
queuesize = 1000000
```

The currently supported values for the input `type` are `syslog-tcp` (text-based
syslog messages over a TCP socket) and `syslog-tls` (text-based syslog messages
over TLS).

When using TLS, `tls_ciphers` is optional and defaults to a safe suite, but
`tls_cert` and `tls_key` are required.

`tls_compression` is `false` by default, but might be turned on if saving
bandwidth is more important than CPU cycles.

Up to `queuesize` messages may be buffered in memory if the final datastore
cannot keep up with the input rate.

The currently supported input `format` types are `rfc5424`, `gelf` and `ltsv`:

* [RFC 5424](https://tools.ietf.org/html/rfc5424),
* [GELF](https://www.graylog.org/resources/gelf-2/)
* [LTSV](http://ltsv.org)

### RFC5424

Record example:

    <23>1 2015-08-05T15:53:45.637824Z testhostname appname 69 42 [origin@123 software="test script" swVersion="0.0.1"] test message

RFC 5424 messages are assumed to be on a single line and to be made of valid
UTF8 sequences.

Structured data are optional, but supported. The above example includes two
key-value pairs as structured data: `(software, test script)` and
`(swVersion, 0.0.1)`.
Pay attention to the fact that RFC 5424 requires structured data values requires
proper escaping: a `\` character should be prepended to `]`, `"` and `\\`
characters (not bytes, due to UTF-8 encoding).

Messages can optionally be framed, i.e. prepend the length of the message before
each message. This depends on the configuration of the log producer.

In order to disable/enable framing, a `framed` property should be added to the
`[input]` section of the Flowgger config file:

    framed = false

### GELF

Record example:

    {"version":"1.1", "host": "example.org", "short_message": "A short message that helps you identify what is going on", "full_message": "Backtrace here\n\nmore stuff", "timestamp": 1385053862.3072, "level": 1, "_user_id": 9001, "_some_info": "foo", "_some_env_var": "bar"}

The GELF codec doesn't support compression nor chunking.
Chunking is useless with TCP, and compression can be better handled by the TLS
layer.

Versions 1.0 and 1.1 of the GELF protocol are supported. As required by the
specification, the `host` and `short_message` properties are mandatory.

As a log collector and not a log producer, Flowgger also makes the `timestamp`
property mandatory.

Values can be of any type, including booleans and `null` values.

### LTSV

Record example:

    host:127.0.0.1<TAB>ident:-<TAB>user:frank<TAB>time:[10/Oct/2015:13:55:36 -0700]<TAB>req:GET /apache_pb.gif HTTP/1.0<TAB>status:200<TAB>size:2326<TAB>referer:http://www.example.com/start.html

LTSV is especially designed for structured data, and is faster to
parse than RFC 5424 and GELF.

From a producer perspective, LTSV is extremely simple to implement.

The timestamp (the `time` property) can be in RFC 3339 format (preferred) or in
English format with the timezone.

This timestamp, as well as the `host` property, are mandatory.

Records may include a special property named `message`, which contains a
human-readable description of the event (equivalent to `short_message` in GELF
or to the final, non-structured message in RFC 5424).

`level` is another optional, special property, that can be used to provide the
syslog severity level. It should be between 0 and 7.

#### LTSV schema

By design, and unlike JSON-based formats, values in LTSV records are not typed,
and are assumed to be strings by default.

However, it may be desirable to enforce type constraints, and to retains the
types when converting LTSV to typed formats such as GELF.

In order to do so, a schema can be defined for LTSV inputs, in an
`[input.ltsv_schema]` section of the Flowgger configuration file:

    [input.ltsv_schema]
    counter = "u64"
    amount = "f64"

Supported types are:

- `string`
- `bool` (boolean value)
- `f64` (floating-point number)
- `i64` (signed integer)
- `u64` (unsigned integer)

Pay attention to the fact that some of these values may not have a
representation in the target format. For example, Javascript, hence JSON, hence
GELF can only represent values up to 2^53-1 without losing precision.

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

After having been decoded, records are reencoded in `format` format. Currently,
only Greylog's GELF is supported.

Structured data from RFC5424 records show up in GELF data as additional fields.

Optionally, additional properties can be added to every GELF record, by
providing a table in a `[output.gelf_extra]` section.

```
[output.gelf_extra]
x-header1 = "zok"
x-header2 = "zik"
```

If no additional properties are required, this section doesn't have to be
present in the configuration file.

The only data stores (`type`) currently supported by Flowgger are
`kafka` and `debug` (which just prints to the screen).

When using Kafka, the output data is dispatched to a pool of `kafka_threads`
workers.

You probably want to keep the number of Kafka threads low. However, increasing
`kafka_coalesce` can drastically improve performance.
With `kafka_coalesce` set to `N`, writes to Kafka will happen in batches of
`N` records.

If your traffic rate is fairly high, setting this to `10000` is a reasonable
ballpark figure. However, Flowgger always waits for a full batch to be buffered
before sending it to Kafka. So, if your incoming traffic rate is low, you
should disable coalescing by setting `kafka_coalesce` to `1`.

`kafka_acks` controls whether Flowgger waits for an acknowledgment from the
Kafka broker after having sent a batch. If you want to favor speed over data
safety, use `kafka_acks = 0`.

What are some use cases for this?
---------------------------------

Currently: injecting massive amounts of non-critical syslog data to an
ElasticSearch cluster, possibly via [Graylog](https://www.graylog.org/).

Other protocols and codecs will be implemented later on.

How efficient is RFC5424?
-------------------------

It's absolutely terrible. If you can, opt for [LTSV](http://ltsv.org) or binary
formats such as [Cap'n Proto](https://capnproto.org/) instead.
