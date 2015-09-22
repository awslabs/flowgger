
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
type = "tls"
listen = "0.0.0.0:6514"
format = "ltsv"
framing = "line"
tls_cert = "flowgger.pem"
tls_key = "flowgger.pem"
tls_ca = "flowgger.pem"
tls_compression = false
tls_verify_peer = false
tls_method = "TLSv1.2"
redis_connect = "127.0.0.1"
redis_queue_key = "logs"
redis_threads = 1
queuesize = 1000000
```

The currently supported values for the input `type` are `tcp`
(text-based syslog messages over a TCP socket), `tls` (text-based
syslog messages over TLS), `redis` (Redis queue), `udp` and `tls_co`.

### TCP

In the configuration file:

```toml
[input]
type = "tcp"
```

TCP accepts plain, uncrypted, unauthenticated messages. It is compatible with
most syslog daemons and other log collectors.

The TCP input assumes that records are separated by line breaks (LF / CR+LF) by
default. However, this can be changed using the `framing` option:

```toml
framing = "line"
```

Supported framing types are:
- `line`: line breaks
- `nul`: NUL characters (usually required for GELF over TCP)
- `syslen`: length-prefixed syslog messages as specified in RFC 5425.

### TLS

```toml
[input]
type = "tls"
```

When using TLS, `tls_ciphers` is optional and defaults to a safe suite, but
`tls_cert` and `tls_key` are required.

A self-signed certificate and key can be created with:

```bash
openssl req -x509 -nodes -newkey rsa:3072 -sha256 \
  -keyout flowgger.pem -out flowgger.pem
```

`tls_compression` is `false` by default, but might be turned on if saving
bandwidth is more important than CPU cycles and logs don't contain
secrets.

Certificate-based client authentication is also supported. In order to use it,
set `tls_verify_peer` to `true`, and add the path to a file containing one or
more client certificates:

```toml
tls_verify_peer = false
tls_ca_file = "flowgger-client.pem"
```

The TCP input assumes that records are separated by line breaks (LF / CR+LF) by
default. However, this can be changed using the `framing` option:

```toml
framing = "line"
```

Supported framing types are:
- `line`: line breaks
- `nul`: NUL characters (usually required for GELF over TCP)
- `syslen`: length-prefixed syslog messages as specified in RFC 5425. However,
line breaks also act as delimiters, in order to recover from corrupted/invalid
entries.

### TLS, using coroutines

With the TCP and TLS inputs, each client connection gets a dedicated thread,
and a dedicated parser. Each connection can thus fully take advantage of a
CPU core.

To max out the performance of a Flowgger instance, incoming messages should
be balanced over persistent connections whose number equals the number of CPU
cores to use. If necessary, an intermediate log collector such as fluentd,
rsyslog or syslog-ng can help achieve this distribution. On a dedicated 4 cores
instance, reserving 1 core for the output module and 3 cores to parse the input
is a reasonable ballpark figure.

Having more persistent connections than available CPU cores is not recommended,
as it will introduce additional context switches.
On bare metal hardware, the overhead may not be significant.
However, virtualized environments can perform poorly, especially on older CPUs
or hypervisors without support for extended page tables. Cheap shared hosting
providers can also enforce a fairly low number of allowed threads per instance.

Flowgger can use an alternative TLS implementation leveraging coroutines in
order to handle an arbitrary number of connections using a fixed number of
threads. This should be used only as a last resort, if adding intermediate
collectors is not an option. If the number of persistent connections is lower
or equal to the number of CPU cores, use the standard implementation. In fact,
you probably never want to use that implementation unless you really want to,
or have to.

Support for this is disabled by default, and requires Flowgger to be compiled
with the "coroutines" feature, which will automatically pull quite a few nasty
dependencies.

```bash
cargo build --features=coroutines --release
```

The coroutines-based implementation of the TLS input processor can be enabled
in the configuration file with:

```toml
[input]
type = "tls_co"
```

Other available properties are the same as the standard TLS input processor,
with the exception of the session timeout, which is ignored in this
implementation.

### Redis

Flowgger can also retrieve messages from a queue speaking the Redis
protocol, such as Redis itself or Ardb:

```toml
type = "redis"
redis_connect = "127.0.0.1"
redis_queue_key = "logs"
redis_threads = 1
```

This uses the Redis reliable queue pattern, moving messages to a
temporary list whose key is `<redis_queue_key>.tmp.<thread number>`.

### Input formats

Up to `queuesize` messages may be buffered in memory if the final datastore
cannot keep up with the input rate.

The currently supported input `format` types are `rfc5424`, `gelf` and `ltsv`:

* [RFC 5424](https://tools.ietf.org/html/rfc5424),
* [JSON (GELF)](https://www.graylog.org/resources/gelf/)
* [LTSV](http://ltsv.org)

### RFC5424

Record example:

    <23>1 2015-08-05T15:53:45.637824Z testhostname appname 69 42 [origin@123 software="test script" swVersion="0.0.1"] test message

RFC 5424 messages are assumed to be on a single line and to be made of valid
UTF8 sequences.

Structured data are optional, but supported. The above example includes two
key-value pairs as structured data: `(software, test script)` and
`(swVersion, 0.0.1)`.

Pay attention to the fact that RFC 5424 requires structured data
values requires proper escaping: a `\` character should be prepended
to `]`, `"` and `\\` characters (not bytes, due to UTF-8 encoding).

RFC 5424 messages commonly prefix messages by the length, whose support can
be enabled using `framing = "syslen"`.

### JSON (GELF)

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

#### Notes on log4perl_gelf

`Log::Log4perl::Layout::GELF` is a module to add GELF support to Log4Perl, a
logging framework for a language called Perl.

Unfortunately, this module:

- Doesn't implement the GELF specification. Lines numbers, timestamps and
severity levels are sent as UTF-8 strings where the specification mentions that
they MUST be numbers.
- Can send UTF-8 strings that cannot be parsed as UTF-8 strings.
- Can send invalid, unparsable JSON.
- Can only send messages over UDP. TCP support is documented but cannot
possibly work with any GELF parsers: messages are concatenated without any
delimiter, and the output of the concatenation gets compressed as a single
chunk, which is unworkable on a persistent TCP connection.
- Hasn't been updated since 2011.

There are no compatibility hacks that Flowgger or any other GELF (or even JSON)
parser could implement in order to reliably support the output of this module
when used with TCP.

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
only Graylog's GELF is supported.

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
