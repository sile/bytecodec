use std;
use std::cmp;
use std::fmt;
use std::io::{self, Read};
use std::ops::Deref;

use {Error, ErrorKind, Result};
use combinator::{AndThen, Collect, DecoderChain, IgnoreRest, Map, MapErr, Take, Validate};

pub trait Decode {
    type Item;

    // NOTE: 一バイトも消費されない場合には、もうデコード可能なitemが存在しないことを意味する
    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>>;

    // Returns the number of bytes needed to proceed next state.
    fn requiring_bytes(&self) -> Option<u64> {
        None
    }
}
impl Decode for () {
    type Item = ();

    fn decode(&mut self, _buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        Ok(Some(()))
    }
}
impl<D: Decode> Decode for Option<D> {
    type Item = Option<D::Item>;

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        if let Some(ref mut d) = *self {
            Ok(track!(d.decode(buf))?.map(Some))
        } else {
            Ok(None)
        }
    }
}

// TODO: ExactSizeDecoder

pub trait DecodeExt: Decode + Sized {
    fn map<T, F>(self, f: F) -> Map<Self, T, F>
    where
        F: Fn(Self::Item) -> T,
    {
        Map::new(self, f)
    }

    fn map_err<F>(self, f: F) -> MapErr<Self, F>
    where
        F: Fn(Error) -> Error,
    {
        MapErr::new(self, f)
    }

    fn and_then<D, F>(self, f: F) -> AndThen<Self, D, F>
    where
        F: Fn(Self::Item) -> D,
        D: Decode,
    {
        AndThen::new(self, f)
    }

    fn chain<D: Decode>(self, other: D) -> DecoderChain<Self, D> {
        DecoderChain::new(self, other)
    }

    fn collect<T>(self) -> Collect<Self, T>
    where
        T: Extend<Self::Item> + Default,
    {
        Collect::new(self)
    }

    fn take(self, size: u64) -> Take<Self> {
        Take::new(self, size)
    }

    fn present(self, b: bool) -> Option<Self> {
        if b {
            Some(self)
        } else {
            None
        }
    }

    fn ignore_rest(self) -> IgnoreRest<Self> {
        IgnoreRest::new(self)
    }

    fn validate<F, E>(self, f: F) -> Validate<Self, F, E>
    where
        F: for<'a> Fn(&'a Self::Item) -> std::result::Result<(), E>,
        Error: From<E>,
    {
        Validate::new(self, f)
    }

    fn boxed(self) -> BoxDecoder<Self::Item>
    where
        Self: Send + 'static,
    {
        BoxDecoder(Box::new(self))
    }
}
impl<T: Decode> DecodeExt for T {}

pub struct BoxDecoder<T>(Box<Decode<Item = T> + Send + 'static>);
impl<T> fmt::Debug for BoxDecoder<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BoxDecoder(_)")
    }
}
impl<T> Decode for BoxDecoder<T> {
    type Item = T;

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        self.0.decode(buf)
    }
}

#[derive(Debug)]
pub struct DecodeBuf<'a> {
    buf: &'a [u8],
    offset: usize,
    remaining_bytes: Option<u64>,
}
impl DecodeBuf<'static> {
    pub fn eos() -> Self {
        DecodeBuf {
            buf: &[],
            offset: 0,
            remaining_bytes: Some(0),
        }
    }
}
impl<'a> DecodeBuf<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        DecodeBuf {
            buf,
            offset: 0,
            remaining_bytes: None,
        }
    }

    pub fn with_remaining_bytes(buf: &'a [u8], remaining_bytes: u64) -> Self {
        DecodeBuf {
            buf,
            offset: 0,
            remaining_bytes: Some(remaining_bytes),
        }
    }

    // buf.len()に加えて、次のitemをデコードするのに必要なバイト数.
    // 不明な場合には`None`
    pub fn remaining_bytes(&self) -> Option<u64> {
        self.remaining_bytes
    }

    pub fn is_eos(&self) -> bool {
        self.remaining_bytes().map_or(false, |n| n == 0)
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
