//! `#[cfg(feature = "json_codec")]` JSON encoder and decoder that use [serde_json] internally.
//!
//! [serde_json]: https://crates.io/crates/serde_json
use std::io::{Read, Write};
use std::marker::PhantomData;
use serde::{Deserialize, Serialize};
use serde_json;
use trackable::error::ErrorKindExt;

use {Decode, DecodeBuf, Encode, EncodeBuf, ErrorKind, Result};
use monolithic::{MonolithicDecode, MonolithicDecoder, MonolithicEncode, MonolithicEncoder};

/// JSON decoder.
///
/// Note that this decodes items monolithically
/// so very large items may impair real-time property of the system.
///
/// # Examples
///
/// ```
/// # extern crate bytecodec;
/// # extern crate serde_json;
/// use bytecodec::{Decode, DecodeBuf};
/// use bytecodec::json_codec::JsonDecoder;
/// use serde_json::Value;
///
/// # fn main() {
/// let mut decoder = JsonDecoder::<Value>::new();
///
/// decoder.decode(&mut DecodeBuf::new(b"[1, 2")).unwrap();
/// let json = decoder
///    .decode(&mut DecodeBuf::new_as_eos(b", 3]"))
///    .unwrap()
///    .unwrap();
///
/// assert_eq!(json.to_string(), "[1,2,3]");
/// # }
/// ```
#[derive(Debug)]
pub struct JsonDecoder<T>(MonolithicDecoder<MonolithicJsonDecoder<T>>);
impl<T> JsonDecoder<T>
where
    T: for<'de> Deserialize<'de>,
{
    /// Makes a new `JsonDecoder` instance.
    pub fn new() -> Self {
        JsonDecoder(MonolithicDecoder::new(MonolithicJsonDecoder::new()))
    }
}
impl<T> Decode for JsonDecoder<T>
where
    T: for<'de> Deserialize<'de>,
{
    type Item = T;

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        track!(self.0.decode(buf))
    }

    fn has_terminated(&self) -> bool {
        self.0.has_terminated()
    }

    fn is_idle(&self) -> bool {
        self.0.is_idle()
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        self.0.requiring_bytes_hint()
    }
}
impl<T> Default for JsonDecoder<T>
where
    T: for<'de> Deserialize<'de>,
{
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
struct MonolithicJsonDecoder<T>(PhantomData<T>);
impl<T> MonolithicJsonDecoder<T> {
    fn new() -> Self {
        MonolithicJsonDecoder(PhantomData)
    }
}
impl<T> MonolithicDecode for MonolithicJsonDecoder<T>
where
    T: for<'de> Deserialize<'de>,
{
    type Item = T;

    fn monolithic_decode<R: Read>(&self, reader: R) -> Result<Self::Item> {
        track!(serde_json::from_reader(reader).map_err(|e| ErrorKind::InvalidInput.cause(e).into()))
    }
}

/// JSON encoder.
///
/// Note that this encodes items monolithically
/// so very large items may impair real-time property of the system.
#[derive(Debug)]
pub struct JsonEncoder<T: Serialize>(MonolithicEncoder<MonolithicJsonEncoder<T>>);
impl<T> JsonEncoder<T>
where
    T: Serialize,
{
    /// Makes a new `JsonEncoder` instance.
    pub fn new() -> Self {
        JsonEncoder(MonolithicEncoder::new(MonolithicJsonEncoder::new()))
    }
}
impl<T> Encode for JsonEncoder<T>
where
    T: Serialize,
{
    type Item = T;

    fn encode(&mut self, buf: &mut EncodeBuf) -> Result<()> {
        track!(self.0.encode(buf))
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track!(self.0.start_encoding(item))
    }

    fn is_idle(&self) -> bool {
        self.0.is_idle()
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        self.0.requiring_bytes_hint()
    }
}
impl<T> Default for JsonEncoder<T>
where
    T: Serialize,
{
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
struct MonolithicJsonEncoder<T>(PhantomData<T>);
impl<T> MonolithicJsonEncoder<T> {
    fn new() -> Self {
        MonolithicJsonEncoder(PhantomData)
    }
}
impl<T> MonolithicEncode for MonolithicJsonEncoder<T>
where
    T: Serialize,
{
    type Item = T;

    fn monolithic_encode<W: Write>(&self, item: &Self::Item, writer: W) -> Result<()> {
        track!(
            serde_json::to_writer(writer, item)
                .map_err(|e| ErrorKind::InvalidInput.cause(e).into())
        )
    }
}

#[cfg(test)]
mod test {
    use serde_json::Value;

    use {Decode, DecodeBuf};
    use json_codec::JsonDecoder;

    #[test]
    fn json_decoder_works() {
        let mut decoder = JsonDecoder::<Value>::new();

        decoder.decode(&mut DecodeBuf::new(b"[1, 2")).unwrap();
        let json = decoder
            .decode(&mut DecodeBuf::new_as_eos(b", 3]"))
            .unwrap()
            .unwrap();

        assert_eq!(json.to_string(), "[1,2,3]");
    }
}
