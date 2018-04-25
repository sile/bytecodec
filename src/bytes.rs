//! Encoders and decoders for reading/writing byte sequences.
use std::cmp;
use std::mem;
use trackable::error::ErrorKindExt;

use {ByteCount, Decode, Encode, Eos, ErrorKind, ExactBytesEncode, Result};

/// `BytesEncoder` writes the given bytes into an output byte sequence.
///
/// # Examples
///
/// ```
/// use bytecodec::{Encode, EncodeExt};
/// use bytecodec::bytes::BytesEncoder;
/// use bytecodec::io::IoEncodeExt;
///
/// let mut output = Vec::new();
/// let mut encoder = BytesEncoder::with_item(b"foo").unwrap();
/// encoder.encode_all(&mut output).unwrap();
/// assert!(encoder.is_idle());
/// assert_eq!(output, b"foo");
/// ```
#[derive(Debug)]
pub struct BytesEncoder<B> {
    bytes: Option<B>,
    offset: usize,
}
impl<B> BytesEncoder<B> {
    /// Makes a new `BytesEncoder` instance.
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

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut size = 0;
        let drop_item = if let Some(ref b) = self.bytes {
            size = cmp::min(buf.len(), b.as_ref().len() - self.offset);
            (&mut buf[..size]).copy_from_slice(&b.as_ref()[self.offset..][..size]);
            self.offset += size;
            if self.offset == b.as_ref().len() {
                true
            } else {
                track_assert!(!eos.is_reached(), ErrorKind::UnexpectedEos;
                              buf.len(), size, self.offset, b.as_ref().len());
                false
            }
        } else {
            false
        };
        if drop_item {
            self.bytes = None;
        }
        Ok(size)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track_assert!(self.is_idle(), ErrorKind::EncoderFull);
        self.bytes = Some(item);
        self.offset = 0;
        Ok(())
    }

    fn requiring_bytes(&self) -> ByteCount {
        ByteCount::Finite(self.exact_requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.bytes.is_none()
    }
}
impl<B: AsRef<[u8]>> ExactBytesEncode for BytesEncoder<B> {
    fn exact_requiring_bytes(&self) -> u64 {
        self.bytes
            .as_ref()
            .map_or(0, |b| b.as_ref().len() - self.offset) as u64
    }
}

/// A variant of `BytesDecoder` for copyable bytes types.
///
/// Unlike `BytesDecoder`, this has no restriction on decoding count.
///
/// # Examples
///
/// ```
/// use bytecodec::{Decode, Eos};
/// use bytecodec::bytes::CopyableBytesDecoder;
///
/// let mut decoder = CopyableBytesDecoder::new([0; 3]);
/// let mut input = b"foobar";
///
/// // Decodes first item
/// assert_eq!(decoder.requiring_bytes().to_u64(), Some(3));
/// let (_, item) = decoder.decode(&input[0..3], Eos::new(false)).unwrap();
/// assert_eq!(item.as_ref(), Some(b"foo"));
///
/// // Decodes second item
/// assert_eq!(decoder.requiring_bytes().to_u64(), Some(3));
/// let (_, item) = decoder.decode(&input[3..5], Eos::new(false)).unwrap();
/// assert_eq!(item, None);
/// assert_eq!(decoder.requiring_bytes().to_u64(), Some(1));
///
/// let (_, item) = decoder.decode(&input[5..], Eos::new(true)).unwrap();
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

    /// Returns a reference to the inner bytes.
    pub fn inner_ref(&self) -> &B {
        &self.bytes
    }

    /// Returns a mutable reference to the inner bytes.
    pub fn inner_mut(&mut self) -> &mut B {
        &mut self.bytes
    }

    /// Takes ownership of this instance and returns the inner bytes.
    pub fn into_inner(self) -> B {
        self.bytes
    }
}
impl<B: AsRef<[u8]> + AsMut<[u8]> + Copy> Decode for CopyableBytesDecoder<B> {
    type Item = B;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        let size = cmp::min(buf.len(), self.bytes.as_ref().len() - self.offset);
        (&mut self.bytes.as_mut()[self.offset..][..size]).copy_from_slice(&buf[..size]);
        self.offset += size;

        if self.offset == self.bytes.as_mut().len() {
            self.offset = 0;
            Ok((size, Some(self.bytes)))
        } else if eos.is_reached() {
            track_assert_eq!(self.offset, 0, ErrorKind::UnexpectedEos);
            Ok((size, None))
        } else {
            Ok((size, None))
        }
    }

    fn has_terminated(&self) -> bool {
        false
    }

    fn requiring_bytes(&self) -> ByteCount {
        ByteCount::Finite((self.bytes.as_ref().len() - self.offset) as u64)
    }
}

/// `BytesDecoder` copies bytes from an input sequence to a slice.
///
/// This is a oneshot decoder (i.e., it decodes only one item).
///
/// # Examples
///
/// ```
/// use bytecodec::Decode;
/// use bytecodec::bytes::BytesDecoder;
/// use bytecodec::io::IoDecodeExt;
///
/// let mut decoder = BytesDecoder::new([0; 3]);
/// assert_eq!(decoder.requiring_bytes().to_u64(), Some(3));
///
/// let item = decoder.decode_exact(b"foobar".as_ref()).unwrap();
/// assert_eq!(item.as_ref(), b"foo");
/// assert_eq!(decoder.requiring_bytes().to_u64(), Some(0)); // no more items are decoded
/// ```
#[derive(Debug)]
pub struct BytesDecoder<B> {
    bytes: Option<B>,
    offset: usize,
}
impl<B: AsRef<[u8]> + AsMut<[u8]>> BytesDecoder<B> {
    /// Makes a new `BytesDecoder` instance for filling the given byte slice.
    pub fn new(bytes: B) -> Self {
        BytesDecoder {
            bytes: Some(bytes),
            offset: 0,
        }
    }

    fn exact_requiring_bytes(&self) -> u64 {
        self.bytes
            .as_ref()
            .map_or(0, |b| b.as_ref().len() - self.offset) as u64
    }

    fn buf_len(&self) -> usize {
        self.bytes.as_ref().map_or(0, |b| b.as_ref().len())
    }
}
impl<B: AsRef<[u8]> + AsMut<[u8]>> Decode for BytesDecoder<B> {
    type Item = B;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        let size = {
            let mut bytes = track_assert_some!(self.bytes.as_mut(), ErrorKind::DecoderTerminated);
            let size = cmp::min(buf.len(), bytes.as_ref().len() - self.offset);
            (&mut bytes.as_mut()[self.offset..][..size]).copy_from_slice(&buf[..size]);
            self.offset += size;
            size
        };
        if self.exact_requiring_bytes() == 0 {
            Ok((size, self.bytes.take()))
        } else {
            track_assert!(!eos.is_reached(), ErrorKind::UnexpectedEos; self.offset, self.buf_len());
            Ok((size, None))
        }
    }

    fn has_terminated(&self) -> bool {
        self.bytes.is_none()
    }

    fn requiring_bytes(&self) -> ByteCount {
        ByteCount::Finite(self.exact_requiring_bytes())
    }
}

/// `RemainingBytesDecoder` reads all the bytes from a input sequence until it reaches EOS.
///
/// # Examples
///
/// ```
/// use bytecodec::{Decode, Eos};
/// use bytecodec::bytes::RemainingBytesDecoder;
///
/// let mut decoder = RemainingBytesDecoder::new();
/// assert_eq!(decoder.requiring_bytes().to_u64(), None);
///
/// let (size, item) = decoder.decode(b"foo", Eos::new(false)).unwrap();
/// assert_eq!(item, None);
/// assert_eq!(size, 3);
///
/// let (size, item) = decoder.decode(b"bar", Eos::new(true)).unwrap();
/// assert_eq!(item, Some(b"foobar".to_vec()));
/// assert_eq!(size, 3);
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

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        if let Some(remaining) = eos.remaining_bytes().to_u64() {
            self.0.reserve_exact(buf.len() + remaining as usize);
        }

        self.0.extend_from_slice(buf);
        if eos.is_reached() {
            Ok((buf.len(), Some(mem::replace(&mut self.0, Vec::new()))))
        } else {
            Ok((buf.len(), None))
        }
    }

    fn has_terminated(&self) -> bool {
        false
    }

    fn requiring_bytes(&self) -> ByteCount {
        ByteCount::Infinite
    }
}

#[derive(Debug)]
struct Utf8Bytes<T>(T);
impl<T: AsRef<str>> AsRef<[u8]> for Utf8Bytes<T> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref().as_bytes()
    }
}

/// `Utf8Encoder` writes the given Rust string into an output byte sequence.
///
/// # Examples
///
/// ```
/// use bytecodec::{Encode, EncodeExt};
/// use bytecodec::bytes::Utf8Encoder;
/// use bytecodec::io::IoEncodeExt;
///
/// let mut output = Vec::new();
/// let mut encoder = Utf8Encoder::with_item("foo").unwrap();
/// encoder.encode_all(&mut output).unwrap();
/// assert!(encoder.is_idle());
/// assert_eq!(output, b"foo");
/// ```
#[derive(Debug)]
pub struct Utf8Encoder<S = String>(BytesEncoder<Utf8Bytes<S>>);
impl<S> Utf8Encoder<S> {
    /// Makes a new `Utf8Encoder` instance.
    pub fn new() -> Self {
        Utf8Encoder(BytesEncoder::new())
    }
}
impl<S: AsRef<str>> Encode for Utf8Encoder<S> {
    type Item = S;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        track!(self.0.encode(buf, eos))
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track!(self.0.start_encoding(Utf8Bytes(item)))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.0.requiring_bytes()
    }

    fn is_idle(&self) -> bool {
        self.0.is_idle()
    }
}
impl<S: AsRef<str>> ExactBytesEncode for Utf8Encoder<S> {
    fn exact_requiring_bytes(&self) -> u64 {
        self.0.exact_requiring_bytes()
    }
}
impl<S> Default for Utf8Encoder<S> {
    fn default() -> Self {
        Self::new()
    }
}

/// `Utf8Decoder` decodes Rust strings from a input byte sequence.
///
/// # Examples
///
/// ```
/// use bytecodec::{Decode, Eos};
/// use bytecodec::bytes::Utf8Decoder;
///
/// let mut decoder = Utf8Decoder::new();
///
/// let (_, item) = decoder.decode(b"foo", Eos::new(true)).unwrap();
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

    /// Returns a reference to the inner bytes decoder.
    pub fn inner_ref(&self) -> &D {
        &self.0
    }

    /// Returns a mutable reference to the inner bytes decoder.
    pub fn inner_mut(&mut self) -> &mut D {
        &mut self.0
    }

    /// Takes ownership of this instance and returns the inner bytes decoder.
    pub fn into_inner(self) -> D {
        self.0
    }
}
impl<D> Decode for Utf8Decoder<D>
where
    D: Decode<Item = Vec<u8>>,
{
    type Item = String;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        let (size, item) = track!(self.0.decode(buf, eos))?;
        match item {
            Some(b) => {
                let s = track!(String::from_utf8(b).map_err(|e| ErrorKind::InvalidInput.cause(e)))?;
                Ok((size, Some(s)))
            }
            None => Ok((size, None)),
        }
    }

    fn has_terminated(&self) -> bool {
        self.0.has_terminated()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.0.requiring_bytes()
    }
}

#[cfg(test)]
mod test {
    use {Encode, EncodeExt, ErrorKind};
    use io::{IoDecodeExt, IoEncodeExt};
    use super::*;

    #[test]
    fn bytes_decoder_works() {
        let mut decoder = BytesDecoder::new([0; 3]);
        assert_eq!(decoder.requiring_bytes().to_u64(), Some(3));

        let mut input = b"foobar".as_ref();
        let item = decoder.decode_exact(&mut input).unwrap();
        assert_eq!(item.as_ref(), b"foo");
        assert_eq!(decoder.requiring_bytes().to_u64(), Some(0));

        assert_eq!(
            decoder.decode_exact(&mut input).err().map(|e| *e.kind()),
            Some(ErrorKind::DecoderTerminated)
        );
    }

    #[test]
    fn utf8_encoder_works() {
        let mut buf = Vec::new();
        let mut encoder = Utf8Encoder::with_item("foo").unwrap();
        encoder.encode_all(&mut buf).unwrap();
        assert!(encoder.is_idle());
        assert_eq!(buf, b"foo");
    }
}
