use std::cmp;

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
