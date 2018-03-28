bytecodec
=========

A tiny Rust framework for implementing byte-oriented encoders/decoders.

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
