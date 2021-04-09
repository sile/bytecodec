//! `#[cfg(feature = "json_codec")]` JSON encoder and decoder that use [serde_json] internally.
//!
//! [serde_json]: https://crates.io/crates/serde_json
use crate::monolithic::{MonolithicDecode, MonolithicDecoder, MonolithicEncode, MonolithicEncoder};
use crate::{ByteCount, Decode, Encode, Eos, ErrorKind, Result};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::marker::PhantomData;
use trackable::error::ErrorKindExt;

/// JSON decoder.
///
/// Note that this decodes items monolithically
/// so very large items may impair real-time property of the system.
///
/// # Examples
///
/// ```
/// use bytecodec::{Decode, Eos};
/// use bytecodec::json_codec::JsonDecoder;
/// use serde_json::Value;
///
/// let mut decoder = JsonDecoder::<Value>::new();
///
/// decoder.decode(b"[1, 2", Eos::new(false)).unwrap();
/// decoder.decode(b", 3]", Eos::new(true)).unwrap();
/// let json = decoder.finish_decoding().unwrap();
///
/// assert_eq!(json.to_string(), "[1,2,3]");
/// ```
#[derive(Debug)]
pub struct JsonDecoder<T>(MonolithicDecoder<MonolithicJsonDecoder<T>>)
where
    T: for<'de> Deserialize<'de>;
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

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        track!(self.0.decode(buf, eos))
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        track!(self.0.finish_decoding())
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.0.requiring_bytes()
    }

    fn is_idle(&self) -> bool {
        self.0.is_idle()
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
        track!(serde_json::to_writer(writer, item)
            .map_err(|e| ErrorKind::InvalidInput.cause(e).into()))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::json_codec::JsonDecoder;
    use crate::{Decode, Encode, EncodeExt, Eos};
    use serde::ser::{Serialize, SerializeStruct, Serializer};
    use serde_json::Value;

    #[test]
    fn json_decoder_works() {
        let mut decoder = JsonDecoder::<Value>::new();

        track_try_unwrap!(decoder.decode(b"[1, 2", Eos::new(false)));
        track_try_unwrap!(decoder.decode(b", 3]", Eos::new(true)));
        let json = track_try_unwrap!(decoder.finish_decoding());

        assert_eq!(json.to_string(), "[1,2,3]");
    }

    #[test]
    fn json_encoder_works() {
        let item = (1, 2, 3);

        let mut buf = [0; 7];
        let mut encoder = JsonEncoder::with_item(item).unwrap();
        assert_eq!(encoder.encode(&mut buf[..2], Eos::new(false)).unwrap(), 2);
        assert_eq!(encoder.encode(&mut buf[2..], Eos::new(true)).unwrap(), 5);
        assert_eq!(&buf, b"[1,2,3]");
    }

    #[test]
    fn encode_to_json_string_using_serde_works() {
        #[derive(Debug)]
        struct Item {
            id: u64,
            name: String,
        }
        impl Serialize for Item {
            fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let mut state = serializer.serialize_struct("Item", 2)?;
                state.serialize_field("id", &self.id)?;
                state.serialize_field("name", &self.name)?;
                state.end()
            }
        }

        let item = Item {
            id: 4,
            name: "item4".to_owned(),
        };

        let bytes = JsonEncoder::new().encode_into_bytes(item).unwrap();
        assert_eq!(
            String::from_utf8(bytes).unwrap(),
            r#"{"id":4,"name":"item4"}"#
        );
    }
}
