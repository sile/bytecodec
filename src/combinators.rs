use std::marker::PhantomData;

use {Decode, Encode, Error, Result};

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

    fn decode(&mut self, buf: &[u8], eos: bool) -> Result<usize> {
        self.inner.decode(buf, eos)
    }

    fn pop_item(&mut self) -> Result<Option<Self::Item>> {
        self.inner.pop_item().map(|r| r.map(&self.map))
    }

    fn decode_size_hint(&self) -> Option<usize> {
        self.inner.decode_size_hint()
    }
}

#[derive(Debug)]
pub struct MapErr<T, F> {
    inner: T,
    map_err: F,
}
impl<T, F> MapErr<T, F> {
    pub(crate) fn new(inner: T, map_err: F) -> Self
    where
        F: Fn(Error) -> Error,
    {
        MapErr { inner, map_err }
    }
}
impl<T, F> Decode for MapErr<T, F>
where
    T: Decode,
    F: Fn(Error) -> Error,
{
    type Item = T::Item;

    fn decode(&mut self, buf: &[u8], eos: bool) -> Result<usize> {
        self.inner.decode(buf, eos).map_err(&self.map_err)
    }

    fn pop_item(&mut self) -> Result<Option<Self::Item>> {
        self.inner.pop_item().map_err(&self.map_err)
    }

    fn decode_size_hint(&self) -> Option<usize> {
        self.inner.decode_size_hint()
    }
}
impl<T, F> Encode for MapErr<T, F>
where
    T: Encode,
    F: Fn(Error) -> Error,
{
    type Item = T::Item;

    fn encode(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.inner.encode(buf).map_err(&self.map_err)
    }

    fn push_item(&mut self, item: Self::Item) -> Result<Option<Self::Item>> {
        self.inner.push_item(item).map_err(&self.map_err)
    }

    fn encode_size_hint(&self) -> Option<usize> {
        self.inner.encode_size_hint()
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

    fn decode(&mut self, buf: &[u8], eos: bool) -> Result<usize> {
        if let Some(ref mut d) = self.decoder1 {
            d.decode(buf, eos)
        } else {
            self.decoder0.decode(buf, eos)
        }
    }

    fn pop_item(&mut self) -> Result<Option<Self::Item>> {
        loop {
            if self.decoder1.is_some() {
                let item = self.decoder1.as_mut().expect("Never fails").pop_item()?;
                if item.is_some() {
                    self.decoder1 = None;
                }
                return Ok(item);
            } else {
                if let Some(d) = self.decoder0.pop_item()?.map(&self.and_then) {
                    self.decoder1 = Some(d);
                } else {
                    return Ok(None);
                }
            }
        }
    }

    fn decode_size_hint(&self) -> Option<usize> {
        // TODO:
        if let Some(ref d) = self.decoder1 {
            d.decode_size_hint()
        } else {
            self.decoder0.decode_size_hint()
        }
    }
}
