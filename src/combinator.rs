use std;
use std::cmp;
use std::iter;
use std::marker::PhantomData;
use std::mem;

pub use chain::{DecoderChain, EncoderChain};

use {Decode, DecodeBuf, Encode, EncodeBuf, Error, ErrorKind, Result};
use marker::ExactBytesDecode;

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
        track!(self.decoder.decode(buf)).map(|r| r.map(&self.map))
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        self.decoder.requiring_bytes_hint()
    }
}
impl<D, T, F> ExactBytesDecode for Map<D, T, F>
where
    D: ExactBytesDecode,
    F: Fn(D::Item) -> T,
{
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
        track!(self.codec.decode(buf)).map_err(&self.map_err)
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        self.codec.requiring_bytes_hint()
    }
}
impl<D, F> ExactBytesDecode for MapErr<D, F>
where
    D: ExactBytesDecode,
    F: Fn(Error) -> Error,
{
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
        loop {
            if let Some(ref mut d) = self.decoder1 {
                item = track!(d.decode(buf))?;
                break;
            } else if let Some(d) = track!(self.decoder0.decode(buf))?.map(&self.and_then) {
                self.decoder1 = Some(d);
            } else {
                break;
            }
        }
        if item.is_some() {
            self.decoder1 = None;
        }
        Ok(item)
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        if let Some(ref d) = self.decoder1 {
            d.requiring_bytes_hint()
        } else {
            self.decoder0.requiring_bytes_hint()
        }
    }
}
impl<D0, D1, F> ExactBytesDecode for AndThen<D0, D1, F>
where
    D0: ExactBytesDecode,
    D1: ExactBytesDecode,
    F: Fn(D0::Item) -> D1,
{
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

// deref, deref_mut
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
        while let Some(item) = track!(self.decoder.decode(buf))? {
            self.items.extend(iter::once(item));
        }

        if buf.is_eos() || self.decoder.terminated() {
            Ok(Some(mem::replace(&mut self.items, T::default())))
        } else {
            Ok(None)
        }
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        self.decoder.requiring_bytes_hint()
    }
}
impl<D, T> ExactBytesDecode for Collect<D, T>
where
    D: ExactBytesDecode,
    T: Extend<D::Item> + Default,
{
}

// TODO: rename
#[derive(Debug)]
pub struct Take<D> {
    decoder: D,
    limit: u64,
}
impl<D> Take<D> {
    pub(crate) fn new(decoder: D, limit: u64) -> Self {
        Take { decoder, limit }
    }

    pub fn set_limit(&mut self, limit: u64) {
        self.limit = limit;
    }
}
impl<D: Decode> Decode for Take<D> {
    type Item = D::Item;

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        let min_len = cmp::min(buf.len() as u64, self.limit) as usize;
        let remaining_len = self.limit - min_len as u64;
        let remaining_len = buf.remaining_bytes()
            .map_or(remaining_len, |n| cmp::min(n, remaining_len));

        let (item, consumed_len) = {
            let mut limited_buf = DecodeBuf::with_remaining_bytes(&buf[..min_len], remaining_len);
            let item = track!(self.decoder.decode(&mut limited_buf);
                              self.limit, buf.len(), buf.remaining_bytes())?;
            let consumed_len = min_len - limited_buf.len();
            (item, consumed_len)
        };

        self.limit -= consumed_len as u64;
        track!(buf.consume(consumed_len))?;
        Ok(item)
    }
}
impl<D: Decode> ExactBytesDecode for Take<D> {
    fn requiring_bytes(&self) -> u64 {
        self.requiring_bytes_hint()
            .map_or(self.limit, |n| cmp::min(n, self.limit))
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
            track!((self.validate)(&item).map_err(Error::from))?;
            Ok(Some(item))
        } else {
            Ok(None)
        }
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        self.decoder.requiring_bytes_hint()
    }
}
impl<D, F, E> ExactBytesDecode for Validate<D, F, E>
where
    D: ExactBytesDecode,
    F: for<'a> Fn(&'a D::Item) -> std::result::Result<(), E>,
    Error: From<E>,
{
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
        track_assert!(
            buf.remaining_bytes().is_some(),
            ErrorKind::InvalidInput,
            "Cannot ignore infinity byte stream"
        );

        if self.item.is_none() {
            self.item = track!(self.decoder.decode(buf))?;
        }
        if self.item.is_some() {
            let rest = buf.len();
            track!(buf.consume(rest))?;
            if buf.is_eos() {
                return Ok(self.item.take());
            }
        }
        Ok(None)
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        if self.item.is_none() {
            self.decoder.requiring_bytes_hint()
        } else {
            None
        }
    }
}
