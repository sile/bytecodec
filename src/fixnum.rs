//! Encoders and decoders for numbers which have fixed length binary representation.
use byteorder::{BigEndian, ByteOrder, LittleEndian};

use bytes::{BytesEncoder, CopyableBytesDecoder};
use {ByteCount, Decode, Encode, Eos, ErrorKind, Result, SizedEncode};

macro_rules! impl_decode {
    ($ty:ty, $item:ty) => {
        impl Decode for $ty {
            type Item = $item;

            fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
                track!(self.0.decode(buf, eos))
            }

            fn finish_decoding(&mut self) -> Result<Self::Item> {
                track!(self.0.finish_decoding()).map(Self::decode_item)
            }

            fn requiring_bytes(&self) -> ByteCount {
                self.0.requiring_bytes()
            }
        }
    };
}

macro_rules! impl_encode {
    ($ty:ty, $item:ty) => {
        impl Encode for $ty {
            type Item = $item;

            fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
                track!(self.0.encode(buf, eos))
            }

            fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
                let mut b = Default::default();
                track!(Self::encode_item(item, &mut b))?;
                track!(self.0.start_encoding(b))
            }

            fn requiring_bytes(&self) -> ByteCount {
                self.0.requiring_bytes()
            }

            fn is_idle(&self) -> bool {
                self.0.is_idle()
            }
        }
        impl SizedEncode for $ty {
            fn exact_requiring_bytes(&self) -> u64 {
                self.0.exact_requiring_bytes()
            }
        }
    };
}

/// Decoder which decodes `u8` values.
///
/// # Examples
///
/// ```
/// use bytecodec::Decode;
/// use bytecodec::fixnum::U8Decoder;
/// use bytecodec::io::IoDecodeExt;
///
/// let mut decoder = U8Decoder::new();
/// let item = decoder.decode_exact([7].as_ref()).unwrap();
/// assert_eq!(item, 7);
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

/// Encoder which encodes `u8` values.
///
/// # Examples
///
/// ```
/// use bytecodec::EncodeExt;
/// use bytecodec::fixnum::U8Encoder;
/// use bytecodec::io::IoEncodeExt;
///
/// let mut output = Vec::new();
/// let mut encoder = U8Encoder::with_item(7).unwrap();
/// encoder.encode_all(&mut output).unwrap();
/// assert_eq!(output, [7]);
/// ```
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
/// use bytecodec::Decode;
/// use bytecodec::fixnum::I8Decoder;
/// use bytecodec::io::IoDecodeExt;
///
/// let mut decoder = I8Decoder::new();
/// let item = decoder.decode_exact([255].as_ref()).unwrap();
/// assert_eq!(item, -1);
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

/// Encoder which encodes `i8` values.
///
/// # Examples
///
/// ```
/// use bytecodec::EncodeExt;
/// use bytecodec::fixnum::I8Encoder;
/// use bytecodec::io::IoEncodeExt;
///
/// let mut output = Vec::new();
/// let mut encoder = I8Encoder::with_item(-1).unwrap();
/// encoder.encode_all(&mut output).unwrap();
/// assert_eq!(output, [255]);
/// ```
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
/// use bytecodec::Decode;
/// use bytecodec::fixnum::U16beDecoder;
/// use bytecodec::io::IoDecodeExt;
///
/// let mut decoder = U16beDecoder::new();
/// let item = decoder.decode_exact([0x01, 0x02].as_ref()).unwrap();
/// assert_eq!(item, 0x0102u16);
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
/// use bytecodec::Decode;
/// use bytecodec::fixnum::U16leDecoder;
/// use bytecodec::io::IoDecodeExt;
///
/// let mut decoder = U16leDecoder::new();
/// let item = decoder.decode_exact([0x01, 0x02].as_ref()).unwrap();
/// assert_eq!(item, 0x0201u16);
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

/// Encoder which encodes `u16` values by big-endian byte order.
///
/// # Examples
///
/// ```
/// use bytecodec::EncodeExt;
/// use bytecodec::fixnum::U16beEncoder;
/// use bytecodec::io::IoEncodeExt;
///
/// let mut output = Vec::new();
/// let mut encoder = U16beEncoder::with_item(0x0102).unwrap();
/// encoder.encode_all(&mut output).unwrap();
/// assert_eq!(output, [0x01, 0x02]);
/// ```
#[derive(Debug, Default)]
pub struct U16beEncoder(BytesEncoder<[u8; 2]>);
impl U16beEncoder {
    /// Makes a new `U16beEncoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: u16, b: &mut [u8; 2]) -> Result<()> {
        BigEndian::write_u16(b, n);
        Ok(())
    }
}
impl_encode!(U16beEncoder, u16);

/// Encoder which encodes `u16` values by little-endian byte order.
///
/// # Examples
///
/// ```
/// use bytecodec::EncodeExt;
/// use bytecodec::fixnum::U16leEncoder;
/// use bytecodec::io::IoEncodeExt;
///
/// let mut output = Vec::new();
/// let mut encoder = U16leEncoder::with_item(0x0102).unwrap();
/// encoder.encode_all(&mut output).unwrap();
/// assert_eq!(output, [0x02, 0x01]);
/// ```
#[derive(Debug, Default)]
pub struct U16leEncoder(BytesEncoder<[u8; 2]>);
impl U16leEncoder {
    /// Makes a new `U16leEncoder` instance.
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
/// use bytecodec::Decode;
/// use bytecodec::fixnum::I16beDecoder;
/// use bytecodec::io::IoDecodeExt;
///
/// let mut decoder = I16beDecoder::new();
/// let item = decoder.decode_exact([0x01, 0x02].as_ref()).unwrap();
/// assert_eq!(item, 0x0102i16);
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
/// use bytecodec::Decode;
/// use bytecodec::fixnum::I16leDecoder;
/// use bytecodec::io::IoDecodeExt;
///
/// let mut decoder = I16leDecoder::new();
/// let item = decoder.decode_exact([0x01, 0x02].as_ref()).unwrap();
/// assert_eq!(item, 0x0201i16);
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

/// Encoder which encodes `i16` values by big-endian byte order.
///
/// # Examples
///
/// ```
/// use bytecodec::EncodeExt;
/// use bytecodec::fixnum::I16beEncoder;
/// use bytecodec::io::IoEncodeExt;
///
/// let mut output = Vec::new();
/// let mut encoder = I16beEncoder::with_item(-2).unwrap();
/// encoder.encode_all(&mut output).unwrap();
/// assert_eq!(output, [0xFF, 0xFE]);
/// ```
#[derive(Debug, Default)]
pub struct I16beEncoder(BytesEncoder<[u8; 2]>);
impl I16beEncoder {
    /// Makes a new `I16beEncoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: i16, b: &mut [u8; 2]) -> Result<()> {
        BigEndian::write_i16(b, n);
        Ok(())
    }
}
impl_encode!(I16beEncoder, i16);

/// Encoder which encodes `i16` values by little-endian byte order.
///
/// # Examples
///
/// ```
/// use bytecodec::EncodeExt;
/// use bytecodec::fixnum::I16leEncoder;
/// use bytecodec::io::IoEncodeExt;
///
/// let mut output = Vec::new();
/// let mut encoder = I16leEncoder::with_item(-2).unwrap();
/// encoder.encode_all(&mut output).unwrap();
/// assert_eq!(output, [0xFE, 0xFF]);
/// ```
#[derive(Debug, Default)]
pub struct I16leEncoder(BytesEncoder<[u8; 2]>);
impl I16leEncoder {
    /// Makes a new `I16leEncoder` instance.
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
/// use bytecodec::Decode;
/// use bytecodec::fixnum::U24beDecoder;
/// use bytecodec::io::IoDecodeExt;
///
/// let mut decoder = U24beDecoder::new();
/// let item = decoder.decode_exact([0x01, 0x02, 0x03].as_ref()).unwrap();
/// assert_eq!(item, 0x0001_0203u32);
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
/// use bytecodec::Decode;
/// use bytecodec::fixnum::U24leDecoder;
/// use bytecodec::io::IoDecodeExt;
///
/// let mut decoder = U24leDecoder::new();
/// let item = decoder.decode_exact([0x01, 0x02, 0x03].as_ref()).unwrap();
/// assert_eq!(item, 0x0003_0201u32);
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

/// Encoder which encodes unsigned 24-bit integers by big-endian byte order.
///
/// Although the type of items is `u32`, the most significant 8-bits must be `0`.
///
/// # Examples
///
/// ```
/// use bytecodec::EncodeExt;
/// use bytecodec::fixnum::U24beEncoder;
/// use bytecodec::io::IoEncodeExt;
///
/// let mut output = Vec::new();
/// let mut encoder = U24beEncoder::with_item(0x0001_0203).unwrap();
/// encoder.encode_all(&mut output).unwrap();
/// assert_eq!(output, [0x01, 0x02, 0x03]);
/// ```
#[derive(Debug, Default)]
pub struct U24beEncoder(BytesEncoder<[u8; 3]>);
impl U24beEncoder {
    /// Makes a new `U24beEncoder` instance.
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

/// Encoder which encodes unsigned 24-bit integers by little-endian byte order.
///
/// Although the type of items is `u32`, the most significant 8-bits must be `0`.
///
/// # Examples
///
/// ```
/// use bytecodec::EncodeExt;
/// use bytecodec::fixnum::U24leEncoder;
/// use bytecodec::io::IoEncodeExt;
///
/// let mut output = Vec::new();
/// let mut encoder = U24leEncoder::with_item(0x0001_0203).unwrap();
/// encoder.encode_all(&mut output).unwrap();
/// assert_eq!(output, [0x03, 0x02, 0x01]);
/// ```
#[derive(Debug, Default)]
pub struct U24leEncoder(BytesEncoder<[u8; 3]>);
impl U24leEncoder {
    /// Makes a new `U24leEncoder` instance.
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
/// use bytecodec::Decode;
/// use bytecodec::fixnum::U32beDecoder;
/// use bytecodec::io::IoDecodeExt;
///
/// let mut decoder = U32beDecoder::new();
/// let item = decoder.decode_exact([0x01, 0x02, 0x03, 0x04].as_ref()).unwrap();
/// assert_eq!(item, 0x0102_0304u32);
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
/// use bytecodec::Decode;
/// use bytecodec::fixnum::U32leDecoder;
/// use bytecodec::io::IoDecodeExt;
///
/// let mut decoder = U32leDecoder::new();
/// let item = decoder.decode_exact([0x01, 0x02, 0x03, 0x04].as_ref()).unwrap();
/// assert_eq!(item, 0x0403_0201u32);
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

/// Encoder which encodes `u32` values by big-endian byte order.
///
/// # Examples
///
/// ```
/// use bytecodec::EncodeExt;
/// use bytecodec::fixnum::U32beEncoder;
/// use bytecodec::io::IoEncodeExt;
///
/// let mut output = Vec::new();
/// let mut encoder = U32beEncoder::with_item(0x0102_0304).unwrap();
/// encoder.encode_all(&mut output).unwrap();
/// assert_eq!(output, [0x01, 0x02, 0x03, 0x04]);
/// ```
#[derive(Debug, Default)]
pub struct U32beEncoder(BytesEncoder<[u8; 4]>);
impl U32beEncoder {
    /// Makes a new `U32beEncoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: u32, b: &mut [u8; 4]) -> Result<()> {
        BigEndian::write_u32(b, n);
        Ok(())
    }
}
impl_encode!(U32beEncoder, u32);

/// Encoder which encodes `u32` values by little-endian byte order.
///
/// # Examples
///
/// ```
/// use bytecodec::EncodeExt;
/// use bytecodec::fixnum::U32leEncoder;
/// use bytecodec::io::IoEncodeExt;
///
/// let mut output = Vec::new();
/// let mut encoder = U32leEncoder::with_item(0x0102_0304).unwrap();
/// encoder.encode_all(&mut output).unwrap();
/// assert_eq!(output, [0x04, 0x03, 0x02, 0x01]);
/// ```
#[derive(Debug, Default)]
pub struct U32leEncoder(BytesEncoder<[u8; 4]>);
impl U32leEncoder {
    /// Makes a new `U32leEncoder` instance.
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
/// use bytecodec::Decode;
/// use bytecodec::fixnum::I32beDecoder;
/// use bytecodec::io::IoDecodeExt;
///
/// let mut decoder = I32beDecoder::new();
/// let item = decoder.decode_exact([0x01, 0x02, 0x03, 0x04].as_ref()).unwrap();
/// assert_eq!(item, 0x0102_0304i32);
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
/// use bytecodec::Decode;
/// use bytecodec::fixnum::I32leDecoder;
/// use bytecodec::io::IoDecodeExt;
///
/// let mut decoder = I32leDecoder::new();
/// let item = decoder.decode_exact([0x01, 0x02, 0x03, 0x04].as_ref()).unwrap();
/// assert_eq!(item, 0x0403_0201i32);
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

/// Encoder which encodes `i32` values by big-endian byte order.
///
/// # Examples
///
/// ```
/// use bytecodec::EncodeExt;
/// use bytecodec::fixnum::I32beEncoder;
/// use bytecodec::io::IoEncodeExt;
///
/// let mut output = Vec::new();
/// let mut encoder = I32beEncoder::with_item(-2).unwrap();
/// encoder.encode_all(&mut output).unwrap();
/// assert_eq!(output, [0xFF, 0xFF, 0xFF, 0xFE]);
/// ```
#[derive(Debug, Default)]
pub struct I32beEncoder(BytesEncoder<[u8; 4]>);
impl I32beEncoder {
    /// Makes a new `I32beEncoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: i32, b: &mut [u8; 4]) -> Result<()> {
        BigEndian::write_i32(b, n);
        Ok(())
    }
}
impl_encode!(I32beEncoder, i32);

/// Encoder which encodes `i32` values by little-endian byte order.
///
/// # Examples
///
/// ```
/// use bytecodec::EncodeExt;
/// use bytecodec::fixnum::I32leEncoder;
/// use bytecodec::io::IoEncodeExt;
///
/// let mut output = Vec::new();
/// let mut encoder = I32leEncoder::with_item(-2).unwrap();
/// encoder.encode_all(&mut output).unwrap();
/// assert_eq!(output, [0xFE, 0xFF, 0xFF, 0xFF]);
/// ```
#[derive(Debug, Default)]
pub struct I32leEncoder(BytesEncoder<[u8; 4]>);
impl I32leEncoder {
    /// Makes a new `I32leEncoder` instance.
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
/// use bytecodec::Decode;
/// use bytecodec::fixnum::U40beDecoder;
/// use bytecodec::io::IoDecodeExt;
///
/// let mut decoder = U40beDecoder::new();
/// let item = decoder.decode_exact([0x01, 0x02, 0x03, 0x04, 0x05].as_ref()).unwrap();
/// assert_eq!(item, 0x0000_0001_0203_0405u64);
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
/// use bytecodec::Decode;
/// use bytecodec::fixnum::U40leDecoder;
/// use bytecodec::io::IoDecodeExt;
///
/// let mut decoder = U40leDecoder::new();
/// let item = decoder.decode_exact([0x01, 0x02, 0x03, 0x04, 0x05].as_ref()).unwrap();
/// assert_eq!(item, 0x0000_0005_0403_0201u64);
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

/// Encoder which encodes unsigned 40-bit integers by big-endian byte order.
///
/// Although the type of items is `u64`, the most significant 24-bits must be `0`.
///
/// # Examples
///
/// ```
/// use bytecodec::EncodeExt;
/// use bytecodec::fixnum::U40beEncoder;
/// use bytecodec::io::IoEncodeExt;
///
/// let mut output = Vec::new();
/// let mut encoder = U40beEncoder::with_item(0x0000_0001_0203_0405).unwrap();
/// encoder.encode_all(&mut output).unwrap();
/// assert_eq!(output, [0x01, 0x02, 0x03, 0x04, 0x05]);
/// ```
#[derive(Debug, Default)]
pub struct U40beEncoder(BytesEncoder<[u8; 5]>);
impl U40beEncoder {
    /// Makes a new `U40beEncoder` instance.
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

/// Encoder which encodes unsigned 40-bit integers by little-endian byte order.
///
/// Although the type of items is `u64`, the most significant 24-bits must be `0`.
///
/// # Examples
///
/// ```
/// use bytecodec::EncodeExt;
/// use bytecodec::fixnum::U40leEncoder;
/// use bytecodec::io::IoEncodeExt;
///
/// let mut output = Vec::new();
/// let mut encoder = U40leEncoder::with_item(0x0000_0001_0203_0405).unwrap();
/// encoder.encode_all(&mut output).unwrap();
/// assert_eq!(output, [0x05, 0x04, 0x03, 0x02, 0x01]);
/// ```
#[derive(Debug, Default)]
pub struct U40leEncoder(BytesEncoder<[u8; 5]>);
impl U40leEncoder {
    /// Makes a new `U40leEncoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: u64, b: &mut [u8; 5]) -> Result<()> {
        track_assert!(n <= 0xFF_FFFF_FFFF, ErrorKind::InvalidInput);
        let len = b.len();
        LittleEndian::write_uint(b, n, len);
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
/// use bytecodec::Decode;
/// use bytecodec::fixnum::U48beDecoder;
/// use bytecodec::io::IoDecodeExt;
///
/// let mut decoder = U48beDecoder::new();
/// let item = decoder.decode_exact([0x01, 0x02, 0x03, 0x04, 0x05, 0x06].as_ref()).unwrap();
/// assert_eq!(item, 0x0000_0102_0304_0506u64);
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
/// use bytecodec::Decode;
/// use bytecodec::fixnum::U48leDecoder;
/// use bytecodec::io::IoDecodeExt;
///
/// let mut decoder = U48leDecoder::new();
/// let item = decoder.decode_exact([0x01, 0x02, 0x03, 0x04, 0x05, 0x06].as_ref()).unwrap();
/// assert_eq!(item, 0x0000_0605_0403_0201u64);
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

/// Encoder which encodes unsigned 48-bit integers by big-endian byte order.
///
/// Although the type of items is `u64`, the most significant 16-bits must be `0`.
///
/// # Examples
///
/// ```
/// use bytecodec::EncodeExt;
/// use bytecodec::fixnum::U48beEncoder;
/// use bytecodec::io::IoEncodeExt;
///
/// let mut output = Vec::new();
/// let mut encoder = U48beEncoder::with_item(0x0000_0102_0304_0506).unwrap();
/// encoder.encode_all(&mut output).unwrap();
/// assert_eq!(output, [0x01, 0x02, 0x03, 0x04, 0x05, 0x06]);
/// ```
#[derive(Debug, Default)]
pub struct U48beEncoder(BytesEncoder<[u8; 6]>);
impl U48beEncoder {
    /// Makes a new `U48beEncoder` integers.
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: u64, b: &mut [u8; 6]) -> Result<()> {
        track_assert!(n <= 0xFFFF_FFFF_FFFF, ErrorKind::InvalidInput);
        let len = b.len();
        BigEndian::write_uint(b, n, len);
        Ok(())
    }
}
impl_encode!(U48beEncoder, u64);

/// Encoder which encodes unsigned 48-bit integers by little-endian byte order.
///
/// Although the type of items is `u64`, the most significant 16-bits must be `0`.
///
/// # Examples
///
/// ```
/// use bytecodec::EncodeExt;
/// use bytecodec::fixnum::U48leEncoder;
/// use bytecodec::io::IoEncodeExt;
///
/// let mut output = Vec::new();
/// let mut encoder = U48leEncoder::with_item(0x0000_0102_0304_0506).unwrap();
/// encoder.encode_all(&mut output).unwrap();
/// assert_eq!(output, [0x06, 0x05, 0x04, 0x03, 0x02, 0x01]);
/// ```
#[derive(Debug, Default)]
pub struct U48leEncoder(BytesEncoder<[u8; 6]>);
impl U48leEncoder {
    /// Makes a new `U48leEncoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: u64, b: &mut [u8; 6]) -> Result<()> {
        track_assert!(n <= 0xFFFF_FFFF_FFFF, ErrorKind::InvalidInput);
        let len = b.len();
        LittleEndian::write_uint(b, n, len);
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
/// use bytecodec::Decode;
/// use bytecodec::fixnum::U56beDecoder;
/// use bytecodec::io::IoDecodeExt;
///
/// let mut decoder = U56beDecoder::new();
/// let item = decoder.decode_exact([0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07].as_ref()).unwrap();
/// assert_eq!(item, 0x0001_0203_0405_0607u64);
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
/// use bytecodec::Decode;
/// use bytecodec::fixnum::U56leDecoder;
/// use bytecodec::io::IoDecodeExt;
///
/// let mut decoder = U56leDecoder::new();
/// let item = decoder.decode_exact([0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07].as_ref()).unwrap();
/// assert_eq!(item, 0x0007_0605_0403_0201u64);
/// ```
#[derive(Debug, Default)]
pub struct U56leDecoder(CopyableBytesDecoder<[u8; 7]>);
impl U56leDecoder {
    /// Makes a new `U56leDecoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_item(b: [u8; 7]) -> u64 {
        LittleEndian::read_uint(&b, b.len())
    }
}
impl_decode!(U56leDecoder, u64);

/// Encoder which encodes unsigned 56-bit integers by big-endian byte order.
///
/// Although the type of items is `u64`, the most significant 8-bits must be `0`.
///
/// # Examples
///
/// ```
/// use bytecodec::EncodeExt;
/// use bytecodec::fixnum::U56beEncoder;
/// use bytecodec::io::IoEncodeExt;
///
/// let mut output = Vec::new();
/// let mut encoder = U56beEncoder::with_item(0x0001_0203_0405_0607).unwrap();
/// encoder.encode_all(&mut output).unwrap();
/// assert_eq!(output, [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07]);
/// ```
#[derive(Debug, Default)]
pub struct U56beEncoder(BytesEncoder<[u8; 7]>);
impl U56beEncoder {
    /// Makes a new `U56beEncoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: u64, b: &mut [u8; 7]) -> Result<()> {
        track_assert!(n <= 0xFF_FFFF_FFFF_FFFF, ErrorKind::InvalidInput);
        let len = b.len();
        BigEndian::write_uint(b, n, len);
        Ok(())
    }
}
impl_encode!(U56beEncoder, u64);

/// Encoder which encodes unsigned 56-bit integers by little-endian byte order.
///
/// Although the type of items is `u64`, the most significant 8-bits must be `0`.
///
/// # Examples
///
/// ```
/// use bytecodec::EncodeExt;
/// use bytecodec::fixnum::U56leEncoder;
/// use bytecodec::io::IoEncodeExt;
///
/// let mut output = Vec::new();
/// let mut encoder = U56leEncoder::with_item(0x0001_0203_0405_0607).unwrap();
/// encoder.encode_all(&mut output).unwrap();
/// assert_eq!(output, [0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01]);
/// ```
#[derive(Debug, Default)]
pub struct U56leEncoder(BytesEncoder<[u8; 7]>);
impl U56leEncoder {
    /// Makes a new `U56leEncoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: u64, b: &mut [u8; 7]) -> Result<()> {
        track_assert!(n <= 0xFF_FFFF_FFFF_FFFF, ErrorKind::InvalidInput);
        let len = b.len();
        LittleEndian::write_uint(b, n, len);
        Ok(())
    }
}
impl_encode!(U56leEncoder, u64);

/// Decoder which decodes `u64` values by big-endian byte order.
///
/// # Examples
///
/// ```
/// use bytecodec::Decode;
/// use bytecodec::fixnum::U64beDecoder;
/// use bytecodec::io::IoDecodeExt;
///
/// let mut decoder = U64beDecoder::new();
/// let item = decoder.decode_exact([0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08].as_ref()).unwrap();
/// assert_eq!(item, 0x0102_0304_0506_0708u64);
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
/// use bytecodec::Decode;
/// use bytecodec::fixnum::U64leDecoder;
/// use bytecodec::io::IoDecodeExt;
///
/// let mut decoder = U64leDecoder::new();
/// let item = decoder.decode_exact([0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08].as_ref()).unwrap();
/// assert_eq!(item, 0x0807_0605_0403_0201u64);
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

/// Encoder which encodes `u64` values by big-endian byte order.
///
/// # Examples
///
/// ```
/// use bytecodec::EncodeExt;
/// use bytecodec::fixnum::U64beEncoder;
/// use bytecodec::io::IoEncodeExt;
///
/// let mut output = Vec::new();
/// let mut encoder = U64beEncoder::with_item(0x0102_0304_0506_0708).unwrap();
/// encoder.encode_all(&mut output).unwrap();
/// assert_eq!(output, [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);
/// ```
#[derive(Debug, Default)]
pub struct U64beEncoder(BytesEncoder<[u8; 8]>);
impl U64beEncoder {
    /// Makes a new `U64beEncoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: u64, b: &mut [u8; 8]) -> Result<()> {
        BigEndian::write_u64(b, n);
        Ok(())
    }
}
impl_encode!(U64beEncoder, u64);

/// Encoder which encodes `u64` values by big-endian byte order.
///
/// # Examples
///
/// ```
/// use bytecodec::EncodeExt;
/// use bytecodec::fixnum::U64leEncoder;
/// use bytecodec::io::IoEncodeExt;
///
/// let mut output = Vec::new();
/// let mut encoder = U64leEncoder::with_item(0x0102_0304_0506_0708).unwrap();
/// encoder.encode_all(&mut output).unwrap();
/// assert_eq!(output, [0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01]);
/// ```
#[derive(Debug, Default)]
pub struct U64leEncoder(BytesEncoder<[u8; 8]>);
impl U64leEncoder {
    /// Makes a new `U64leEncoder` instance.
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
/// use bytecodec::Decode;
/// use bytecodec::fixnum::I64beDecoder;
/// use bytecodec::io::IoDecodeExt;
///
/// let mut decoder = I64beDecoder::new();
/// let item = decoder.decode_exact([0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08].as_ref()).unwrap();
/// assert_eq!(item, 0x0102_0304_0506_0708i64);
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
/// use bytecodec::Decode;
/// use bytecodec::fixnum::I64leDecoder;
/// use bytecodec::io::IoDecodeExt;
///
/// let mut decoder = I64leDecoder::new();
/// let item = decoder.decode_exact([0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08].as_ref()).unwrap();
/// assert_eq!(item, 0x0807_0605_0403_0201i64);
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

/// Encoder which encodes `i64` values by big-endian byte order.
///
/// # Examples
///
/// ```
/// use bytecodec::EncodeExt;
/// use bytecodec::fixnum::I64beEncoder;
/// use bytecodec::io::IoEncodeExt;
///
/// let mut output = Vec::new();
/// let mut encoder = I64beEncoder::with_item(-2).unwrap();
/// encoder.encode_all(&mut output).unwrap();
/// assert_eq!(output, [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE]);
/// ```
#[derive(Debug, Default)]
pub struct I64beEncoder(BytesEncoder<[u8; 8]>);
impl I64beEncoder {
    /// Makes a new `I64beEncoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: i64, b: &mut [u8; 8]) -> Result<()> {
        BigEndian::write_i64(b, n);
        Ok(())
    }
}
impl_encode!(I64beEncoder, i64);

/// Encoder which encodes `i64` values by little-endian byte order.
///
/// # Examples
///
/// ```
/// use bytecodec::EncodeExt;
/// use bytecodec::fixnum::I64leEncoder;
/// use bytecodec::io::IoEncodeExt;
///
/// let mut output = Vec::new();
/// let mut encoder = I64leEncoder::with_item(-2).unwrap();
/// encoder.encode_all(&mut output).unwrap();
/// assert_eq!(output, [0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
/// ```
#[derive(Debug, Default)]
pub struct I64leEncoder(BytesEncoder<[u8; 8]>);
impl I64leEncoder {
    /// Makes a new `I64leEncoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: i64, b: &mut [u8; 8]) -> Result<()> {
        LittleEndian::write_i64(b, n);
        Ok(())
    }
}
impl_encode!(I64leEncoder, i64);

/// Decoder which decodes `f32` values by big-endian byte order.
///
/// # Examples
///
/// ```
/// use bytecodec::Decode;
/// use bytecodec::fixnum::F32beDecoder;
/// use bytecodec::io::IoDecodeExt;
///
/// let mut decoder = F32beDecoder::new();
/// let item = decoder.decode_exact([66, 246, 204, 205].as_ref()).unwrap();
/// assert_eq!(item, 123.4);
/// ```
#[derive(Debug, Default)]
pub struct F32beDecoder(CopyableBytesDecoder<[u8; 4]>);
impl F32beDecoder {
    /// Makes a new `F32beDecoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_item(b: [u8; 4]) -> f32 {
        BigEndian::read_f32(&b)
    }
}
impl_decode!(F32beDecoder, f32);

/// Decoder which decodes `f32` values by little-endian byte order.
///
/// # Examples
///
/// ```
/// use bytecodec::Decode;
/// use bytecodec::fixnum::F32leDecoder;
/// use bytecodec::io::IoDecodeExt;
///
/// let mut decoder = F32leDecoder::new();
/// let item = decoder.decode_exact([205, 204, 246, 66].as_ref()).unwrap();
/// assert_eq!(item, 123.4);
/// ```
#[derive(Debug, Default)]
pub struct F32leDecoder(CopyableBytesDecoder<[u8; 4]>);
impl F32leDecoder {
    /// Makes a new `F32leDecoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_item(b: [u8; 4]) -> f32 {
        LittleEndian::read_f32(&b)
    }
}
impl_decode!(F32leDecoder, f32);

/// Encoder which encodes `f32` values by big-endian byte order.
///
/// # Examples
///
/// ```
/// use bytecodec::EncodeExt;
/// use bytecodec::fixnum::F32beEncoder;
/// use bytecodec::io::IoEncodeExt;
///
/// let mut output = Vec::new();
/// let mut encoder = F32beEncoder::with_item(123.4).unwrap();
/// encoder.encode_all(&mut output).unwrap();
/// assert_eq!(output, [66, 246, 204, 205]);
/// ```
#[derive(Debug, Default)]
pub struct F32beEncoder(BytesEncoder<[u8; 4]>);
impl F32beEncoder {
    /// Makes a new `F32beEncoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: f32, b: &mut [u8; 4]) -> Result<()> {
        BigEndian::write_f32(b, n);
        Ok(())
    }
}
impl_encode!(F32beEncoder, f32);

/// Encoder which encodes `f32` values by little-endian byte order.
///
/// # Examples
///
/// ```
/// use bytecodec::EncodeExt;
/// use bytecodec::fixnum::F32leEncoder;
/// use bytecodec::io::IoEncodeExt;
///
/// let mut output = Vec::new();
/// let mut encoder = F32leEncoder::with_item(123.4).unwrap();
/// encoder.encode_all(&mut output).unwrap();
/// assert_eq!(output, [205, 204, 246, 66]);
/// ```
#[derive(Debug, Default)]
pub struct F32leEncoder(BytesEncoder<[u8; 4]>);
impl F32leEncoder {
    /// Makes a new `F32leEncoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: f32, b: &mut [u8; 4]) -> Result<()> {
        LittleEndian::write_f32(b, n);
        Ok(())
    }
}
impl_encode!(F32leEncoder, f32);

/// Decoder which decodes `f64` values by big-endian byte order.
///
/// # Examples
///
/// ```
/// use bytecodec::Decode;
/// use bytecodec::fixnum::F64beDecoder;
/// use bytecodec::io::IoDecodeExt;
///
/// let mut decoder = F64beDecoder::new();
/// let item = decoder.decode_exact([64, 94, 221, 47, 26, 159, 190, 119].as_ref()).unwrap();
/// assert_eq!(item, 123.456);
/// ```
#[derive(Debug, Default)]
pub struct F64beDecoder(CopyableBytesDecoder<[u8; 8]>);
impl F64beDecoder {
    /// Makes a new `F64beDecoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_item(b: [u8; 8]) -> f64 {
        BigEndian::read_f64(&b)
    }
}
impl_decode!(F64beDecoder, f64);

/// Decoder which decodes `f64` values by little-endian byte order.
///
/// # Examples
///
/// ```
/// use bytecodec::Decode;
/// use bytecodec::fixnum::F64leDecoder;
/// use bytecodec::io::IoDecodeExt;
///
/// let mut decoder = F64leDecoder::new();
/// let item = decoder.decode_exact([119, 190, 159, 26, 47, 221, 94, 64].as_ref()).unwrap();
/// assert_eq!(item, 123.456);
/// ```
#[derive(Debug, Default)]
pub struct F64leDecoder(CopyableBytesDecoder<[u8; 8]>);
impl F64leDecoder {
    /// Makes a new `F64leDecoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_item(b: [u8; 8]) -> f64 {
        LittleEndian::read_f64(&b)
    }
}
impl_decode!(F64leDecoder, f64);

/// Encoder which encodes `f64` values by big-endian byte order.
///
/// # Examples
///
/// ```
/// use bytecodec::EncodeExt;
/// use bytecodec::fixnum::F64beEncoder;
/// use bytecodec::io::IoEncodeExt;
///
/// let mut output = Vec::new();
/// let mut encoder = F64beEncoder::with_item(123.456).unwrap();
/// encoder.encode_all(&mut output).unwrap();
/// assert_eq!(output, [64, 94, 221, 47, 26, 159, 190, 119]);
/// ```
#[derive(Debug, Default)]
pub struct F64beEncoder(BytesEncoder<[u8; 8]>);
impl F64beEncoder {
    /// Makes a new `F64beEncoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: f64, b: &mut [u8; 8]) -> Result<()> {
        BigEndian::write_f64(b, n);
        Ok(())
    }
}
impl_encode!(F64beEncoder, f64);

/// Encoder which encodes `f64` values by little-endian byte order.
///
/// # Examples
///
/// ```
/// use bytecodec::EncodeExt;
/// use bytecodec::fixnum::F64leEncoder;
/// use bytecodec::io::IoEncodeExt;
///
/// let mut output = Vec::new();
/// let mut encoder = F64leEncoder::with_item(123.456).unwrap();
/// encoder.encode_all(&mut output).unwrap();
/// assert_eq!(output, [119, 190, 159, 26, 47, 221, 94, 64]);
/// ```
#[derive(Debug, Default)]
pub struct F64leEncoder(BytesEncoder<[u8; 8]>);
impl F64leEncoder {
    /// Makes a new `F64leEncoder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: f64, b: &mut [u8; 8]) -> Result<()> {
        LittleEndian::write_f64(b, n);
        Ok(())
    }
}
impl_encode!(F64leEncoder, f64);

#[cfg(test)]
mod test {
    use super::*;
    use Encode;
    use io::{IoDecodeExt, IoEncodeExt};

    macro_rules! assert_encode_decode {
        ($encoder:ident, $decoder:ident, $item:expr, $bytes:expr) => {
            let mut output = Vec::new();
            let mut encoder = $encoder::new();
            track_try_unwrap!(encoder.start_encoding($item));
            track_try_unwrap!(encoder.encode_all(&mut output));
            assert_eq!(output, $bytes);

            let mut decoder = $decoder::new();
            let item = track_try_unwrap!(decoder.decode_exact(&$bytes[..]));
            assert_eq!(item, $item);
        };
    }

    #[test]
    fn fixnum_works() {
        assert_encode_decode!(U8Encoder, U8Decoder, 7, [7]);
        assert_encode_decode!(I8Encoder, I8Decoder, -1, [255]);
        assert_encode_decode!(U16beEncoder, U16beDecoder, 0x0102, [0x01, 0x02]);
        assert_encode_decode!(U16leEncoder, U16leDecoder, 0x0102, [0x02, 0x01]);
        assert_encode_decode!(I16beEncoder, I16beDecoder, -2, [0xFF, 0xFE]);
        assert_encode_decode!(I16leEncoder, I16leDecoder, -2, [0xFE, 0xFF]);
        assert_encode_decode!(U24beEncoder, U24beDecoder, 0x01_0203, [0x01, 0x02, 0x03]);
        assert_encode_decode!(U24leEncoder, U24leDecoder, 0x01_0203, [0x03, 0x02, 0x01]);
        assert_encode_decode!(
            U32beEncoder,
            U32beDecoder,
            0x0102_0304,
            [0x01, 0x02, 0x03, 0x04]
        );
        assert_encode_decode!(
            U32leEncoder,
            U32leDecoder,
            0x0102_0304,
            [0x04, 0x03, 0x02, 0x01]
        );
        assert_encode_decode!(I32beEncoder, I32beDecoder, -2, [0xFF, 0xFF, 0xFF, 0xFE]);
        assert_encode_decode!(I32leEncoder, I32leDecoder, -2, [0xFE, 0xFF, 0xFF, 0xFF]);
        assert_encode_decode!(
            U40beEncoder,
            U40beDecoder,
            0x01_0203_0405,
            [0x01, 0x02, 0x03, 0x04, 0x05]
        );
        assert_encode_decode!(
            U40leEncoder,
            U40leDecoder,
            0x01_0203_0405,
            [0x05, 0x04, 0x03, 0x02, 0x01]
        );
        assert_encode_decode!(
            U48beEncoder,
            U48beDecoder,
            0x0102_0304_0506,
            [0x01, 0x02, 0x03, 0x04, 0x05, 0x06]
        );
        assert_encode_decode!(
            U48leEncoder,
            U48leDecoder,
            0x0102_0304_0506,
            [0x06, 0x05, 0x04, 0x03, 0x02, 0x01]
        );
        assert_encode_decode!(
            U56beEncoder,
            U56beDecoder,
            0x01_0203_0405_0607,
            [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07]
        );
        assert_encode_decode!(
            U56leEncoder,
            U56leDecoder,
            0x01_0203_0405_0607,
            [0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01]
        );
        assert_encode_decode!(
            U64beEncoder,
            U64beDecoder,
            0x0102_0304_0506_0708,
            [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]
        );
        assert_encode_decode!(
            U64leEncoder,
            U64leDecoder,
            0x0102_0304_0506_0708,
            [0x08, 0x7, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01]
        );
        assert_encode_decode!(
            I64beEncoder,
            I64beDecoder,
            -2,
            [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE]
        );
        assert_encode_decode!(
            I64leEncoder,
            I64leDecoder,
            -2,
            [0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]
        );
        assert_encode_decode!(F32beEncoder, F32beDecoder, -123.4, [194, 246, 204, 205]);
        assert_encode_decode!(F32leEncoder, F32leDecoder, -123.4, [205, 204, 246, 194]);
        assert_encode_decode!(
            F64beEncoder,
            F64beDecoder,
            -123.456,
            [192, 94, 221, 47, 26, 159, 190, 119]
        );
        assert_encode_decode!(
            F64leEncoder,
            F64leDecoder,
            -123.456,
            [119, 190, 159, 26, 47, 221, 94, 192]
        );
    }
}
