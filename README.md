bytecodec
=========

[![bytecodec](http://meritbadge.herokuapp.com/bytecodec)](https://crates.io/crates/bytecodec)
[![Documentation](https://docs.rs/bytecodec/badge.svg)](https://docs.rs/bytecodec)
[![Build Status](https://travis-ci.org/sile/bytecodec.svg?branch=master)](https://travis-ci.org/sile/bytecodec)
[![Code Coverage](https://codecov.io/gh/sile/bytecodec/branch/master/graph/badge.svg)](https://codecov.io/gh/sile/bytecodec/branch/master)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A tiny Rust framework for implementing byte-oriented encoders/decoders.

[Documentation](https://docs.rs/bytecodec)

- encoder/decoder
- byte-oriented
- incremental
  - multiplex, demultiplex
  - prioritize multiple streams
- composable
- support serde
  - note that serde is inherently monolithic (or synchronous)
    - large objects disturb real time property
  - convenient for small/middle size objects
- independent from I/O
  - Easily adapt to synchronous I/O, asynchronous I/O, UDP, etc
- trackable error
- suboptimal (only one memory copy is necessary)
