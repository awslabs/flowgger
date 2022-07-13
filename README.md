

![Flowgger](https://raw.github.com/awslabs/flowgger/master/flowgger.png)

[![CI Build](https://github.com/awslabs/flowgger/actions/workflows/ci.yml/badge.svg)](https://github.com/awslabs/flowgger/actions/workflows/ci.yml) [![License: BSD2](https://img.shields.io/badge/License-BSD2-brightgreen.svg)](https://github.com/awslabs/flowgger/blob/master/LICENSE)

<a name="0.3.0"></a>
### New major version: 0.3.0 (2022-03-14)

#### Breaking Changes

*   Migrate from chrono to [time](https://docs.rs/time/latest/time/) as per https://rustsec.org/advisories/RUSTSEC-2020-0071
    * String formatting changed from strftime to [time](https://docs.rs/time/latest/time/format_description/index.html) custom formatting - see ```flowgger.toml``` for examples on change

---

Flowgger is a fast, simple and lightweight data collector written in Rust.

It reads log entries over a given protocol, extracts them, decodes them using a
given format, re-encodes them into a different format, and asynchronously pushes
the result into a remote data store.

Flowgger is designed to be:
- Paranoid: it carefully validates input data to prevent injection of
malformed/incomplete records down the chain.
- Safe: written in Rust, without any `unsafe` code.
- Fast: even though messages are systematically parsed and validated, Flowgger
is orders of magnitude faster than Logstash and Fluentd.
- Standalone: it comes as a single executable file, and doesn't require a JVM.

Flowgger supports common input types: stdin, UDP, TCP, TLS and Redis,
as well as multiple input formats: JSON (GELF), LTSV, Cap'n Proto and
RFC5424. Normalized messages can be sent to Kafka, Graylog, to downstream
Flowgger servers, or to other log collectors for further processing.

# [Jump to the Flowgger documentation](https://github.com/awslabs/flowgger/wiki)
