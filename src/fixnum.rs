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

            fn remaining_bytes(&self) -> Option<u64> {
                self.0.remaining_bytes()
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct U8Decoder(CopyableBytesDecoder<[u8; 1]>);
impl U8Decoder {
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
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: u8, b: &mut [u8; 1]) -> Result<()> {
        b[0] = n;
        Ok(())
    }
}
impl_encode!(U8Encoder, u8);

#[derive(Debug, Default)]
pub struct I8Decoder(CopyableBytesDecoder<[u8; 1]>);
impl I8Decoder {
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
    pub fn new() -> Self {
        Self::default()
    }

    fn encode_item(n: i8, b: &mut [u8; 1]) -> Result<()> {
        b[0] = n as u8;
        Ok(())
    }
}
impl_encode!(I8Encoder, i8);

#[derive(Debug, Default)]
pub struct U16beDecoder(CopyableBytesDecoder<[u8; 2]>);
impl U16beDecoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_item(b: [u8; 2]) -> u16 {
        BigEndian::read_u16(&b)
    }
}
impl_decode!(U16beDecoder, u16);

#[derive(Debug, Default)]
pub struct U16leDecoder(CopyableBytesDecoder<[u8; 2]>);
impl U16leDecoder {
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

#[derive(Debug, Default)]
pub struct I16beDecoder(CopyableBytesDecoder<[u8; 2]>);
impl I16beDecoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_item(b: [u8; 2]) -> i16 {
        BigEndian::read_i16(&b)
    }
}
impl_decode!(I16beDecoder, i16);

#[derive(Debug, Default)]
pub struct I16leDecoder(CopyableBytesDecoder<[u8; 2]>);
impl I16leDecoder {
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

#[derive(Debug, Default)]
pub struct U24beDecoder(CopyableBytesDecoder<[u8; 3]>);
impl U24beDecoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_item(b: [u8; 3]) -> u32 {
        BigEndian::read_u24(&b)
    }
}
impl_decode!(U24beDecoder, u32);

#[derive(Debug, Default)]
pub struct U24leDecoder(CopyableBytesDecoder<[u8; 3]>);
impl U24leDecoder {
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

#[derive(Debug, Default)]
pub struct U32beDecoder(CopyableBytesDecoder<[u8; 4]>);
impl U32beDecoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_item(b: [u8; 4]) -> u32 {
        BigEndian::read_u32(&b)
    }
}
impl_decode!(U32beDecoder, u32);

#[derive(Debug, Default)]
pub struct U32leDecoder(CopyableBytesDecoder<[u8; 4]>);
impl U32leDecoder {
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

#[derive(Debug, Default)]
pub struct I32beDecoder(CopyableBytesDecoder<[u8; 4]>);
impl I32beDecoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_item(b: [u8; 4]) -> i32 {
        BigEndian::read_i32(&b)
    }
}
impl_decode!(I32beDecoder, i32);

#[derive(Debug, Default)]
pub struct I32leDecoder(CopyableBytesDecoder<[u8; 4]>);
impl I32leDecoder {
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

#[derive(Debug, Default)]
pub struct U40beDecoder(CopyableBytesDecoder<[u8; 5]>);
impl U40beDecoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_item(b: [u8; 5]) -> u64 {
        BigEndian::read_uint(&b, b.len())
    }
}
impl_decode!(U40beDecoder, u64);

#[derive(Debug, Default)]
pub struct U40leDecoder(CopyableBytesDecoder<[u8; 5]>);
impl U40leDecoder {
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

#[derive(Debug, Default)]
pub struct U48beDecoder(CopyableBytesDecoder<[u8; 6]>);
impl U48beDecoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_item(b: [u8; 6]) -> u64 {
        BigEndian::read_uint(&b, b.len())
    }
}
impl_decode!(U48beDecoder, u64);

#[derive(Debug, Default)]
pub struct U48leDecoder(CopyableBytesDecoder<[u8; 6]>);
impl U48leDecoder {
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

#[derive(Debug, Default)]
pub struct U56beDecoder(CopyableBytesDecoder<[u8; 7]>);
impl U56beDecoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_item(b: [u8; 7]) -> u64 {
        BigEndian::read_uint(&b, b.len())
    }
}
impl_decode!(U56beDecoder, u64);

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

#[derive(Debug, Default)]
pub struct U64beDecoder(CopyableBytesDecoder<[u8; 8]>);
impl U64beDecoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_item(b: [u8; 8]) -> u64 {
        BigEndian::read_u64(&b)
    }
}
impl_decode!(U64beDecoder, u64);

#[derive(Debug, Default)]
pub struct U64leDecoder(CopyableBytesDecoder<[u8; 8]>);
impl U64leDecoder {
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

#[derive(Debug, Default)]
pub struct I64beDecoder(CopyableBytesDecoder<[u8; 8]>);
impl I64beDecoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_item(b: [u8; 8]) -> i64 {
        BigEndian::read_i64(&b)
    }
}
impl_decode!(I64beDecoder, i64);

#[derive(Debug, Default)]
pub struct I64leDecoder(CopyableBytesDecoder<[u8; 8]>);
impl I64leDecoder {
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
