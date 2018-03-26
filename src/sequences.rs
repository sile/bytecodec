use std::cmp;
use std::iter;
use trackable::error::ErrorKindExt;

use {Decode, Encode, ErrorKind, Result};

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

    fn decode(&mut self, buf: &[u8], eos: bool) -> Result<usize> {
        track!(self.0.decode(buf, eos))
    }

    fn pop_item(&mut self) -> Result<Option<Self::Item>> {
        let item = track!(self.0.pop_item())?;
        if let Some(bytes) = item {
            let s = track!(String::from_utf8(bytes).map_err(|e| ErrorKind::InvalidInput.cause(e)))?;
            Ok(Some(s))
        } else {
            Ok(None)
        }
    }

    fn decode_size_hint(&self) -> Option<usize> {
        Some(self.0.remaining_size())
    }
}
impl Encode for Utf8 {
    type Item = String;

    fn encode(&mut self, buf: &mut [u8]) -> Result<usize> {
        track!(self.0.encode(buf))
    }

    fn push_item(&mut self, item: Self::Item) -> Result<Option<Self::Item>> {
        track!(self.0.push_item(item.into_bytes()))
            .map(|t| t.map(|bytes| unsafe { String::from_utf8_unchecked(bytes) }))
    }

    fn encode_size_hint(&self) -> Option<usize> {
        Some(self.0.remaining_size())
    }
}

#[derive(Debug)]
pub struct FromIter<T, D> {
    decoder: D,
    items: Option<T>,
    eos: bool,
}
impl<T: Default, D> FromIter<T, D> {
    pub fn new(decoder: D) -> Self {
        FromIter {
            decoder,
            items: Some(T::default()),
            eos: false,
        }
    }
}
impl<T, D> Decode for FromIter<T, D>
where
    T: Extend<D::Item>,
    D: Decode,
{
    type Item = T;

    fn decode(&mut self, buf: &[u8], eos: bool) -> Result<usize> {
        self.eos = eos;
        track!(self.decoder.decode(buf, eos))
    }

    fn pop_item(&mut self) -> Result<Option<Self::Item>> {
        if let Some(items) = self.items.as_mut() {
            while let Some(item) = track!(self.decoder.pop_item())? {
                items.extend(iter::once(item));
            }
        }
        if self.eos {
            Ok(self.items.take())
        } else {
            Ok(None)
        }
    }

    fn decode_size_hint(&self) -> Option<usize> {
        self.decoder.decode_size_hint()
    }
}

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
impl<T> Encode for Iter<T>
where
    T: Iterator,
    T::Item: Encode,
{
    type Item = T;

    fn encode(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut offset = 0;
        while offset < buf.len() && self.current.is_some() {
            let buf = &mut buf[offset..];

            let mut x = self.current.take().expect("Never fails");
            let size = track!(x.encode(buf))?;
            offset += size;
            if size == buf.len() {
                self.current = Some(x);
            } else {
                self.current = self.iter.next();
            }
        }
        Ok(offset)
    }

    fn push_item(&mut self, mut item: Self::Item) -> Result<Option<Self::Item>> {
        if self.current.is_none() {
            self.current = item.next();
            self.iter = item;
            Ok(None)
        } else {
            Ok(Some(item))
        }
    }

    fn encode_size_hint(&self) -> Option<usize> {
        self.current.as_ref().and_then(|x| x.encode_size_hint())
    }
}
