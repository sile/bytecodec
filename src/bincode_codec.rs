//! `#[cfg(feature = "bincode_codec")]` JSON encoder and decoder that use [bincode] internally.
//!
//! [bincode]: https://crates.io/crates/bincode
use std::io::{Read, Write};
use std::marker::PhantomData;
use bincode;
use serde::{Deserialize, Serialize};
use trackable::error::ErrorKindExt;

use {ByteCount, Decode, Encode, Eos, ErrorKind, Result};
use monolithic::{MonolithicDecode, MonolithicDecoder, MonolithicEncode, MonolithicEncoder};

/// Bincode decoder.
///
/// Note that this decodes items monolithically
/// so very large items may impair real-time property of the system.
#[derive(Debug)]
pub struct BincodeDecoder<T>(MonolithicDecoder<MonolithicBincodeDecoder<T>>);
impl<T> BincodeDecoder<T>
where
    T: for<'de> Deserialize<'de>,
{
    /// Makes a new `BincodeDecoder` instance.
    pub fn new() -> Self {
        BincodeDecoder(MonolithicDecoder::new(MonolithicBincodeDecoder::new()))
    }
}
impl<T> Decode for BincodeDecoder<T>
where
    T: for<'de> Deserialize<'de>,
{
    type Item = T;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        track!(self.0.decode(buf, eos))
    }

    fn is_idle(&self) -> bool {
        self.0.is_idle()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.0.requiring_bytes()
    }
}
impl<T> Default for BincodeDecoder<T>
where
    T: for<'de> Deserialize<'de>,
{
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
struct MonolithicBincodeDecoder<T>(PhantomData<T>);
impl<T> MonolithicBincodeDecoder<T> {
    fn new() -> Self {
        MonolithicBincodeDecoder(PhantomData)
    }
}
impl<T> MonolithicDecode for MonolithicBincodeDecoder<T>
where
    T: for<'de> Deserialize<'de>,
{
    type Item = T;

    fn monolithic_decode<R: Read>(&self, reader: R) -> Result<Self::Item> {
        track!(
            bincode::deserialize_from(reader).map_err(|e| ErrorKind::InvalidInput.cause(e).into())
        )
    }
}

/// JSON encoder.
///
/// Note that this encodes items monolithically
/// so very large items may impair real-time property of the system.
#[derive(Debug)]
pub struct BincodeEncoder<T: Serialize>(MonolithicEncoder<MonolithicBincodeEncoder<T>>);
impl<T> BincodeEncoder<T>
where
    T: Serialize,
{
    /// Makes a new `BincodeEncoder` instance.
    pub fn new() -> Self {
        BincodeEncoder(MonolithicEncoder::new(MonolithicBincodeEncoder::new()))
    }
}
impl<T> Encode for BincodeEncoder<T>
where
    T: Serialize,
{
    type Item = T;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        track!(self.0.encode(buf, eos))
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track!(self.0.start_encoding(item))
    }

    fn is_idle(&self) -> bool {
        self.0.is_idle()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.0.requiring_bytes()
    }
}
impl<T> Default for BincodeEncoder<T>
where
    T: Serialize,
{
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
struct MonolithicBincodeEncoder<T>(PhantomData<T>);
impl<T> MonolithicBincodeEncoder<T> {
    fn new() -> Self {
        MonolithicBincodeEncoder(PhantomData)
    }
}
impl<T> MonolithicEncode for MonolithicBincodeEncoder<T>
where
    T: Serialize,
{
    type Item = T;

    fn monolithic_encode<W: Write>(&self, item: &Self::Item, writer: W) -> Result<()> {
        track!(
            bincode::serialize_into(writer, item)
                .map_err(|e| ErrorKind::InvalidInput.cause(e).into())
        )
    }
}

#[cfg(test)]
mod test {
    use EncodeExt;
    use io::{IoDecodeExt, IoEncodeExt};
    use super::*;

    #[test]
    fn bincode_works() {
        let item = (1, Some(2), 3);

        let mut buf = Vec::new();
        let mut encoder = BincodeEncoder::with_item(item).unwrap();
        encoder.encode_all(&mut buf).unwrap();

        let mut decoder = BincodeDecoder::<(u8, Option<u16>, u32)>::new();
        let decoded = decoder.decode_exact(&buf[..]).unwrap();
        assert_eq!(decoded, item);
    }
}
