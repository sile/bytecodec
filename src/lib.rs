extern crate byteorder;
#[macro_use]
extern crate trackable;

pub use buf::{DecodeBuf, EncodeBuf};
pub use chain::{StartDecoderChain, StartEncoderChain};
pub use decode::{Decode, DecodeExt, DecodedValue};
pub use encode::{Encode, EncodeExt, ExactBytesEncode};
pub use error::{Error, ErrorKind};

pub mod bytes;
pub mod combinator;
pub mod fixnum;
pub mod io;

mod buf;
mod chain;
mod decode;
mod encode;
mod error;

/// This crate specific `Result` type.
pub type Result<T> = std::result::Result<T, Error>;
