extern crate byteorder;
#[macro_use]
extern crate trackable;

pub use encode::{BoxEncoder, Encode, EncodeBuf, EncodeExt};
pub use error::{Error, ErrorKind};
pub use traits::{BoxDecoder, Decode, DecodeBuf, DecodeExt, MakeDecoder};

pub mod combinators;
pub mod maker;
pub mod numbers;
pub mod sequences;

mod encode;
mod error;
mod traits;

pub type Result<T> = std::result::Result<T, Error>;
