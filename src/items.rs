//! Encodable/decodable items.
use std::cmp;
use byteorder::{BigEndian, ByteOrder};

use {Decode, Encode, ErrorKind, Result};

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

#[derive(Debug)]
pub struct U16be(Bytes<[u8; 2]>);
impl U16be {
    pub fn new(n: u16) -> Self {
        let mut b = [0; 2];
        BigEndian::write_u16(&mut b, n);
        U16be(Bytes::new(b))
    }
}
impl Default for U16be {
    fn default() -> Self {
        U16be(Bytes::new([0; 2]))
    }
}
impl Decode for U16be {
    type Item = u16;

    fn decode(&mut self, buf: &[u8], eos: bool) -> Result<usize> {
        track!(self.0.decode(buf, eos))
    }

    fn pop_item(&mut self) -> Result<Option<Self::Item>> {
        let item = track!(self.0.pop_item())?;
        Ok(item.map(|b| BigEndian::read_u16(&b)))
    }

    fn decode_size_hint(&self) -> Option<usize> {
        self.0.decode_size_hint()
    }
}
impl Encode for U16be {
    type Item = u16;

    fn encode(&mut self, buf: &mut [u8]) -> Result<usize> {
        track!(self.0.encode(buf))
    }

    fn push_item(&mut self, item: Self::Item) -> Result<Option<Self::Item>> {
        let mut b = [0; 2];
        BigEndian::write_u16(&mut b, item);
        track!(self.0.push_item(b).map(|r| r.map(|_| item)))
    }

    fn encode_size_hint(&self) -> Option<usize> {
        self.0.encode_size_hint()
    }
}

#[derive(Debug)]
pub struct Bytes<B> {
    bytes: Option<B>,
    offset: usize,
}
impl<B> Bytes<B> {
    pub fn new(bytes: B) -> Self {
        Bytes {
            bytes: Some(bytes),
            offset: 0,
        }
    }
}
impl<B: AsRef<[u8]>> Bytes<B> {
    fn remaining_size(&self) -> usize {
        self.bytes
            .as_ref()
            .map_or(0, |b| b.as_ref().len() - self.offset)
    }
}
impl<B: AsRef<[u8]> + AsMut<[u8]>> Decode for Bytes<B> {
    type Item = B;

    fn decode(&mut self, buf: &[u8], eos: bool) -> Result<usize> {
        if let Some(ref mut b) = self.bytes {
            let size = cmp::min(buf.len(), b.as_ref().len() - self.offset);
            (&mut b.as_mut()[self.offset..][..size]).copy_from_slice(&buf[..size]);
            self.offset += size;
            if eos {
                track_assert_eq!(self.offset, b.as_ref().len(), ErrorKind::InvalidInput);
            }
            Ok(size)
        } else {
            Ok(0)
        }
    }

    fn pop_item(&mut self) -> Result<Option<Self::Item>> {
        if self.bytes
            .as_ref()
            .map_or(false, |b| b.as_ref().len() == self.offset)
        {
            Ok(self.bytes.take())
        } else {
            Ok(None)
        }
    }

    fn decode_size_hint(&self) -> Option<usize> {
        Some(self.remaining_size())
    }
}
impl<B: AsRef<[u8]>> Encode for Bytes<B> {
    type Item = B;

    fn encode(&mut self, buf: &mut [u8]) -> Result<usize> {
        if let Some(ref mut b) = self.bytes {
            let size = cmp::min(buf.len(), b.as_ref().len() - self.offset);
            (&mut buf[..size]).copy_from_slice(&b.as_ref()[self.offset..][..size]);
            self.offset += size;
            Ok(size)
        } else {
            Ok(0)
        }
    }

    fn push_item(&mut self, item: Self::Item) -> Result<Option<Self::Item>> {
        if self.remaining_size() == 0 {
            self.bytes = Some(item);
            self.offset = 0;
            Ok(None)
        } else {
            Ok(Some(item))
        }
    }

    fn encode_size_hint(&self) -> Option<usize> {
        Some(self.remaining_size())
    }
}

// #[derive(Debug)]
// pub struct U16be(u8);
// impl U8 {
//     pub fn new(v: u8) -> Self {
//         U8(v)
//     }
// }
// impl Encode for U8 {
//     type Item = u8;

//     fn encode(&mut self, buf: &mut [u8]) -> Result<usize> {
//         track_assert_ne!(buf.len(), 0, ErrorKind::InvalidInput);
//         buf[0] = self.0;
//         Ok(1)
//     }
// }
