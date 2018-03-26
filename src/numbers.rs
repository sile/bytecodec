use byteorder::{BigEndian, ByteOrder, LittleEndian};

use {Decode, Encode, ErrorKind, Result};
use sequences::Bytes;

macro_rules! impl_codec {
    ($ty:ty, $item:ty, $size:expr, $read:expr, $write:expr) => {
        impl Decode for $ty {
            type Item = $item;

            fn decode(&mut self, buf: &[u8], eos: bool) -> Result<usize> {
                track!(self.0.decode(buf, eos))
            }

            fn pop_item(&mut self) -> Result<Option<Self::Item>> {
                let item = track!(self.0.pop_item())?;
                Ok(item.map(|b| $read(&b)))
            }

            fn decode_size_hint(&self) -> Option<usize> {
                self.0.decode_size_hint()
            }
        }

        impl Encode for $ty {
            type Item = $item;

            fn encode(&mut self, buf: &mut [u8]) -> Result<usize> {
                track!(self.0.encode(buf))
            }

            fn push_item(&mut self, item: Self::Item) -> Result<Option<Self::Item>> {
                let mut b = [0; $size];
                $write(&mut b, item);
                track!(self.0.push_item(b).map(|r| r.map(|_| item)))
            }

            fn encode_size_hint(&self) -> Option<usize> {
                self.0.encode_size_hint()
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct U8(Option<u8>);
impl U8 {
    pub fn new(v: u8) -> Self {
        U8(Some(v))
    }

    pub fn empty() -> Self {
        Self::default()
    }
}
impl Decode for U8 {
    type Item = u8;

    fn decode(&mut self, buf: &[u8], _eos: bool) -> Result<usize> {
        if self.0.is_none() {
            track_assert_ne!(buf.len(), 0, ErrorKind::InvalidInput);
            self.0 = Some(buf[0]);
            Ok(1)
        } else {
            Ok(0)
        }
    }

    fn pop_item(&mut self) -> Result<Option<Self::Item>> {
        Ok(self.0.take())
    }

    fn decode_size_hint(&self) -> Option<usize> {
        Some(self.0.map(|_| 0).unwrap_or(1))
    }
}
impl Encode for U8 {
    type Item = u8;

    fn encode(&mut self, buf: &mut [u8]) -> Result<usize> {
        if let Some(v) = self.0.take() {
            track_assert_ne!(buf.len(), 0, ErrorKind::InvalidInput);
            buf[0] = v;
            Ok(1)
        } else {
            Ok(0)
        }
    }

    fn push_item(&mut self, item: Self::Item) -> Result<Option<Self::Item>> {
        if self.0.is_none() {
            self.0 = Some(item);
            Ok(None)
        } else {
            Ok(Some(item))
        }
    }

    fn encode_size_hint(&self) -> Option<usize> {
        Some(self.0.map(|_| 1).unwrap_or(0))
    }
}

#[derive(Debug, Default)]
pub struct I8(Option<i8>);
impl I8 {
    pub fn new(v: i8) -> Self {
        I8(Some(v))
    }

    pub fn empty() -> Self {
        Self::default()
    }
}
impl Decode for I8 {
    type Item = i8;

    fn decode(&mut self, buf: &[u8], _eos: bool) -> Result<usize> {
        if self.0.is_none() {
            track_assert_ne!(buf.len(), 0, ErrorKind::InvalidInput);
            self.0 = Some(buf[0] as i8);
            Ok(1)
        } else {
            Ok(0)
        }
    }

    fn pop_item(&mut self) -> Result<Option<Self::Item>> {
        Ok(self.0.take())
    }

    fn decode_size_hint(&self) -> Option<usize> {
        Some(self.0.map(|_| 0).unwrap_or(1))
    }
}
impl Encode for I8 {
    type Item = i8;

    fn encode(&mut self, buf: &mut [u8]) -> Result<usize> {
        if let Some(v) = self.0.take() {
            track_assert_ne!(buf.len(), 0, ErrorKind::InvalidInput);
            buf[0] = v as u8;
            Ok(1)
        } else {
            Ok(0)
        }
    }

    fn push_item(&mut self, item: Self::Item) -> Result<Option<Self::Item>> {
        if self.0.is_none() {
            self.0 = Some(item);
            Ok(None)
        } else {
            Ok(Some(item))
        }
    }

    fn encode_size_hint(&self) -> Option<usize> {
        Some(self.0.map(|_| 1).unwrap_or(0))
    }
}

#[derive(Debug, Default)]
pub struct U16be(Bytes<[u8; 2]>);
impl U16be {
    pub fn new(n: u16) -> Self {
        let mut b = [0; 2];
        BigEndian::write_u16(&mut b, n);
        U16be(Bytes::new(b))
    }
}
impl_codec!(U16be, u16, 2, BigEndian::read_u16, BigEndian::write_u16);

#[derive(Debug, Default)]
pub struct U16le(Bytes<[u8; 2]>);
impl U16le {
    pub fn new(n: u16) -> Self {
        let mut b = [0; 2];
        LittleEndian::write_u16(&mut b, n);
        U16le(Bytes::new(b))
    }
}
impl_codec!(
    U16le,
    u16,
    2,
    LittleEndian::read_u16,
    LittleEndian::write_u16
);

#[derive(Debug, Default)]
pub struct I16be(Bytes<[u8; 2]>);
impl I16be {
    pub fn new(n: i16) -> Self {
        let mut b = [0; 2];
        BigEndian::write_i16(&mut b, n);
        I16be(Bytes::new(b))
    }
}
impl_codec!(I16be, i16, 2, BigEndian::read_i16, BigEndian::write_i16);

#[derive(Debug, Default)]
pub struct I16le(Bytes<[u8; 2]>);
impl I16le {
    pub fn new(n: i16) -> Self {
        let mut b = [0; 2];
        LittleEndian::write_i16(&mut b, n);
        I16le(Bytes::new(b))
    }
}
impl_codec!(
    I16le,
    i16,
    2,
    LittleEndian::read_i16,
    LittleEndian::write_i16
);

#[derive(Debug, Default)]
pub struct U24be(Bytes<[u8; 3]>);
impl U24be {
    pub fn new(n: u32) -> Result<Self> {
        track_assert!(n <= 0xFF_FFFF, ErrorKind::InvalidInput);
        let mut b = [0; 3];
        BigEndian::write_u24(&mut b, n);
        Ok(U24be(Bytes::new(b)))
    }

    pub fn new_truncate(n: u32) -> Self {
        let mut b = [0; 3];
        BigEndian::write_u24(&mut b, n & 0xFF_FFFF);
        U24be(Bytes::new(b))
    }
}
impl_codec!(U24be, u32, 3, BigEndian::read_u24, BigEndian::write_u24);

#[derive(Debug, Default)]
pub struct U24le(Bytes<[u8; 3]>);
impl U24le {
    pub fn new(n: u32) -> Result<Self> {
        track_assert!(n <= 0xFF_FFFF, ErrorKind::InvalidInput);
        let mut b = [0; 3];
        LittleEndian::write_u24(&mut b, n);
        Ok(U24le(Bytes::new(b)))
    }

    pub fn new_truncate(n: u32) -> Self {
        let mut b = [0; 3];
        BigEndian::write_u24(&mut b, n & 0xFF_FFFF);
        U24le(Bytes::new(b))
    }
}
impl_codec!(
    U24le,
    u32,
    3,
    LittleEndian::read_u24,
    LittleEndian::write_u24
);

#[derive(Debug, Default)]
pub struct U32be(Bytes<[u8; 4]>);
impl U32be {
    pub fn new(n: u32) -> Self {
        let mut b = [0; 4];
        BigEndian::write_u32(&mut b, n);
        U32be(Bytes::new(b))
    }
}
impl_codec!(U32be, u32, 4, BigEndian::read_u32, BigEndian::write_u32);

#[derive(Debug, Default)]
pub struct U32le(Bytes<[u8; 4]>);
impl U32le {
    pub fn new(n: u32) -> Self {
        let mut b = [0; 4];
        LittleEndian::write_u32(&mut b, n);
        U32le(Bytes::new(b))
    }
}
impl_codec!(
    U32le,
    u32,
    4,
    LittleEndian::read_u32,
    LittleEndian::write_u32
);

#[derive(Debug, Default)]
pub struct I32be(Bytes<[u8; 4]>);
impl I32be {
    pub fn new(n: i32) -> Self {
        let mut b = [0; 4];
        BigEndian::write_i32(&mut b, n);
        I32be(Bytes::new(b))
    }
}
impl_codec!(I32be, i32, 4, BigEndian::read_i32, BigEndian::write_i32);

#[derive(Debug, Default)]
pub struct I32le(Bytes<[u8; 4]>);
impl I32le {
    pub fn new(n: i32) -> Self {
        let mut b = [0; 4];
        LittleEndian::write_i32(&mut b, n);
        I32le(Bytes::new(b))
    }
}
impl_codec!(
    I32le,
    i32,
    4,
    LittleEndian::read_i32,
    LittleEndian::write_i32
);

fn write_u40be(buf: &mut [u8], n: u64) {
    BigEndian::write_uint(buf, n, 5);
}

fn write_u40le(buf: &mut [u8], n: u64) {
    LittleEndian::write_uint(buf, n, 5);
}

fn read_u40be(buf: &[u8]) -> u64 {
    BigEndian::read_uint(buf, 5)
}

fn read_u40le(buf: &[u8]) -> u64 {
    LittleEndian::read_uint(buf, 5)
}

#[derive(Debug, Default)]
pub struct U40be(Bytes<[u8; 5]>);
impl U40be {
    pub fn new(n: u64) -> Result<Self> {
        track_assert!(n <= 0xFF_FFFF_FFFF, ErrorKind::InvalidInput);
        let mut b = [0; 5];
        write_u40be(&mut b, n);
        Ok(U40be(Bytes::new(b)))
    }

    pub fn new_truncate(n: u64) -> Self {
        let mut b = [0; 5];
        write_u40be(&mut b, n & 0xFF_FFFF_FFFF);
        U40be(Bytes::new(b))
    }
}
impl_codec!(U40be, u64, 5, read_u40be, write_u40be);

#[derive(Debug, Default)]
pub struct U40le(Bytes<[u8; 5]>);
impl U40le {
    pub fn new(n: u64) -> Result<Self> {
        track_assert!(n <= 0xFF_FFFF_FFFF, ErrorKind::InvalidInput);
        let mut b = [0; 5];
        write_u40le(&mut b, n);
        Ok(U40le(Bytes::new(b)))
    }

    pub fn new_truncate(n: u64) -> Self {
        let mut b = [0; 5];
        write_u40le(&mut b, n & 0xFF_FFFF_FFFF);
        U40le(Bytes::new(b))
    }
}
impl_codec!(U40le, u64, 5, read_u40le, write_u40le);

fn write_u48be(buf: &mut [u8], n: u64) {
    BigEndian::write_uint(buf, n, 6);
}

fn write_u48le(buf: &mut [u8], n: u64) {
    LittleEndian::write_uint(buf, n, 6);
}

fn read_u48be(buf: &[u8]) -> u64 {
    BigEndian::read_uint(buf, 6)
}

fn read_u48le(buf: &[u8]) -> u64 {
    LittleEndian::read_uint(buf, 6)
}

#[derive(Debug, Default)]
pub struct U48be(Bytes<[u8; 6]>);
impl U48be {
    pub fn new(n: u64) -> Result<Self> {
        track_assert!(n <= 0xFFFF_FFFF_FFFF, ErrorKind::InvalidInput);
        let mut b = [0; 6];
        write_u48be(&mut b, n);
        Ok(U48be(Bytes::new(b)))
    }

    pub fn new_truncate(n: u64) -> Self {
        let mut b = [0; 6];
        write_u48be(&mut b, n & 0xFFFF_FFFF_FFFF);
        U48be(Bytes::new(b))
    }
}
impl_codec!(U48be, u64, 6, read_u48be, write_u48be);

#[derive(Debug, Default)]
pub struct U48le(Bytes<[u8; 6]>);
impl U48le {
    pub fn new(n: u64) -> Result<Self> {
        track_assert!(n <= 0xFFFF_FFFF_FFFF, ErrorKind::InvalidInput);
        let mut b = [0; 6];
        write_u48le(&mut b, n);
        Ok(U48le(Bytes::new(b)))
    }

    pub fn new_truncate(n: u64) -> Self {
        let mut b = [0; 6];
        write_u48le(&mut b, n & 0xFFFF_FFFF_FFFF);
        U48le(Bytes::new(b))
    }
}
impl_codec!(U48le, u64, 6, read_u48le, write_u48le);

fn write_u56be(buf: &mut [u8], n: u64) {
    BigEndian::write_uint(buf, n, 7);
}

fn write_u56le(buf: &mut [u8], n: u64) {
    LittleEndian::write_uint(buf, n, 7);
}

fn read_u56be(buf: &[u8]) -> u64 {
    BigEndian::read_uint(buf, 7)
}

fn read_u56le(buf: &[u8]) -> u64 {
    LittleEndian::read_uint(buf, 7)
}

#[derive(Debug, Default)]
pub struct U56be(Bytes<[u8; 7]>);
impl U56be {
    pub fn new(n: u64) -> Result<Self> {
        track_assert!(n <= 0xFF_FFFF_FFFF_FFFF, ErrorKind::InvalidInput);
        let mut b = [0; 7];
        write_u56be(&mut b, n);
        Ok(U56be(Bytes::new(b)))
    }

    pub fn new_truncate(n: u64) -> Self {
        let mut b = [0; 7];
        write_u56be(&mut b, n & 0xFF_FFFF_FFFF_FFFF);
        U56be(Bytes::new(b))
    }
}
impl_codec!(U56be, u64, 7, read_u56be, write_u56be);

#[derive(Debug, Default)]
pub struct U56le(Bytes<[u8; 7]>);
impl U56le {
    pub fn new(n: u64) -> Result<Self> {
        track_assert!(n <= 0xFF_FFFF_FFFF_FFFF, ErrorKind::InvalidInput);
        let mut b = [0; 7];
        write_u56le(&mut b, n);
        Ok(U56le(Bytes::new(b)))
    }

    pub fn new_truncate(n: u64) -> Self {
        let mut b = [0; 7];
        write_u56le(&mut b, n & 0xFF_FFFF_FFFF_FFFF);
        U56le(Bytes::new(b))
    }
}
impl_codec!(U56le, u64, 7, read_u56le, write_u56le);

#[derive(Debug, Default)]
pub struct U64be(Bytes<[u8; 8]>);
impl U64be {
    pub fn new(n: u64) -> Self {
        let mut b = [0; 8];
        BigEndian::write_u64(&mut b, n);
        U64be(Bytes::new(b))
    }
}
impl_codec!(U64be, u64, 8, BigEndian::read_u64, BigEndian::write_u64);

#[derive(Debug, Default)]
pub struct U64le(Bytes<[u8; 8]>);
impl U64le {
    pub fn new(n: u64) -> Self {
        let mut b = [0; 8];
        LittleEndian::write_u64(&mut b, n);
        U64le(Bytes::new(b))
    }
}
impl_codec!(
    U64le,
    u64,
    8,
    LittleEndian::read_u64,
    LittleEndian::write_u64
);

#[derive(Debug, Default)]
pub struct I64be(Bytes<[u8; 8]>);
impl I64be {
    pub fn new(n: i64) -> Self {
        let mut b = [0; 8];
        BigEndian::write_i64(&mut b, n);
        I64be(Bytes::new(b))
    }
}
impl_codec!(I64be, i64, 8, BigEndian::read_i64, BigEndian::write_i64);

#[derive(Debug, Default)]
pub struct I64le(Bytes<[u8; 8]>);
impl I64le {
    pub fn new(n: i64) -> Self {
        let mut b = [0; 8];
        LittleEndian::write_i64(&mut b, n);
        I64le(Bytes::new(b))
    }
}
impl_codec!(
    I64le,
    i64,
    8,
    LittleEndian::read_i64,
    LittleEndian::write_i64
);

#[derive(Debug, Default)]
pub struct F32be(Bytes<[u8; 4]>);
impl F32be {
    pub fn new(n: f32) -> Self {
        let mut b = [0; 4];
        BigEndian::write_f32(&mut b, n);
        F32be(Bytes::new(b))
    }
}
impl_codec!(F32be, f32, 4, BigEndian::read_f32, BigEndian::write_f32);

#[derive(Debug, Default)]
pub struct F32le(Bytes<[u8; 4]>);
impl F32le {
    pub fn new(n: f32) -> Self {
        let mut b = [0; 4];
        LittleEndian::write_f32(&mut b, n);
        F32le(Bytes::new(b))
    }
}
impl_codec!(
    F32le,
    f32,
    4,
    LittleEndian::read_f32,
    LittleEndian::write_f32
);

#[derive(Debug, Default)]
pub struct F64be(Bytes<[u8; 8]>);
impl F64be {
    pub fn new(n: f64) -> Self {
        let mut b = [0; 8];
        BigEndian::write_f64(&mut b, n);
        F64be(Bytes::new(b))
    }
}
impl_codec!(F64be, f64, 8, BigEndian::read_f64, BigEndian::write_f64);

#[derive(Debug, Default)]
pub struct F64le(Bytes<[u8; 8]>);
impl F64le {
    pub fn new(n: f64) -> Self {
        let mut b = [0; 8];
        LittleEndian::write_f64(&mut b, n);
        F64le(Bytes::new(b))
    }
}
impl_codec!(
    F64le,
    f64,
    8,
    LittleEndian::read_f64,
    LittleEndian::write_f64
);
