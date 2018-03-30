extern crate byteorder;
#[macro_use]
extern crate trackable;

pub use chain::{StartDecoderChain, StartEncoderChain};
pub use decode::{Decode, DecodeBuf, DecodeExt};
pub use encode::{Encode, EncodeBuf, EncodeExt, ExactSizeEncode};
pub use error::{Error, ErrorKind};

pub mod buf; // TODO: rename (io?)
pub mod bytes;
pub mod combinator;
pub mod fixnum;
// TODO: rename
pub mod marker {
    pub use decode::ExactBytesDecode;
}

mod chain;
mod decode;
mod encode;
mod error;

pub type Result<T> = std::result::Result<T, Error>;
