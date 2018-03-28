use std::cmp;
use std::fmt;
use std::io::{self, Write};
use std::ops::{Deref, DerefMut};

use {Error, ErrorKind, Result};
use combinator::{EncoderChain, MapErr, Optional, Repeat, StartEncodingFrom};

pub trait Encode {
    type Item;

    // NOTE: 一バイトも書き込まれない場合には、エンコード終了を意味する
    fn encode(&mut self, buf: &mut EncodeBuf) -> Result<()>;

    fn start_encoding(&mut self, item: Self::Item) -> Result<()>;

    fn remaining_bytes(&self) -> Option<u64>;
}

pub trait EncodeExt: Encode + Sized {
    fn map_err<F>(self, f: F) -> MapErr<Self, F>
    where
        F: Fn(Error) -> Error,
    {
        MapErr::new(self, f)
    }

    fn start_encoding_from<T, F>(self, f: F) -> StartEncodingFrom<Self, T, F>
    where
        F: Fn(T) -> Self::Item,
    {
        StartEncodingFrom::new(self, f)
    }

    fn chain<E: Encode>(self, other: E) -> EncoderChain<Self, E, Self::Item> {
        EncoderChain::new(self, other)
    }

    fn repeat<I>(self) -> Repeat<Self, I>
    where
        I: Iterator<Item = Self::Item>,
    {
        Repeat::new(self)
    }

    fn optional(self) -> Optional<Self> {
        Optional::new(self)
    }

    fn boxed(self) -> BoxEncoder<Self::Item>
    where
        Self: Send + 'static,
    {
        BoxEncoder(Box::new(self))
    }
}
impl<T: Encode> EncodeExt for T {}

pub struct BoxEncoder<T>(Box<Encode<Item = T> + Send + 'static>);
impl<T> fmt::Debug for BoxEncoder<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BoxEncoder(_)")
    }
}
impl<T> Encode for BoxEncoder<T> {
    type Item = T;

    fn encode(&mut self, buf: &mut EncodeBuf) -> Result<()> {
        self.0.encode(buf)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        self.0.start_encoding(item)
    }

    fn remaining_bytes(&self) -> Option<u64> {
        self.0.remaining_bytes()
    }
}

#[derive(Debug)]
pub struct EncodeBuf<'a> {
    buf: &'a mut [u8],
    offset: usize,
}
impl<'a> EncodeBuf<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        EncodeBuf { buf, offset: 0 }
    }

    pub fn consume(&mut self, size: usize) -> Result<()> {
        track_assert!(self.offset + size <= self.len(), ErrorKind::InvalidInput;
                      self.offset, size, self.len());
        self.offset += size;
        Ok(())
    }
}
impl<'a> AsRef<[u8]> for EncodeBuf<'a> {
    fn as_ref(&self) -> &[u8] {
        &self.buf[self.offset..]
    }
}
impl<'a> AsMut<[u8]> for EncodeBuf<'a> {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.buf[self.offset..]
    }
}
impl<'a> Deref for EncodeBuf<'a> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
impl<'a> DerefMut for EncodeBuf<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}
impl<'a> Write for EncodeBuf<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let size = cmp::min(self.len(), buf.len());
        (&mut self.as_mut()[..size]).copy_from_slice(&buf[..size]);
        self.offset += size;
        Ok(size)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
