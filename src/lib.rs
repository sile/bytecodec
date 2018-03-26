extern crate byteorder;
#[macro_use]
extern crate trackable;

pub use error::{Error, ErrorKind};
pub use traits::{Decode, DecodeExt, Encode, EncodeExt, MakeDecoder, MakeEncoder};

pub mod items;
pub mod combinators;
pub mod sequences;

mod error;
mod traits;

pub type Result<T> = std::result::Result<T, Error>;
