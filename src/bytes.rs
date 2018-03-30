//! Encoders and decoders for reading/writing byte sequences.
use std::io::{Read, Write};
use std::mem;
use trackable::error::ErrorKindExt;

use {Decode, DecodeBuf, Encode, EncodeBuf, Error, ErrorKind, Result};

#[derive(Debug)]
pub struct BytesEncoder<B> {
    bytes: Option<B>,
    offset: usize,
}
impl<B> BytesEncoder<B> {
    pub fn new() -> Self {
        Self::default()
    }
}
impl<B> Default for BytesEncoder<B> {
    fn default() -> Self {
        BytesEncoder {
            bytes: None,
            offset: 0,
        }
    }
}
impl<B: AsRef<[u8]>> Encode for BytesEncoder<B> {
    type Item = B;

    fn encode(&mut self, buf: &mut EncodeBuf) -> Result<()> {
        if let Some(ref mut b) = self.bytes {
            let size = track!(buf.write(&b.as_ref()[self.offset..]).map_err(Error::from))?;
            self.offset += size;
        }
        Ok(())
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track_assert_eq!(self.remaining_bytes(), Some(0), ErrorKind::Full);
        self.bytes = Some(item);
        self.offset = 0;
        Ok(())
    }

    fn remaining_bytes(&self) -> Option<u64> {
        Some(
            self.bytes
                .as_ref()
                .map_or(0, |b| (b.as_ref().len() - self.offset) as u64),
        )
    }
}

/// A variant of `BytesDecoder` for copyable bytes types.
///
/// Unlike `BytesDecoder`, this has no restriction on decoding count.
///
/// # Examples
///
/// ```
/// use bytecodec::{Decode, DecodeBuf};
/// use bytecodec::bytes::CopyableBytesDecoder;
///
/// let mut decoder = CopyableBytesDecoder::new([0; 3]);
/// let mut input = DecodeBuf::new(b"fooba");
///
/// // Decodes first item
/// assert_eq!(decoder.requiring_bytes_hint(), Some(3));
/// let item = decoder.decode(&mut input).unwrap();
/// assert_eq!(item.as_ref(), Some(b"foo"));
///
/// // Decodes second item
/// assert_eq!(decoder.requiring_bytes_hint(), Some(3));
/// let item = decoder.decode(&mut input).unwrap();
/// assert_eq!(item, None);
/// assert_eq!(decoder.requiring_bytes_hint(), Some(1));
///
/// let mut input = DecodeBuf::new(b"r");
/// let item = decoder.decode(&mut input).unwrap();
/// assert_eq!(item.as_ref(), Some(b"bar"));
/// ```
#[derive(Debug, Default)]
pub struct CopyableBytesDecoder<B> {
    bytes: B,
    offset: usize,
}
impl<B> CopyableBytesDecoder<B> {
    /// Makes a new `CopyableBytesDecoder` instance.
    pub fn new(bytes: B) -> Self {
        CopyableBytesDecoder { bytes, offset: 0 }
    }
}
impl<B: AsRef<[u8]> + AsMut<[u8]> + Copy> Decode for CopyableBytesDecoder<B> {
    type Item = B;

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        let size = track!(
            buf.read(&mut self.bytes.as_mut()[self.offset..])
                .map_err(Error::from)
        )?;
        self.offset += size;

        if self.offset == self.bytes.as_mut().len() {
            self.offset = 0;
            Ok(Some(self.bytes))
        } else {
            track_assert!(!buf.is_eos(), ErrorKind::UnexpectedEos);
            Ok(None)
        }
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        Some((self.bytes.as_ref().len() - self.offset) as u64)
    }
}

/// `BytesDecoder` copies bytes from an input sequence to a slice.
///
/// This is a oneshot decoder (i.e., it decodes only one item).
///
/// # Examples
///
/// ```
/// use bytecodec::{Decode, DecodeBuf};
/// use bytecodec::bytes::BytesDecoder;
///
/// let mut decoder = BytesDecoder::new([0; 3]);
/// assert_eq!(decoder.requiring_bytes_hint(), Some(3));
///
/// let mut input = DecodeBuf::new(b"foobar");
/// let item = decoder.decode(&mut input).unwrap();
/// assert_eq!(item.as_ref(), Some(b"foo"));
/// assert_eq!(decoder.requiring_bytes_hint(), Some(0)); // no more items are decoded
/// ```
#[derive(Debug)]
pub struct BytesDecoder<B> {
    bytes: Option<B>,
    offset: usize,
}
impl<B> BytesDecoder<B> {
    /// Makes a new `BytesDecoder` instance for filling the given byte slice.
    pub fn new(bytes: B) -> Self {
        BytesDecoder {
            bytes: Some(bytes),
            offset: 0,
        }
    }
}
impl<B: AsRef<[u8]> + AsMut<[u8]>> Decode for BytesDecoder<B> {
    type Item = B;

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        if let Some(ref mut bytes) = self.bytes {
            let size = track!(
                buf.read(&mut bytes.as_mut()[self.offset..])
                    .map_err(Error::from)
            )?;
            self.offset += size;
        } else {
            track_panic!(ErrorKind::DecoderTerminated);
        }

        if Some(self.offset) == self.bytes.as_ref().map(|b| b.as_ref().len()) {
            Ok(self.bytes.take())
        } else {
            track_assert!(!buf.is_eos(), ErrorKind::UnexpectedEos);
            Ok(None)
        }
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        let n = self.bytes
            .as_ref()
            .map_or(0, |b| b.as_ref().len() - self.offset);
        Some(n as u64)
    }
}

/// `RemainingBytesDecoder` reads all the bytes from a input sequence until it reaches EOS.
///
/// # Examples
///
/// ```
/// use bytecodec::{Decode, DecodeBuf};
/// use bytecodec::bytes::RemainingBytesDecoder;
///
/// let mut decoder = RemainingBytesDecoder::new();
/// assert_eq!(decoder.requiring_bytes_hint(), None);
///
/// let mut input = DecodeBuf::new(b"foo");
/// let item = decoder.decode(&mut input).unwrap();
/// assert_eq!(item, None);
/// assert!(!input.is_eos());
///
/// let mut input = DecodeBuf::with_remaining_bytes(b"bar", 0);
/// let item = decoder.decode(&mut input).unwrap();
/// assert_eq!(item, Some(b"foobar".to_vec()));
/// assert!(input.is_eos());
/// ```
#[derive(Debug, Default)]
pub struct RemainingBytesDecoder(Vec<u8>);
impl RemainingBytesDecoder {
    /// Makes a new `RemainingBytesDecoder` instance.
    pub fn new() -> Self {
        Self::default()
    }
}
impl Decode for RemainingBytesDecoder {
    type Item = Vec<u8>;

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        if let Some(additional) = buf.remaining_bytes() {
            self.0.reserve_exact(buf.len() + additional as usize);
        }

        track!(buf.read_to_end(&mut self.0).map_err(Error::from))?;
        if buf.is_eos() {
            Ok(Some(mem::replace(&mut self.0, Vec::new())))
        } else {
            Ok(None)
        }
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        None
    }
}

#[derive(Debug)]
struct Utf8Bytes<T>(T);
impl<T: AsRef<str>> AsRef<[u8]> for Utf8Bytes<T> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref().as_bytes()
    }
}

#[derive(Debug)]
pub struct Utf8Encoder<S = String>(BytesEncoder<Utf8Bytes<S>>);
impl<S> Utf8Encoder<S> {
    pub fn new() -> Self {
        Self::default()
    }
}
impl<S: AsRef<str>> Encode for Utf8Encoder<S> {
    type Item = S;

    fn encode(&mut self, buf: &mut EncodeBuf) -> Result<()> {
        track!(self.0.encode(buf))
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track!(self.0.start_encoding(Utf8Bytes(item)))
    }

    fn remaining_bytes(&self) -> Option<u64> {
        self.0.remaining_bytes()
    }
}
impl<S> Default for Utf8Encoder<S> {
    fn default() -> Self {
        Self::new()
    }
}

/// `Utf8Decoder` decodes Rust strings from a input byte sequence.
/// # Examples
///
/// ```
/// use bytecodec::{Decode, DecodeBuf};
/// use bytecodec::bytes::Utf8Decoder;
///
/// let mut decoder = Utf8Decoder::new();
///
/// let mut input = DecodeBuf::with_remaining_bytes(b"foo", 0);
/// let item = decoder.decode(&mut input).unwrap();
/// assert_eq!(item, Some("foo".to_owned()));
/// ```
#[derive(Debug, Default)]
pub struct Utf8Decoder<D = RemainingBytesDecoder>(D);
impl Utf8Decoder<RemainingBytesDecoder> {
    /// Makes a new `Utf8Decoder` that uses `RemainingBytesDecoder` as the internal bytes decoder.
    pub fn new() -> Self {
        Utf8Decoder(RemainingBytesDecoder::new())
    }
}
impl<D> Utf8Decoder<D>
where
    D: Decode<Item = Vec<u8>>,
{
    /// Makes a new `Utf8Decoder` with the given bytes decoder.
    pub fn with_bytes_decoder(bytes_decoder: D) -> Self {
        Utf8Decoder(bytes_decoder)
    }
}
impl<D> Decode for Utf8Decoder<D>
where
    D: Decode<Item = Vec<u8>>,
{
    type Item = String;

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        if let Some(bytes) = track!(self.0.decode(buf))? {
            let s = track!(String::from_utf8(bytes).map_err(|e| ErrorKind::InvalidInput.cause(e)))?;
            Ok(Some(s))
        } else {
            Ok(None)
        }
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        self.0.requiring_bytes_hint()
    }
}

#[cfg(test)]
mod test {
    use {Decode, DecodeBuf, ErrorKind};
    use super::*;

    #[test]
    fn bytes_decoder_works() {
        let mut decoder = BytesDecoder::new([0; 3]);
        assert_eq!(decoder.requiring_bytes_hint(), Some(3));

        let mut input = DecodeBuf::new(b"foobar");
        let item = decoder.decode(&mut input).unwrap();
        assert_eq!(item.as_ref(), Some(b"foo"));
        assert_eq!(decoder.requiring_bytes_hint(), Some(0));

        assert_eq!(
            decoder.decode(&mut input).err().map(|e| *e.kind()),
            Some(ErrorKind::DecoderTerminated)
        );
    }
}
