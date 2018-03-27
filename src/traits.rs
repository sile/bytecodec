use std::cmp;
use std::io::{self, Read, Write};
use std::ops::{Deref, DerefMut};

use {Error, ErrorKind, Result};
use combinators::{AndThen, Buffered, Chain, Chain2, Flatten, Map, MapErr, MapFrom};

#[derive(Debug)]
pub struct EncodeBuf<'a> {
    buf: &'a mut [u8],
    offset: usize,
    completed: bool,
}
impl<'a> EncodeBuf<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        EncodeBuf {
            buf,
            offset: 0,
            completed: true,
        }
    }

    pub fn consume(&mut self, size: usize) -> Result<()> {
        track_assert!(self.offset + size <= self.len(), ErrorKind::InvalidInput;
                      self.offset, size, self.len());
        self.offset += size;
        self.completed = size == 0;
        Ok(())
    }

    // TODO: rename
    pub fn is_completed(&self) -> bool {
        self.completed
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
        self.completed = size == 0;
        Ok(size)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

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

    // 下限を返す
    fn decode_size_hint(&self) -> usize {
        0
    }
}

pub trait MakeDecoder {
    type Decoder: Decode;

    fn make_decoder(&self) -> Self::Decoder;
}

pub trait Encode {
    type Item;

    // NOTE: 一バイトも書き込まれない場合には、エンコード終了を意味する
    fn encode(&mut self, buf: &mut EncodeBuf) -> Result<()>;

    fn start_encoding(&mut self, item: Self::Item) -> Result<()>;

    // 下限を返す
    // TODO: encoding_size_hint(?)
    fn encode_size_hint(&self) -> usize {
        0
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

    fn chain<T: Decode>(self, other: T) -> Chain<Buffered<Self>, Buffered<T>> {
        Chain::new(Buffered::new(self), Buffered::new(other))
    }

    fn flatten(self) -> Flatten<Self, Self::Item> {
        Flatten::new(self)
    }

    fn boxed(self) -> BoxDecoder<Self::Item>
    where
        Self: Send + 'static,
    {
        BoxDecoder(Box::new(self))
    }
}

pub trait EncodeExt: Encode + Sized {
    fn map_err<F>(self, f: F) -> MapErr<Self, F>
    where
        F: Fn(Error) -> Error,
    {
        MapErr::new(self, f)
    }

    fn chain<T: Encode>(self, other: T) -> Chain2<Self, T, Self::Item> {
        Chain2::new(self, other)
    }

    fn map_from<T, F>(self, f: F) -> MapFrom<Self, T, F>
    where
        F: Fn(T) -> Self::Item,
    {
        MapFrom::new(self, f)
    }

    fn boxed(self) -> BoxEncoder<Self::Item>
    where
        Self: Send + 'static,
    {
        BoxEncoder(Box::new(self))
    }
}
impl<T: Encode> EncodeExt for T {}

impl<T: Decode> DecodeExt for T {}

pub struct BoxEncoder<T>(Box<Encode<Item = T> + Send + 'static>);
impl<T> ::std::fmt::Debug for BoxEncoder<T> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
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

    fn encode_size_hint(&self) -> usize {
        self.0.encode_size_hint()
    }
}

pub struct BoxDecoder<T>(Box<Decode<Item = T> + Send + 'static>);
impl<T> ::std::fmt::Debug for BoxDecoder<T> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "BoxDecoder(_)")
    }
}
impl<T> Decode for BoxDecoder<T> {
    type Item = T;

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        self.0.decode(buf)
    }

    fn decode_size_hint(&self) -> usize {
        self.0.decode_size_hint()
    }
}

pub trait MakeEncoder {
    type Encoder: Encode;

    fn make_encoder(&self) -> Self::Encoder;
}

impl Encode for () {
    type Item = ();

    fn encode(&mut self, _buf: &mut EncodeBuf) -> Result<()> {
        Ok(())
    }

    fn start_encoding(&mut self, _item: Self::Item) -> Result<()> {
        Ok(())
    }
}
impl Decode for () {
    type Item = ();

    fn decode(&mut self, _buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        Ok(Some(()))
    }
}
