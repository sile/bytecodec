extern crate byteorder;
#[macro_use]
extern crate trackable;

pub use chain::{StartDecoderChain, StartEncoderChain};
pub use decode::{BoxDecoder, Decode, DecodeBuf, DecodeExt};
pub use encode::{BoxEncoder, Encode, EncodeBuf, EncodeExt, ExactSizeEncode};
pub use error::{Error, ErrorKind};

pub mod buf;
pub mod bytes_codec;
pub mod combinator;
pub mod fixnum_codec;
pub mod maker;

mod chain;
mod decode;
mod encode;
mod error;

pub type Result<T> = std::result::Result<T, Error>;

// TODO: RingBuffer{to_encode_buf, to_decode_buf}
