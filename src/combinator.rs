use std::marker::PhantomData;

pub use chain::EncoderChain;

use {Decode, DecodeBuf, Encode, EncodeBuf, Error, ErrorKind, Result};

#[derive(Debug)]
pub struct Map<T, U, F> {
    inner: T,
    map: F,
    _phantom: PhantomData<U>,
}
impl<T, U, F> Map<T, U, F> {
    pub(crate) fn new<V>(inner: T, map: F) -> Self
    where
        F: Fn(V) -> U,
    {
        Map {
            inner,
            map,
            _phantom: PhantomData,
        }
    }
}
impl<T, U, F> Decode for Map<T, U, F>
where
    T: Decode,
    F: Fn(T::Item) -> U,
{
    type Item = U;

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        self.inner.decode(buf).map(|r| r.map(&self.map))
    }
}

#[derive(Debug)]
pub struct MapErr<T, F> {
    codec: T,
    map_err: F,
}
impl<T, F> MapErr<T, F> {
    pub(crate) fn new(codec: T, map_err: F) -> Self
    where
        F: Fn(Error) -> Error,
    {
        MapErr { codec, map_err }
    }
}
impl<T, F> Decode for MapErr<T, F>
where
    T: Decode,
    F: Fn(Error) -> Error,
{
    type Item = T::Item;

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        self.codec.decode(buf).map_err(&self.map_err)
    }
}
impl<T, F> Encode for MapErr<T, F>
where
    T: Encode,
    F: Fn(Error) -> Error,
{
    type Item = T::Item;

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
pub struct AndThen<T, U, F> {
    decoder0: T,
    decoder1: Option<U>,
    and_then: F,
}
impl<T, U, F> AndThen<T, U, F> {
    pub(crate) fn new<V>(inner: T, and_then: F) -> Self
    where
        F: Fn(V) -> U,
    {
        AndThen {
            decoder0: inner,
            decoder1: None,
            and_then,
        }
    }
}
impl<T, U, F> Decode for AndThen<T, U, F>
where
    T: Decode,
    U: Decode,
    F: Fn(T::Item) -> U,
{
    type Item = U::Item;

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        let mut item = None;
        while !buf.is_empty() {
            if let Some(ref mut d) = self.decoder1 {
                if let Some(x) = d.decode(buf)? {
                    item = Some(x);
                    break;
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
