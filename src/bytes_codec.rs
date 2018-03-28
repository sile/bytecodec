use std::io::{Read, Write};
use std::mem;
use trackable::error::ErrorKindExt;

use {Decode, DecodeBuf, Encode, EncodeBuf, Error, ErrorKind, Result};

#[derive(Debug)]
pub struct BytesEncoder<B> {
    bytes: Option<B>,
    offset: usize,
}
impl<B> BytesEncoder<B> {
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

    fn encode(&mut self, buf: &mut EncodeBuf) -> Result<()> {
        if let Some(ref mut b) = self.bytes {
            let size = track!(buf.write(&b.as_ref()[self.offset..]).map_err(Error::from))?;
            self.offset += size;
        }
        Ok(())
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track_assert_eq!(self.remaining_bytes(), Some(0), ErrorKind::Full);
        self.bytes = Some(item);
        self.offset = 0;
        Ok(())
    }

    fn remaining_bytes(&self) -> Option<u64> {
        Some(
            self.bytes
                .as_ref()
                .map_or(0, |b| (b.as_ref().len() - self.offset) as u64),
        )
    }
}

#[derive(Debug, Default)]
pub struct BytesDecoder<B> {
    bytes: B,
    offset: usize,
}
impl<B: Default> BytesDecoder<B> {
    pub fn new() -> Self {
        Self::default()
    }
}
impl<B: AsMut<[u8]> + Default> Decode for BytesDecoder<B> {
    type Item = B;

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        let size = track!(
            buf.read(&mut self.bytes.as_mut()[self.offset..])
                .map_err(Error::from)
        )?;
        self.offset += size;

        if self.offset == self.bytes.as_mut().len() {
            let bytes = mem::replace(&mut self.bytes, B::default());
            Ok(Some(bytes))
        } else {
            track_assert!(!buf.is_eos(), ErrorKind::InvalidInput);
            Ok(None)
        }
    }
}

pub type VecEncoder = BytesEncoder<Vec<u8>>;

#[derive(Debug, Default)]
pub struct VecDecoder(Vec<u8>);
impl VecDecoder {
    pub fn new() -> Self {
        Self::default()
    }
}
impl Decode for VecDecoder {
    type Item = Vec<u8>;

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        if let Some(additional) = buf.remaining_bytes() {
            self.0.reserve_exact(buf.len() + additional as usize);
        }

        track!(buf.read_to_end(&mut self.0).map_err(Error::from))?;
        if buf.is_eos() {
            Ok(Some(mem::replace(&mut self.0, Vec::new())))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Default)]
pub struct Utf8Encoder(VecEncoder);
impl Utf8Encoder {
    pub fn new() -> Self {
        Self::default()
    }
}
impl Encode for Utf8Encoder {
    type Item = String;

    fn encode(&mut self, buf: &mut EncodeBuf) -> Result<()> {
        track!(self.0.encode(buf))
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track!(self.0.start_encoding(item.into_bytes()))
    }

    fn remaining_bytes(&self) -> Option<u64> {
        self.0.remaining_bytes()
    }
}

#[derive(Debug, Default)]
pub struct Utf8Decoder<D>(D);
impl<D> Utf8Decoder<D>
where
    D: Decode<Item = Vec<u8>>,
{
    pub fn new(bytes_decoder: D) -> Self {
        Utf8Decoder(bytes_decoder)
    }
}
impl<D> Decode for Utf8Decoder<D>
where
    D: Decode<Item = Vec<u8>>,
{
    type Item = String;

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        if let Some(bytes) = track!(self.0.decode(buf))? {
            let s = track!(String::from_utf8(bytes).map_err(|e| ErrorKind::InvalidInput.cause(e)))?;
            Ok(Some(s))
        } else {
            Ok(None)
        }
    }
}
