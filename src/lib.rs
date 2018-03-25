extern crate byteorder;
#[macro_use]
extern crate trackable;

pub use error::{Error, ErrorKind};
pub use traits::{Decode, Encode, MakeDecoder, MakeEncoder};

pub mod items;

mod error;
mod traits;

pub type Result<T> = std::result::Result<T, Error>;
