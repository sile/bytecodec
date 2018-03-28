extern crate byteorder;
#[macro_use]
extern crate trackable;

pub use chain::{StartDecoderChain, StartEncoderChain};
pub use decode::{BoxDecoder, Decode, DecodeBuf, DecodeExt};
pub use encode::{BoxEncoder, Encode, EncodeBuf, EncodeExt};
pub use error::{Error, ErrorKind};

pub mod bytes_codec;
pub mod combinator;
pub mod fixnum_codec;
pub mod maker;

mod chain;
mod decode;
mod encode;
mod error;

pub type Result<T> = std::result::Result<T, Error>;
