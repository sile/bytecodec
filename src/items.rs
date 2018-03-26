//! Encodable/decodable items.
use byteorder::{BigEndian, ByteOrder, LittleEndian};

use {Decode, Encode, ErrorKind, Result};
use sequences::Bytes;

macro_rules! impl_codec {
    ($ty:ty, $item:ty, $size:expr, $endian:ident, $read:ident, $write:ident) => {
        impl Decode for $ty {
            type Item = $item;

            fn decode(&mut self, buf: &[u8], eos: bool) -> Result<usize> {
                track!(self.0.decode(buf, eos))
            }

            fn pop_item(&mut self) -> Result<Option<Self::Item>> {
                let item = track!(self.0.pop_item())?;
                Ok(item.map(|b| $endian::$read(&b)))
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
                $endian::$write(&mut b, item);
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
impl_codec!(U16be, u16, 2, BigEndian, read_u16, write_u16);

#[derive(Debug, Default)]
pub struct U16le(Bytes<[u8; 2]>);
impl U16le {
    pub fn new(n: u16) -> Self {
        let mut b = [0; 2];
        LittleEndian::write_u16(&mut b, n);
        U16le(Bytes::new(b))
    }
}
impl_codec!(U16le, u16, 2, LittleEndian, read_u16, write_u16);

#[derive(Debug, Default)]
pub struct I16be(Bytes<[u8; 2]>);
impl I16be {
    pub fn new(n: i16) -> Self {
        let mut b = [0; 2];
        BigEndian::write_i16(&mut b, n);
        I16be(Bytes::new(b))
    }
}
impl_codec!(I16be, i16, 2, BigEndian, read_i16, write_i16);

#[derive(Debug, Default)]
pub struct I16le(Bytes<[u8; 2]>);
impl I16le {
    pub fn new(n: i16) -> Self {
        let mut b = [0; 2];
        LittleEndian::write_i16(&mut b, n);
        I16le(Bytes::new(b))
    }
}
impl_codec!(I16le, i16, 2, LittleEndian, read_i16, write_i16);

#[derive(Debug, Default)]
pub struct U24be(Bytes<[u8; 3]>);
impl U24be {
    pub fn new(n: u32) -> Result<Self> {
        track_assert!(n <= 0xFF_FFFF, ErrorKind::InvalidInput);
        let mut b = [0; 3];
        BigEndian::write_u24(&mut b, n);
        Ok(U24be(Bytes::new(b)))
    }
}
impl_codec!(U24be, u32, 3, BigEndian, read_u24, write_u24);

#[derive(Debug, Default)]
pub struct U24le(Bytes<[u8; 3]>);
impl U24le {
    pub fn new(n: u32) -> Result<Self> {
        track_assert!(n <= 0xFF_FFFF, ErrorKind::InvalidInput);
        let mut b = [0; 3];
        LittleEndian::write_u24(&mut b, n);
        Ok(U24le(Bytes::new(b)))
    }
}
impl_codec!(U24le, u32, 3, LittleEndian, read_u24, write_u24);

#[derive(Debug, Default)]
pub struct I24be(Bytes<[u8; 3]>);
impl I24be {
    pub fn new(n: i32) -> Result<Self> {
        track_assert!((n as u32) <= 0xFF_FFFF, ErrorKind::InvalidInput);
        let mut b = [0; 3];
        BigEndian::write_i24(&mut b, n);
        Ok(I24be(Bytes::new(b)))
    }
}
impl_codec!(I24be, i32, 3, BigEndian, read_i24, write_i24);

#[derive(Debug, Default)]
pub struct I24le(Bytes<[u8; 3]>);
impl I24le {
    pub fn new(n: i32) -> Result<Self> {
        track_assert!((n as u32) <= 0xFF_FFFF, ErrorKind::InvalidInput);
        let mut b = [0; 3];
        LittleEndian::write_i24(&mut b, n);
        Ok(I24le(Bytes::new(b)))
    }
}
impl_codec!(I24le, i32, 3, LittleEndian, read_i24, write_i24);

#[derive(Debug, Default)]
pub struct U32be(Bytes<[u8; 4]>);
impl U32be {
    pub fn new(n: u32) -> Self {
        let mut b = [0; 4];
        BigEndian::write_u32(&mut b, n);
        U32be(Bytes::new(b))
    }
}
impl_codec!(U32be, u32, 4, BigEndian, read_u32, write_u32);

#[derive(Debug, Default)]
pub struct U32le(Bytes<[u8; 4]>);
impl U32le {
    pub fn new(n: u32) -> Self {
        let mut b = [0; 4];
        LittleEndian::write_u32(&mut b, n);
        U32le(Bytes::new(b))
    }
}
impl_codec!(U32le, u32, 4, LittleEndian, read_u32, write_u32);

#[derive(Debug, Default)]
pub struct I32be(Bytes<[u8; 4]>);
impl I32be {
    pub fn new(n: i32) -> Self {
        let mut b = [0; 4];
        BigEndian::write_i32(&mut b, n);
        I32be(Bytes::new(b))
    }
}
impl_codec!(I32be, i32, 4, BigEndian, read_i32, write_i32);

#[derive(Debug, Default)]
pub struct I32le(Bytes<[u8; 4]>);
impl I32le {
    pub fn new(n: i32) -> Self {
        let mut b = [0; 4];
        LittleEndian::write_i32(&mut b, n);
        I32le(Bytes::new(b))
    }
}
impl_codec!(I32le, i32, 4, LittleEndian, read_i32, write_i32);
