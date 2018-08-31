//! A tiny framework for implementing encoders/decoders of byte-oriented protocols.
//!
//! # Features
//!
//! - Suited for byte-oriented protocols
//! - Incremental encoding/decoding:
//!   - `Encode` and `Decode` traits support incremental encoding and decoding
//!   - The number of bytes consumed in an execution of `encode/decode` methods
//!     can be completely controlled by the caller
//!   - This property makes it easy to implement,
//!     for example, multi-stream multiplexing, transmittin rate control and asynchronous I/O
//! - Composable:
//!   - By combining multiple encoders (or decoders),
//!     it is easy to build a more complex encoder (or decoder)
//!   - See the examples of `EncodeExt` and `DecodeExt` traits
//! - Reduced number of memory copies:
//!   - In design, only two memory copies are required
//!   - Considering in the decode process,
//!     one is the copy from the actual stream (e.g., TCP socket) to the decoding buffer,
//!     the other one is the copy to construct the item from the buffer.
//! - Supports some [serde] implemention crates:
//!   - Currently [serde_json] and [bincode] are supported (as optional featuers)
//!   - See `json_codec` and `bincode_codec` modules
//! - Easily adapt to synchronous I/O, asynchronous I/O, UDP, etc
//! - Trackable errors:
//!    - By using [trackable] crate, the location where an error occurred can be easily specified
//!    - See `EncodeExt::map_err` and `DecodeExt::map_err` methods
//!
//! [bincode]: https://crates.io/crates/bincode
//! [serde]: https://crates.io/crates/serde
//! [serde_json]: https://crates.io/crates/serde_json
//! [trackable]: https://crates.io/crates/trackable
#![warn(missing_docs)]

#[cfg(feature = "bincode_codec")]
extern crate bincode;
extern crate byteorder;
#[cfg(feature = "serde")]
extern crate serde;
#[cfg(feature = "json_codec")]
extern crate serde_json;
#[macro_use]
extern crate trackable;

pub use byte_count::ByteCount;
pub use decode::{Decode, DecodeExt, TaggedDecode};
pub use encode::{Encode, EncodeExt, SizedEncode};
pub use eos::Eos;
pub use error::{Error, ErrorKind};

#[macro_use]
mod macros;

#[cfg(feature = "bincode_codec")]
pub mod bincode_codec;
pub mod bytes;
pub mod combinator;
pub mod fixnum;
pub mod io;
#[cfg(feature = "json_codec")]
pub mod json_codec;
pub mod marker;
pub mod monolithic;
pub mod null;
pub mod padding;
pub mod slice;
pub mod tuple;

mod byte_count;
mod decode;
mod encode;
mod eos;
mod error;

/// This crate specific `Result` type.
pub type Result<T> = std::result::Result<T, Error>;
