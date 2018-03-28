use std;
use std::cmp;
use std::iter;
use std::marker::PhantomData;
use std::mem;

pub use chain::{DecoderChain, EncoderChain};

use {Decode, DecodeBuf, Encode, EncodeBuf, Error, ErrorKind, Result};

#[derive(Debug)]
pub struct Map<D, T, F> {
    decoder: D,
    map: F,
    _item: PhantomData<T>,
}
impl<D: Decode, T, F> Map<D, T, F> {
    pub(crate) fn new(decoder: D, map: F) -> Self
    where
        F: Fn(D::Item) -> T,
    {
        Map {
            decoder,
            map,
            _item: PhantomData,
        }
    }
}
impl<D, T, F> Decode for Map<D, T, F>
where
    D: Decode,
    F: Fn(D::Item) -> T,
{
    type Item = T;

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        self.decoder.decode(buf).map(|r| r.map(&self.map))
    }
}

#[derive(Debug)]
pub struct MapErr<C, F> {
    codec: C,
    map_err: F,
}
impl<C, F> MapErr<C, F> {
    pub(crate) fn new(codec: C, map_err: F) -> Self
    where
        F: Fn(Error) -> Error,
    {
        MapErr { codec, map_err }
    }
}
impl<D, F> Decode for MapErr<D, F>
where
    D: Decode,
    F: Fn(Error) -> Error,
{
    type Item = D::Item;

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        self.codec.decode(buf).map_err(&self.map_err)
    }
}
impl<E, F> Encode for MapErr<E, F>
where
    E: Encode,
    F: Fn(Error) -> Error,
{
    type Item = E::Item;

    fn encode(&mut self, buf: &mut EncodeBuf) -> Result<()> {
        self.codec.encode(buf).map_err(&self.map_err)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        self.codec.start_encoding(item).map_err(&self.map_err)
    }

    fn remaining_bytes(&self) -> Option<u64> {
        self.codec.remaining_bytes()
    }
}

#[derive(Debug)]
pub struct AndThen<D0, D1, F> {
    decoder0: D0,
    decoder1: Option<D1>,
    and_then: F,
}
impl<D0: Decode, D1, F> AndThen<D0, D1, F> {
    pub(crate) fn new(decoder0: D0, and_then: F) -> Self
    where
        F: Fn(D0::Item) -> D1,
    {
        AndThen {
            decoder0,
            decoder1: None,
            and_then,
        }
    }
}
impl<D0, D1, F> Decode for AndThen<D0, D1, F>
where
    D0: Decode,
    D1: Decode,
    F: Fn(D0::Item) -> D1,
{
    type Item = D1::Item;

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        let mut item = None;
        while !buf.is_empty() && item.is_none() {
            if let Some(ref mut d) = self.decoder1 {
                if let Some(x) = d.decode(buf)? {
                    item = Some(x);
                }
            } else if let Some(d) = self.decoder0.decode(buf)?.map(&self.and_then) {
                self.decoder1 = Some(d);
            }
        }
        if item.is_some() {
            self.decoder1 = None;
        }
        Ok(item)
    }
}

#[derive(Debug)]
pub struct StartEncodingFrom<E, T, F> {
    encoder: E,
    _item: PhantomData<T>,
    from: F,
}
impl<E, T, F> StartEncodingFrom<E, T, F> {
    pub(crate) fn new(encoder: E, from: F) -> Self {
        StartEncodingFrom {
            encoder,
            _item: PhantomData,
            from,
        }
    }
}
impl<E, T, F> Encode for StartEncodingFrom<E, T, F>
where
    E: Encode,
    F: Fn(T) -> E::Item,
{
    type Item = T;

    fn encode(&mut self, buf: &mut EncodeBuf) -> Result<()> {
        track!(self.encoder.encode(buf))
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track!(self.encoder.start_encoding((self.from)(item)))
    }

    fn remaining_bytes(&self) -> Option<u64> {
        self.encoder.remaining_bytes()
    }
}

#[derive(Debug)]
pub struct Repeat<E, I> {
    encoder: E,
    items: Option<I>,
}
impl<E, I> Repeat<E, I> {
    pub(crate) fn new(encoder: E) -> Self {
        Repeat {
            encoder,
            items: None,
        }
    }
}
impl<E, I> Encode for Repeat<E, I>
where
    E: Encode,
    I: Iterator<Item = E::Item>,
{
    type Item = I;

    fn encode(&mut self, buf: &mut EncodeBuf) -> Result<()> {
        while !buf.is_empty() && self.items.is_some() {
            let old_buf_len = buf.len();
            track!(self.encoder.encode(buf))?;
            if old_buf_len == buf.len() {
                if let Some(item) = self.items.as_mut().and_then(|iter| iter.next()) {
                    track!(self.encoder.start_encoding(item))?;
                } else {
                    self.items = None;
                }
            }
        }
        Ok(())
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track_assert!(self.items.is_none(), ErrorKind::Full);
        self.items = Some(item);
        Ok(())
    }

    fn remaining_bytes(&self) -> Option<u64> {
        None
    }
}

#[derive(Debug)]
pub struct Optional<E>(E);
impl<E> Optional<E> {
    pub(crate) fn new(encoder: E) -> Self {
        Optional(encoder)
    }
}
impl<E: Encode> Encode for Optional<E> {
    type Item = Option<E::Item>;

    fn encode(&mut self, buf: &mut EncodeBuf) -> Result<()> {
        track!(self.0.encode(buf))
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        if let Some(item) = item {
            track!(self.0.start_encoding(item))?;
        }
        Ok(())
    }

    fn remaining_bytes(&self) -> Option<u64> {
        self.0.remaining_bytes()
    }
}

#[derive(Debug)]
pub struct Collect<D, T> {
    decoder: D,
    items: T,
}
impl<D, T: Default> Collect<D, T> {
    pub(crate) fn new(decoder: D) -> Self {
        Collect {
            decoder,
            items: T::default(),
        }
    }
}
impl<D, T> Decode for Collect<D, T>
where
    D: Decode,
    T: Extend<D::Item> + Default,
{
    type Item = T;

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        while !buf.is_empty() {
            let old_buf_len = buf.len();
            if let Some(item) = track!(self.decoder.decode(buf))? {
                self.items.extend(iter::once(item));
            } else if old_buf_len == buf.len() {
                break;
            }
        }
        if buf.is_eos() {
            Ok(Some(mem::replace(&mut self.items, T::default())))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug)]
pub struct Take<D> {
    decoder: D,
    limit: u64,
}
impl<D> Take<D> {
    pub(crate) fn new(decoder: D, limit: u64) -> Self {
        Take { decoder, limit }
    }
}
impl<D: Decode> Decode for Take<D> {
    type Item = D::Item;

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        let len = cmp::min(buf.len() as u64, self.limit) as usize;
        let remaining = self.limit - len as u64;
        let remaining = buf.remaining_bytes()
            .map_or(remaining, |n| cmp::min(n, remaining));

        let (item, consumed_len) = {
            let mut buf = DecodeBuf::with_remaining_bytes(&buf[..len], remaining);
            let item = track!(self.decoder.decode(&mut buf))?;
            let consumed_len = len - buf.len();
            (item, consumed_len)
        };

        self.limit -= consumed_len as u64;
        track!(buf.consume(consumed_len))?;
        Ok(item)
    }
}

#[derive(Debug)]
pub struct Validate<D, F, E> {
    decoder: D,
    validate: F,
    _error: PhantomData<E>,
}
impl<D, F, E> Validate<D, F, E> {
    pub(crate) fn new(decoder: D, validate: F) -> Self {
        Validate {
            decoder,
            validate,
            _error: PhantomData,
        }
    }
}
impl<D, F, E> Decode for Validate<D, F, E>
where
    D: Decode,
    F: for<'a> Fn(&'a D::Item) -> std::result::Result<(), E>,
    Error: From<E>,
{
    type Item = D::Item;

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        if let Some(item) = track!(self.decoder.decode(buf))? {
            (self.validate)(&item)?;
            Ok(Some(item))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug)]
pub struct IgnoreRest<D: Decode> {
    decoder: D,
    item: Option<D::Item>,
}
impl<D: Decode> IgnoreRest<D> {
    pub(crate) fn new(decoder: D) -> Self {
        IgnoreRest {
            decoder,
            item: None,
        }
    }
}
impl<D: Decode> Decode for IgnoreRest<D> {
    type Item = D::Item;

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        track_assert!(buf.remaining_bytes().is_some(), ErrorKind::InvalidInput);
        while !buf.is_empty() && self.item.is_none() {
            self.item = track!(self.decoder.decode(buf))?;
        }
        if self.item.is_some() {
            let len = buf.len();
            track!(buf.consume(len))?;
            if buf.is_eos() {
                return Ok(self.item.take());
            }
        }
        Ok(None)
    }
}
