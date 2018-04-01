#[cfg(feature = "bincode_codec")]
extern crate bincode;
extern crate byteorder;
#[cfg(feature = "serde")]
extern crate serde;
#[cfg(feature = "json_codec")]
extern crate serde_json;
#[macro_use]
extern crate trackable;

pub use buf::{DecodeBuf, EncodeBuf};
pub use chain::{StartDecoderChain, StartEncoderChain};
pub use decode::{Decode, DecodeExt, DecodedValue};
pub use encode::{Encode, EncodeExt, ExactBytesEncode};
pub use error::{Error, ErrorKind};

#[cfg(feature = "bincode_codec")]
pub mod bincode_codec;
pub mod bytes;
pub mod combinator;
pub mod fixnum;
#[cfg(feature = "json_codec")]
pub mod json_codec;
pub mod io;
pub mod monolithic;

mod buf;
mod chain;
mod decode;
mod encode;
mod error;

/// This crate specific `Result` type.
pub type Result<T> = std::result::Result<T, Error>;
