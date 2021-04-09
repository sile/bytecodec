bytecodec
=========

[![bytecodec](https://img.shields.io/crates/v/bytecodec.svg)](https://crates.io/crates/bytecodec)
[![Documentation](https://docs.rs/bytecodec/badge.svg)](https://docs.rs/bytecodec)
[![Actions Status](https://github.com/sile/bytecodec/workflows/CI/badge.svg)](https://github.com/sile/bytecodec/actions)
[![Coverage Status](https://coveralls.io/repos/github/sile/bytecodec/badge.svg?branch=main)](https://coveralls.io/github/sile/bytecodec?branch=main)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A tiny Rust framework for implementing encoders/decoders of byte-oriented protocols.

[Documentation](https://docs.rs/bytecodec)

Features
--------

- Suited for byte-oriented protocols
- Incremental encoding/decoding:
  - `Encode` and `Decode` traits support incremental encoding and decoding
  - The number of bytes consumed in an execution of `encode/decode` methods
    can be completely controlled by the caller
  - This property makes it easy to implement,
    for example, multi-stream multiplexing, transmittin rate control and asynchronous I/O
- Composable:
  - By combining multiple encoders (or decoders),
    it is easy to build a more complex encoder (or decoder)
  - See the examples of `EncodeExt` and `DecodeExt` traits
- Reduced number of memory copies:
  - In design, only two memory copies are required
  - Considering in the decode process,
    one is the copy from the actual stream (e.g., TCP socket) to the decoding buffer,
    the other one is the copy to construct the item from the buffer.
- Supports some [serde] implemention crates:
  - Currently [serde_json] and [bincode] are supported (as optional featuers)
  - See `json_codec` and `bincode_codec` modules
- Easily adapt to synchronous I/O, asynchronous I/O, UDP, etc
- Trackable errors:
   - By using [trackable] crate, the location where an error occurred can be easily specified
   - See `EncodeExt::map_err` and `DecodeExt::map_err` methods

[bincode]: https://crates.io/crates/bincode
[serde]: https://crates.io/crates/serde
[serde_json]: https://crates.io/crates/serde_json
[trackable]: https://crates.io/crates/trackable
