use std::cmp;
use std::io::{self, Read};
use std::ops::Deref;

use {Error, ErrorKind, Result};
use combinators::{AndThen, Map, MapErr};

// TODO: move
#[derive(Debug)]
pub struct DecodeBuf<'a> {
    buf: &'a [u8],
    offset: usize,
    eos: bool,
}
impl<'a> DecodeBuf<'a> {
    pub fn new(buf: &'a [u8], eos: bool) -> Self {
        DecodeBuf {
            buf,
            eos,
            offset: 0,
        }
    }

    pub fn is_eos(&self) -> bool {
        self.eos
    }

    pub fn consume(&mut self, size: usize) -> Result<()> {
        track_assert!(self.offset + size <= self.len(), ErrorKind::InvalidInput;
                      self.offset, size, self.len());
        self.offset += size;
        Ok(())
    }
}
impl<'a> AsRef<[u8]> for DecodeBuf<'a> {
    fn as_ref(&self) -> &[u8] {
        &self.buf[self.offset..]
    }
}
impl<'a> Deref for DecodeBuf<'a> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
impl<'a> Read for DecodeBuf<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let size = cmp::min(self.len(), buf.len());
        (&mut buf[..size]).copy_from_slice(&self[..size]);
        self.offset += size;
        Ok(size)
    }
}

pub trait Decode {
    type Item;

    // NOTE: 一バイトも消費されない場合には、もうデコード可能なitemが存在しないことを意味する
    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>>;
    fn decode_size_hint(&self) -> Option<usize> {
        None
    }
}

pub trait MakeDecoder {
    type Decoder: Decode;

    fn make_decoder(&self) -> Self::Decoder;
}

pub trait Encode {
    type Item;

    // NOTE: `結果サイズ < buf.len()`は、エンコード完了を意味する
    fn encode(&mut self, buf: &mut [u8]) -> Result<usize>;

    // Resultは不要?
    fn push_item(&mut self, item: Self::Item) -> Result<Option<Self::Item>>;

    // TODO: Iteratorに合わせる?
    fn encode_size_hint(&self) -> Option<usize> {
        None
    }
}

pub trait DecodeExt: Decode + Sized {
    fn map<T, F>(self, f: F) -> Map<Self, T, F>
    where
        F: Fn(Self::Item) -> T,
    {
        Map::new(self, f)
    }

    fn and_then<T, F>(self, f: F) -> AndThen<Self, T, F>
    where
        F: Fn(Self::Item) -> T,
        T: Decode,
    {
        AndThen::new(self, f)
    }

    fn map_err<F>(self, f: F) -> MapErr<Self, F>
    where
        F: Fn(Error) -> Error,
    {
        MapErr::new(self, f)
    }
}

pub trait EncodeExt: Encode + Sized {
    fn map_err<F>(self, f: F) -> MapErr<Self, F>
    where
        F: Fn(Error) -> Error,
    {
        MapErr::new(self, f)
    }
}

pub trait MakeEncoder {
    type Encoder: Encode;

    fn make_encoder(&self) -> Self::Encoder;
}
