extern crate byteorder;
#[macro_use]
extern crate trackable;

pub use error::{Error, ErrorKind};
pub use traits::{Decode, DecodeBuf, DecodeExt, Encode, EncodeBuf, EncodeExt, MakeDecoder,
                 MakeEncoder};

pub mod combinators;
pub mod numbers;
pub mod sequences;

mod error;
mod traits;

pub type Result<T> = std::result::Result<T, Error>;
