use std::io::{Read, Write};
use trackable::error::ErrorKindExt;

use {Decode, DecodeBuf, Encode, EncodeBuf, Error, ErrorKind, Result};

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
impl<B: Default> Default for Bytes<B> {
    fn default() -> Self {
        Bytes {
            bytes: Some(B::default()),
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

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        let decoded = if let Some(ref mut b) = self.bytes {
            let size = track!(
                buf.read(&mut b.as_mut()[self.offset..])
                    .map_err(Error::from)
            )?;
            self.offset += size;
            if buf.is_eos() {
                track_assert_eq!(self.offset, b.as_ref().len(), ErrorKind::InvalidInput);
            }
            self.offset == b.as_ref().len()
        } else {
            false
        };
        if decoded {
            Ok(self.bytes.take())
        } else {
            Ok(None)
        }
    }

    fn decode_size_hint(&self) -> usize {
        self.remaining_size()
    }
}
impl<B: AsRef<[u8]>> Encode for Bytes<B> {
    type Item = B;

    fn encode(&mut self, buf: &mut EncodeBuf) -> Result<()> {
        if let Some(ref mut b) = self.bytes {
            let size = track!(buf.write(&b.as_ref()[self.offset..]).map_err(Error::from))?;
            self.offset += size;
        }
        Ok(())
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<Option<Self::Item>> {
        if self.remaining_size() == 0 {
            self.bytes = Some(item);
            self.offset = 0;
            Ok(None)
        } else {
            Ok(Some(item))
        }
    }

    fn encode_size_hint(&self) -> usize {
        self.remaining_size()
    }
}

#[derive(Debug)]
pub struct Utf8(Bytes<Vec<u8>>);
impl Utf8 {
    pub fn new(s: String) -> Self {
        Utf8(Bytes::new(s.into_bytes()))
    }

    pub fn zeroes(size: usize) -> Self {
        let s = unsafe { String::from_utf8_unchecked(vec![0; size]) };
        Self::new(s)
    }
}
impl Decode for Utf8 {
    type Item = String;

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        let item = track!(self.0.decode(buf))?;
        if let Some(bytes) = item {
            let s = track!(String::from_utf8(bytes).map_err(|e| ErrorKind::InvalidInput.cause(e)))?;
            Ok(Some(s))
        } else {
            Ok(None)
        }
    }

    fn decode_size_hint(&self) -> usize {
        self.0.remaining_size()
    }
}
impl Encode for Utf8 {
    type Item = String;

    fn encode(&mut self, buf: &mut EncodeBuf) -> Result<()> {
        track!(self.0.encode(buf))
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<Option<Self::Item>> {
        track!(self.0.start_encoding(item.into_bytes()))
            .map(|t| t.map(|bytes| unsafe { String::from_utf8_unchecked(bytes) }))
    }

    fn encode_size_hint(&self) -> usize {
        self.0.remaining_size()
    }
}

// #[derive(Debug)]
// pub struct FromIter<T, D> {
//     decoder: D,
//     items: Option<T>,
//     eos: bool,
// }
// impl<T: Default, D> FromIter<T, D> {
//     pub fn new(decoder: D) -> Self {
//         FromIter {
//             decoder,
//             items: Some(T::default()),
//             eos: false,
//         }
//     }
// }
// impl<T, D> Decode for FromIter<T, D>
// where
//     T: Extend<D::Item>,
//     D: Decode,
// {
//     type Item = T;

//     fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
//         if let Some(items) = self.items.as_mut() {
//             while !buf.is_empty() {
//                 if let Some(item) = track!(self.decoder.decode(buf))? {
//                     items.extend(iter::once(item));
//                 }
//             }
//         }
//         if buf.is_eos() {
//             Ok(self.items.take())
//         } else {
//             Ok(None)
//         }
//     }

//     fn decode_size_hint(&self) -> Option<usize> {
//         self.decoder.decode_size_hint()
//     }
// }

#[derive(Debug)]
pub struct Iter<T: Iterator> {
    iter: T,
    current: Option<T::Item>,
}
impl<T: Iterator> Iter<T> {
    pub fn new(mut iter: T) -> Self {
        let current = iter.next();
        Iter { iter, current }
    }
}
// TODO: impl Decode
impl<T> Encode for Iter<T>
where
    T: Iterator,
    T::Item: Encode,
{
    type Item = T;

    fn encode(&mut self, buf: &mut EncodeBuf) -> Result<()> {
        while !buf.is_empty() && self.current.is_some() {
            let mut x = self.current.take().expect("Never fails");
            track!(x.encode(buf))?;
            if buf.is_completed() {
                self.current = self.iter.next();
            } else {
                self.current = Some(x);
            }
        }
        Ok(())
    }

    fn start_encoding(&mut self, mut item: Self::Item) -> Result<Option<Self::Item>> {
        if self.current.is_none() {
            self.current = item.next();
            self.iter = item;
            Ok(None)
        } else {
            Ok(Some(item))
        }
    }

    fn encode_size_hint(&self) -> usize {
        self.current.as_ref().map_or(0, |x| x.encode_size_hint())
    }
}
