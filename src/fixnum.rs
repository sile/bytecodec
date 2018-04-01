//! Encoders and decoders for numbers which have fixed length binary representation.
use byteorder::{BigEndian, ByteOrder, LittleEndian};

use {Decode, DecodeBuf, Encode, EncodeBuf, ErrorKind, Result};
use bytes::{BytesEncoder, CopyableBytesDecoder};

macro_rules! impl_decode {
    ($ty:ty, $item:ty) => {
        impl Decode for $ty {
            type Item = $item;

            fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
                let item = track!(self.0.decode(buf))?;
                Ok(item.map(Self::decode_item))
            }

            fn has_terminated(&self) -> bool {
                false
            }

            fn requiring_bytes_hint(&self) -> Option<u64> {
                self.0.requiring_bytes_hint()
            }
        }
    }
}

macro_rules! impl_encode {
    ($ty:ty, $item:ty) => {
        impl Encode for $ty {
            type Item = $item;

            fn encode(&mut self, buf: &mut EncodeBuf) -> Result<()> {
                track!(self.0.encode(buf))
            }

            fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
                let mut b = Default::default();
                track!(Self::encode_item(item, &mut b))?;
                track!(self.0.start_encoding(b))
            }

            fn requiring_bytes_hint(&self) -> Option<u64> {
                self.0.requiring_bytes_hint()
            }
        }
    }
}

/// Decoder which decodes `u8` values.
///
/// # Examples
///
/// ```
/// use bytecodec::{Decode, DecodeBuf};
/// use bytecodec::fixnum::U8Decoder;
///
/// let mut decoder = U8Decoder::new();
/// let item = decoder.decode(&mut DecodeBuf::new(&[7][..])).unwrap();
/// assert_eq!(item, Some(7));
/// ```
#[derive(Debug, Default)]
pub struct U8Decoder(CopyableBytesDecoder<[u8; 1]>);
impl U8Decoder {
    /// Makes a new `U8Decoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_item(b: [u8; 1]) -> u8 {
        b[0]
    }
}
impl_decode!(U8Decoder, u8);

#[derive(Debug, Default)]
pub struct U8Encoder(BytesEncoder<[u8; 1]>);
impl U8Encoder {
    /// Makes a new `U8Encoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: u8, b: &mut [u8; 1]) -> Result<()> {
        b[0] = n;
        Ok(())
    }
}
impl_encode!(U8Encoder, u8);

/// Decoder which decodes `i8` values.
///
/// # Examples
///
/// ```
/// use bytecodec::{Decode, DecodeBuf};
/// use bytecodec::fixnum::I8Decoder;
///
/// let mut decoder = I8Decoder::new();
/// let item = decoder.decode(&mut DecodeBuf::new(&[255][..])).unwrap();
/// assert_eq!(item, Some(-1));
/// ```
#[derive(Debug, Default)]
pub struct I8Decoder(CopyableBytesDecoder<[u8; 1]>);
impl I8Decoder {
    /// Makes a new `I8Decoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_item(b: [u8; 1]) -> i8 {
        b[0] as i8
    }
}
impl_decode!(I8Decoder, i8);

#[derive(Debug, Default)]
pub struct I8Encoder(BytesEncoder<[u8; 1]>);
impl I8Encoder {
    /// Makes a new `I8Encoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: i8, b: &mut [u8; 1]) -> Result<()> {
        b[0] = n as u8;
        Ok(())
    }
}
impl_encode!(I8Encoder, i8);

/// Decoder which decodes `u16` values by big-endian byte order.
///
/// # Examples
///
/// ```
/// use bytecodec::{Decode, DecodeBuf};
/// use bytecodec::fixnum::U16beDecoder;
///
/// let mut decoder = U16beDecoder::new();
/// let item = decoder.decode(&mut DecodeBuf::new(&[0x01, 0x02][..])).unwrap();
/// assert_eq!(item, Some(0x0102u16));
/// ```
#[derive(Debug, Default)]
pub struct U16beDecoder(CopyableBytesDecoder<[u8; 2]>);
impl U16beDecoder {
    /// Makes a new `U16beDecoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_item(b: [u8; 2]) -> u16 {
        BigEndian::read_u16(&b)
    }
}
impl_decode!(U16beDecoder, u16);

/// Decoder which decodes `u16` values by little-endian byte order.
///
/// # Examples
///
/// ```
/// use bytecodec::{Decode, DecodeBuf};
/// use bytecodec::fixnum::U16leDecoder;
///
/// let mut decoder = U16leDecoder::new();
/// let item = decoder.decode(&mut DecodeBuf::new(&[0x01, 0x02][..])).unwrap();
/// assert_eq!(item, Some(0x0201u16));
/// ```
#[derive(Debug, Default)]
pub struct U16leDecoder(CopyableBytesDecoder<[u8; 2]>);
impl U16leDecoder {
    /// Makes a new `U16leDecoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_item(b: [u8; 2]) -> u16 {
        LittleEndian::read_u16(&b)
    }
}
impl_decode!(U16leDecoder, u16);

#[derive(Debug, Default)]
pub struct U16beEncoder(BytesEncoder<[u8; 2]>);
impl U16beEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: u16, b: &mut [u8; 2]) -> Result<()> {
        BigEndian::write_u16(b, n);
        Ok(())
    }
}
impl_encode!(U16beEncoder, u16);

#[derive(Debug, Default)]
pub struct U16leEncoder(BytesEncoder<[u8; 2]>);
impl U16leEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: u16, b: &mut [u8; 2]) -> Result<()> {
        LittleEndian::write_u16(b, n);
        Ok(())
    }
}
impl_encode!(U16leEncoder, u16);

/// Decoder which decodes `i16` values by big-endian byte order.
///
/// # Examples
///
/// ```
/// use bytecodec::{Decode, DecodeBuf};
/// use bytecodec::fixnum::I16beDecoder;
///
/// let mut decoder = I16beDecoder::new();
/// let item = decoder.decode(&mut DecodeBuf::new(&[0x01, 0x02][..])).unwrap();
/// assert_eq!(item, Some(0x0102i16));
/// ```
#[derive(Debug, Default)]
pub struct I16beDecoder(CopyableBytesDecoder<[u8; 2]>);
impl I16beDecoder {
    /// Makes a new `I16beDecoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_item(b: [u8; 2]) -> i16 {
        BigEndian::read_i16(&b)
    }
}
impl_decode!(I16beDecoder, i16);

/// Decoder which decodes `i16` values by little-endian byte order.
///
/// # Examples
///
/// ```
/// use bytecodec::{Decode, DecodeBuf};
/// use bytecodec::fixnum::I16leDecoder;
///
/// let mut decoder = I16leDecoder::new();
/// let item = decoder.decode(&mut DecodeBuf::new(&[0x01, 0x02][..])).unwrap();
/// assert_eq!(item, Some(0x0201i16));
/// ```
#[derive(Debug, Default)]
pub struct I16leDecoder(CopyableBytesDecoder<[u8; 2]>);
impl I16leDecoder {
    /// Makes a new `I16leDecoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_item(b: [u8; 2]) -> i16 {
        LittleEndian::read_i16(&b)
    }
}
impl_decode!(I16leDecoder, i16);

#[derive(Debug, Default)]
pub struct I16beEncoder(BytesEncoder<[u8; 2]>);
impl I16beEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: i16, b: &mut [u8; 2]) -> Result<()> {
        BigEndian::write_i16(b, n);
        Ok(())
    }
}
impl_encode!(I16beEncoder, i16);

#[derive(Debug, Default)]
pub struct I16leEncoder(BytesEncoder<[u8; 2]>);
impl I16leEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: i16, b: &mut [u8; 2]) -> Result<()> {
        LittleEndian::write_i16(b, n);
        Ok(())
    }
}
impl_encode!(I16leEncoder, i16);

/// Decoder which decodes unsigned 24-bit integers by big-endian byte order.
///
/// The type of decoded values is `u32`, but the most significant 8-bits always be `0`.
///
/// # Examples
///
/// ```
/// use bytecodec::{Decode, DecodeBuf};
/// use bytecodec::fixnum::U24beDecoder;
///
/// let mut decoder = U24beDecoder::new();
/// let item = decoder.decode(&mut DecodeBuf::new(&[0x01, 0x02, 0x03][..])).unwrap();
/// assert_eq!(item, Some(0x0001_0203u32));
/// ```
#[derive(Debug, Default)]
pub struct U24beDecoder(CopyableBytesDecoder<[u8; 3]>);
impl U24beDecoder {
    /// Makes a new `U24beDecoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_item(b: [u8; 3]) -> u32 {
        BigEndian::read_u24(&b)
    }
}
impl_decode!(U24beDecoder, u32);

/// Decoder which decodes unsigned 24-bit integers by little-endian byte order.
///
/// The type of decoded values is `u32`, but the most significant 8-bits always be `0`.
///
/// # Examples
///
/// ```
/// use bytecodec::{Decode, DecodeBuf};
/// use bytecodec::fixnum::U24leDecoder;
///
/// let mut decoder = U24leDecoder::new();
/// let item = decoder.decode(&mut DecodeBuf::new(&[0x01, 0x02, 0x03][..])).unwrap();
/// assert_eq!(item, Some(0x0003_0201u32));
/// ```
#[derive(Debug, Default)]
pub struct U24leDecoder(CopyableBytesDecoder<[u8; 3]>);
impl U24leDecoder {
    /// Makes a new `U24leDecoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_item(b: [u8; 3]) -> u32 {
        LittleEndian::read_u24(&b)
    }
}
impl_decode!(U24leDecoder, u32);

#[derive(Debug, Default)]
pub struct U24beEncoder(BytesEncoder<[u8; 3]>);
impl U24beEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: u32, b: &mut [u8; 3]) -> Result<()> {
        track_assert!(n <= 0xFF_FFFF, ErrorKind::InvalidInput);
        BigEndian::write_u24(b, n);
        Ok(())
    }
}
impl_encode!(U24beEncoder, u32);

#[derive(Debug, Default)]
pub struct U24leEncoder(BytesEncoder<[u8; 3]>);
impl U24leEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: u32, b: &mut [u8; 3]) -> Result<()> {
        track_assert!(n <= 0xFF_FFFF, ErrorKind::InvalidInput);
        LittleEndian::write_u24(b, n);
        Ok(())
    }
}
impl_encode!(U24leEncoder, u32);

/// Decoder which decodes `u32` values by big-endian byte order.
///
/// # Examples
///
/// ```
/// use bytecodec::{Decode, DecodeBuf};
/// use bytecodec::fixnum::U32beDecoder;
///
/// let mut decoder = U32beDecoder::new();
/// let item = decoder.decode(&mut DecodeBuf::new(&[0x01, 0x02, 0x03, 0x04][..])).unwrap();
/// assert_eq!(item, Some(0x0102_0304u32));
/// ```
#[derive(Debug, Default)]
pub struct U32beDecoder(CopyableBytesDecoder<[u8; 4]>);
impl U32beDecoder {
    /// Makes a new `U32beDecoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_item(b: [u8; 4]) -> u32 {
        BigEndian::read_u32(&b)
    }
}
impl_decode!(U32beDecoder, u32);

/// Decoder which decodes `u32` values by little-endian byte order.
///
/// # Examples
///
/// ```
/// use bytecodec::{Decode, DecodeBuf};
/// use bytecodec::fixnum::U32leDecoder;
///
/// let mut decoder = U32leDecoder::new();
/// let item = decoder.decode(&mut DecodeBuf::new(&[0x01, 0x02, 0x03, 0x04][..])).unwrap();
/// assert_eq!(item, Some(0x0403_0201u32));
/// ```
#[derive(Debug, Default)]
pub struct U32leDecoder(CopyableBytesDecoder<[u8; 4]>);
impl U32leDecoder {
    /// Makes a new `U32leDecoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_item(b: [u8; 4]) -> u32 {
        LittleEndian::read_u32(&b)
    }
}
impl_decode!(U32leDecoder, u32);

#[derive(Debug, Default)]
pub struct U32beEncoder(BytesEncoder<[u8; 4]>);
impl U32beEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: u32, b: &mut [u8; 4]) -> Result<()> {
        BigEndian::write_u32(b, n);
        Ok(())
    }
}
impl_encode!(U32beEncoder, u32);

#[derive(Debug, Default)]
pub struct U32leEncoder(BytesEncoder<[u8; 4]>);
impl U32leEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: u32, b: &mut [u8; 4]) -> Result<()> {
        LittleEndian::write_u32(b, n);
        Ok(())
    }
}
impl_encode!(U32leEncoder, u32);

/// Decoder which decodes `i32` values by big-endian byte order.
///
/// # Examples
///
/// ```
/// use bytecodec::{Decode, DecodeBuf};
/// use bytecodec::fixnum::I32beDecoder;
///
/// let mut decoder = I32beDecoder::new();
/// let item = decoder.decode(&mut DecodeBuf::new(&[0x01, 0x02, 0x03, 0x04][..])).unwrap();
/// assert_eq!(item, Some(0x0102_0304i32));
/// ```
#[derive(Debug, Default)]
pub struct I32beDecoder(CopyableBytesDecoder<[u8; 4]>);
impl I32beDecoder {
    /// Makes a new `I32beDecoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_item(b: [u8; 4]) -> i32 {
        BigEndian::read_i32(&b)
    }
}
impl_decode!(I32beDecoder, i32);

/// Decoder which decodes `i32` values by little-endian byte order.
///
/// # Examples
///
/// ```
/// use bytecodec::{Decode, DecodeBuf};
/// use bytecodec::fixnum::I32leDecoder;
///
/// let mut decoder = I32leDecoder::new();
/// let item = decoder.decode(&mut DecodeBuf::new(&[0x01, 0x02, 0x03, 0x04][..])).unwrap();
/// assert_eq!(item, Some(0x0403_0201i32));
/// ```
#[derive(Debug, Default)]
pub struct I32leDecoder(CopyableBytesDecoder<[u8; 4]>);
impl I32leDecoder {
    /// Makes a new `I32leDecoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_item(b: [u8; 4]) -> i32 {
        LittleEndian::read_i32(&b)
    }
}
impl_decode!(I32leDecoder, i32);

#[derive(Debug, Default)]
pub struct I32beEncoder(BytesEncoder<[u8; 4]>);
impl I32beEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: i32, b: &mut [u8; 4]) -> Result<()> {
        BigEndian::write_i32(b, n);
        Ok(())
    }
}
impl_encode!(I32beEncoder, i32);

#[derive(Debug, Default)]
pub struct I32leEncoder(BytesEncoder<[u8; 4]>);
impl I32leEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: i32, b: &mut [u8; 4]) -> Result<()> {
        LittleEndian::write_i32(b, n);
        Ok(())
    }
}
impl_encode!(I32leEncoder, i32);

/// Decoder which decodes unsigned 40-bit integers by big-endian byte order.
///
/// The type of decoded values is `u64`, but the most significant 24-bits always be `0`.
///
/// # Examples
///
/// ```
/// use bytecodec::{Decode, DecodeBuf};
/// use bytecodec::fixnum::U40beDecoder;
///
/// let mut decoder = U40beDecoder::new();
/// let item = decoder.decode(&mut DecodeBuf::new(&[0x01, 0x02, 0x03, 0x04, 0x05][..])).unwrap();
/// assert_eq!(item, Some(0x0000_0001_0203_0405u64));
/// ```
#[derive(Debug, Default)]
pub struct U40beDecoder(CopyableBytesDecoder<[u8; 5]>);
impl U40beDecoder {
    /// Makes a new `U40beDecoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_item(b: [u8; 5]) -> u64 {
        BigEndian::read_uint(&b, b.len())
    }
}
impl_decode!(U40beDecoder, u64);

/// Decoder which decodes unsigned 40-bit integers by little-endian byte order.
///
/// The type of decoded values is `u64`, but the most significant 24-bits always be `0`.
///
/// # Examples
///
/// ```
/// use bytecodec::{Decode, DecodeBuf};
/// use bytecodec::fixnum::U40leDecoder;
///
/// let mut decoder = U40leDecoder::new();
/// let item = decoder.decode(&mut DecodeBuf::new(&[0x01, 0x02, 0x03, 0x04, 0x05][..])).unwrap();
/// assert_eq!(item, Some(0x0000_0005_0403_0201u64));
/// ```
#[derive(Debug, Default)]
pub struct U40leDecoder(CopyableBytesDecoder<[u8; 5]>);
impl U40leDecoder {
    /// Makes a new `U40leDecoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_item(b: [u8; 5]) -> u64 {
        LittleEndian::read_uint(&b, b.len())
    }
}
impl_decode!(U40leDecoder, u64);

#[derive(Debug, Default)]
pub struct U40beEncoder(BytesEncoder<[u8; 5]>);
impl U40beEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: u64, b: &mut [u8; 5]) -> Result<()> {
        track_assert!(n <= 0xFF_FFFF_FFFF, ErrorKind::InvalidInput);
        let len = b.len();
        BigEndian::write_uint(b, n, len);
        Ok(())
    }
}
impl_encode!(U40beEncoder, u64);

#[derive(Debug, Default)]
pub struct U40leEncoder(BytesEncoder<[u8; 5]>);
impl U40leEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: u64, b: &mut [u8; 5]) -> Result<()> {
        track_assert!(n <= 0xFF_FFFF_FFFF, ErrorKind::InvalidInput);
        let len = b.len();
        BigEndian::write_uint(b, n, len);
        Ok(())
    }
}
impl_encode!(U40leEncoder, u64);

/// Decoder which decodes unsigned 48-bit integers by big-endian byte order.
///
/// The type of decoded values is `u64`, but the most significant 16-bits always be `0`.
///
/// # Examples
///
/// ```
/// use bytecodec::{Decode, DecodeBuf};
/// use bytecodec::fixnum::U48beDecoder;
///
/// let mut decoder = U48beDecoder::new();
/// let item = decoder.decode(&mut DecodeBuf::new(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06][..])).unwrap();
/// assert_eq!(item, Some(0x0000_0102_0304_0506u64));
/// ```
#[derive(Debug, Default)]
pub struct U48beDecoder(CopyableBytesDecoder<[u8; 6]>);
impl U48beDecoder {
    /// Makes a new `U48beDecoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_item(b: [u8; 6]) -> u64 {
        BigEndian::read_uint(&b, b.len())
    }
}
impl_decode!(U48beDecoder, u64);

/// Decoder which decodes unsigned 48-bit integers by little-endian byte order.
///
/// The type of decoded values is `u64`, but the most significant 16-bits always be `0`.
///
/// # Examples
///
/// ```
/// use bytecodec::{Decode, DecodeBuf};
/// use bytecodec::fixnum::U48leDecoder;
///
/// let mut decoder = U48leDecoder::new();
/// let item = decoder.decode(&mut DecodeBuf::new(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06][..])).unwrap();
/// assert_eq!(item, Some(0x0000_0605_0403_0201u64));
/// ```
#[derive(Debug, Default)]
pub struct U48leDecoder(CopyableBytesDecoder<[u8; 6]>);
impl U48leDecoder {
    /// Makes a new `U48leDecoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_item(b: [u8; 6]) -> u64 {
        LittleEndian::read_uint(&b, b.len())
    }
}
impl_decode!(U48leDecoder, u64);

#[derive(Debug, Default)]
pub struct U48beEncoder(BytesEncoder<[u8; 6]>);
impl U48beEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: u64, b: &mut [u8; 6]) -> Result<()> {
        track_assert!(n <= 0xFF_FFFF_FFFF, ErrorKind::InvalidInput);
        let len = b.len();
        BigEndian::write_uint(b, n, len);
        Ok(())
    }
}
impl_encode!(U48beEncoder, u64);

#[derive(Debug, Default)]
pub struct U48leEncoder(BytesEncoder<[u8; 6]>);
impl U48leEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: u64, b: &mut [u8; 6]) -> Result<()> {
        track_assert!(n <= 0xFF_FFFF_FFFF, ErrorKind::InvalidInput);
        let len = b.len();
        BigEndian::write_uint(b, n, len);
        Ok(())
    }
}
impl_encode!(U48leEncoder, u64);

/// Decoder which decodes unsigned 56-bit integers by big-endian byte order.
///
/// The type of decoded values is `u64`, but the most significant 8-bits always be `0`.
///
/// # Examples
///
/// ```
/// use bytecodec::{Decode, DecodeBuf};
/// use bytecodec::fixnum::U56beDecoder;
///
/// let mut decoder = U56beDecoder::new();
/// let item = decoder.decode(&mut DecodeBuf::new(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07][..])).unwrap();
/// assert_eq!(item, Some(0x0001_0203_0405_0607u64));
/// ```
#[derive(Debug, Default)]
pub struct U56beDecoder(CopyableBytesDecoder<[u8; 7]>);
impl U56beDecoder {
    /// Makes a new `U56beDecoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_item(b: [u8; 7]) -> u64 {
        BigEndian::read_uint(&b, b.len())
    }
}
impl_decode!(U56beDecoder, u64);

/// Decoder which decodes unsigned 56-bit integers by little-endian byte order.
///
/// The type of decoded values is `u64`, but the most significant 8-bits always be `0`.
///
/// # Examples
///
/// ```
/// use bytecodec::{Decode, DecodeBuf};
/// use bytecodec::fixnum::U56leDecoder;
///
/// let mut decoder = U56leDecoder::new();
/// let item = decoder.decode(&mut DecodeBuf::new(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07][..])).unwrap();
/// assert_eq!(item, Some(0x0007_0605_0403_0201u64));
/// ```
#[derive(Debug, Default)]
pub struct U56leDecoder(CopyableBytesDecoder<[u8; 7]>);
impl U56leDecoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_item(b: [u8; 7]) -> u64 {
        LittleEndian::read_uint(&b, b.len())
    }
}
impl_decode!(U56leDecoder, u64);

#[derive(Debug, Default)]
pub struct U56beEncoder(BytesEncoder<[u8; 7]>);
impl U56beEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: u64, b: &mut [u8; 7]) -> Result<()> {
        track_assert!(n <= 0xFF_FFFF_FFFF, ErrorKind::InvalidInput);
        let len = b.len();
        BigEndian::write_uint(b, n, len);
        Ok(())
    }
}
impl_encode!(U56beEncoder, u64);

#[derive(Debug, Default)]
pub struct U56leEncoder(BytesEncoder<[u8; 7]>);
impl U56leEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: u64, b: &mut [u8; 7]) -> Result<()> {
        track_assert!(n <= 0xFF_FFFF_FFFF, ErrorKind::InvalidInput);
        let len = b.len();
        BigEndian::write_uint(b, n, len);
        Ok(())
    }
}
impl_encode!(U56leEncoder, u64);

/// Decoder which decodes `u64` values by big-endian byte order.
///
/// # Examples
///
/// ```
/// use bytecodec::{Decode, DecodeBuf};
/// use bytecodec::fixnum::U64beDecoder;
///
/// let mut decoder = U64beDecoder::new();
/// let item = decoder.decode(&mut DecodeBuf::new(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08][..])).unwrap();
/// assert_eq!(item, Some(0x0102_0304_0506_0708u64));
/// ```
#[derive(Debug, Default)]
pub struct U64beDecoder(CopyableBytesDecoder<[u8; 8]>);
impl U64beDecoder {
    /// Makes a new `U64beDecoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_item(b: [u8; 8]) -> u64 {
        BigEndian::read_u64(&b)
    }
}
impl_decode!(U64beDecoder, u64);

/// Decoder which decodes `u64` values by little-endian byte order.
///
/// # Examples
///
/// ```
/// use bytecodec::{Decode, DecodeBuf};
/// use bytecodec::fixnum::U64leDecoder;
///
/// let mut decoder = U64leDecoder::new();
/// let item = decoder.decode(&mut DecodeBuf::new(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08][..])).unwrap();
/// assert_eq!(item, Some(0x0807_0605_0403_0201u64));
/// ```
#[derive(Debug, Default)]
pub struct U64leDecoder(CopyableBytesDecoder<[u8; 8]>);
impl U64leDecoder {
    /// Makes a new `U64leDecoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_item(b: [u8; 8]) -> u64 {
        LittleEndian::read_u64(&b)
    }
}
impl_decode!(U64leDecoder, u64);

#[derive(Debug, Default)]
pub struct U64beEncoder(BytesEncoder<[u8; 8]>);
impl U64beEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: u64, b: &mut [u8; 8]) -> Result<()> {
        BigEndian::write_u64(b, n);
        Ok(())
    }
}
impl_encode!(U64beEncoder, u64);

#[derive(Debug, Default)]
pub struct U64leEncoder(BytesEncoder<[u8; 8]>);
impl U64leEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: u64, b: &mut [u8; 8]) -> Result<()> {
        LittleEndian::write_u64(b, n);
        Ok(())
    }
}
impl_encode!(U64leEncoder, u64);

/// Decoder which decodes `i64` values by big-endian byte order.
///
/// # Examples
///
/// ```
/// use bytecodec::{Decode, DecodeBuf};
/// use bytecodec::fixnum::I64beDecoder;
///
/// let mut decoder = I64beDecoder::new();
/// let item = decoder.decode(&mut DecodeBuf::new(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08][..])).unwrap();
/// assert_eq!(item, Some(0x0102_0304_0506_0708i64));
/// ```
#[derive(Debug, Default)]
pub struct I64beDecoder(CopyableBytesDecoder<[u8; 8]>);
impl I64beDecoder {
    /// Makes a new `I64beDecoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_item(b: [u8; 8]) -> i64 {
        BigEndian::read_i64(&b)
    }
}
impl_decode!(I64beDecoder, i64);

/// Decoder which decodes `i64` values by little-endian byte order.
///
/// # Examples
///
/// ```
/// use bytecodec::{Decode, DecodeBuf};
/// use bytecodec::fixnum::I64leDecoder;
///
/// let mut decoder = I64leDecoder::new();
/// let item = decoder.decode(&mut DecodeBuf::new(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08][..])).unwrap();
/// assert_eq!(item, Some(0x0807_0605_0403_0201i64));
/// ```
#[derive(Debug, Default)]
pub struct I64leDecoder(CopyableBytesDecoder<[u8; 8]>);
impl I64leDecoder {
    /// Makes a new `I64leDecoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_item(b: [u8; 8]) -> i64 {
        LittleEndian::read_i64(&b)
    }
}
impl_decode!(I64leDecoder, i64);

#[derive(Debug, Default)]
pub struct I64beEncoder(BytesEncoder<[u8; 8]>);
impl I64beEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: i64, b: &mut [u8; 8]) -> Result<()> {
        BigEndian::write_i64(b, n);
        Ok(())
    }
}
impl_encode!(I64beEncoder, i64);

#[derive(Debug, Default)]
pub struct I64leEncoder(BytesEncoder<[u8; 8]>);
impl I64leEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: i64, b: &mut [u8; 8]) -> Result<()> {
        LittleEndian::write_i64(b, n);
        Ok(())
    }
}
impl_encode!(I64leEncoder, i64);

#[derive(Debug, Default)]
pub struct F32beDecoder(CopyableBytesDecoder<[u8; 4]>);
impl F32beDecoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_item(b: [u8; 4]) -> f32 {
        BigEndian::read_f32(&b)
    }
}
impl_decode!(F32beDecoder, f32);

#[derive(Debug, Default)]
pub struct F32leDecoder(CopyableBytesDecoder<[u8; 4]>);
impl F32leDecoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_item(b: [u8; 4]) -> f32 {
        LittleEndian::read_f32(&b)
    }
}
impl_decode!(F32leDecoder, f32);

#[derive(Debug, Default)]
pub struct F32beEncoder(BytesEncoder<[u8; 4]>);
impl F32beEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: f32, b: &mut [u8; 4]) -> Result<()> {
        BigEndian::write_f32(b, n);
        Ok(())
    }
}
impl_encode!(F32beEncoder, f32);

#[derive(Debug, Default)]
pub struct F32leEncoder(BytesEncoder<[u8; 4]>);
impl F32leEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: f32, b: &mut [u8; 4]) -> Result<()> {
        LittleEndian::write_f32(b, n);
        Ok(())
    }
}
impl_encode!(F32leEncoder, f32);

#[derive(Debug, Default)]
pub struct F64beDecoder(CopyableBytesDecoder<[u8; 8]>);
impl F64beDecoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_item(b: [u8; 8]) -> f64 {
        BigEndian::read_f64(&b)
    }
}
impl_decode!(F64beDecoder, f64);

#[derive(Debug, Default)]
pub struct F64leDecoder(CopyableBytesDecoder<[u8; 8]>);
impl F64leDecoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_item(b: [u8; 8]) -> f64 {
        LittleEndian::read_f64(&b)
    }
}
impl_decode!(F64leDecoder, f64);

#[derive(Debug, Default)]
pub struct F64beEncoder(BytesEncoder<[u8; 8]>);
impl F64beEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: f64, b: &mut [u8; 8]) -> Result<()> {
        BigEndian::write_f64(b, n);
        Ok(())
    }
}
impl_encode!(F64beEncoder, f64);

#[derive(Debug, Default)]
pub struct F64leEncoder(BytesEncoder<[u8; 8]>);
impl F64leEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: f64, b: &mut [u8; 8]) -> Result<()> {
        LittleEndian::write_f64(b, n);
        Ok(())
    }
}
impl_encode!(F64leEncoder, f64);
