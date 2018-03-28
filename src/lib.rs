extern crate byteorder;
#[macro_use]
extern crate trackable;

pub use decode::{BoxDecoder, Decode, DecodeBuf, DecodeExt};
pub use encode::{BoxEncoder, Encode, EncodeBuf, EncodeExt};
pub use error::{Error, ErrorKind};

pub mod combinators;
pub mod maker;
pub mod numbers;
pub mod sequences;

mod decode;
mod encode;
mod error;

pub type Result<T> = std::result::Result<T, Error>;
