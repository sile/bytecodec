use std::marker::PhantomData;

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

    fn decode_size_hint(&self) -> usize {
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

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        self.inner.decode(buf).map_err(&self.map_err)
    }

    fn decode_size_hint(&self) -> usize {
        self.inner.decode_size_hint()
    }
}
impl<T, F> Encode for MapErr<T, F>
where
    T: Encode,
    F: Fn(Error) -> Error,
{
    type Item = T::Item;

    fn encode(&mut self, buf: &mut EncodeBuf) -> Result<()> {
        self.inner.encode(buf).map_err(&self.map_err)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<Option<Self::Item>> {
        self.inner.start_encoding(item).map_err(&self.map_err)
    }

    fn encode_size_hint(&self) -> usize {
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

    fn decode_size_hint(&self) -> usize {
        // TODO:
        if let Some(ref d) = self.decoder1 {
            d.decode_size_hint()
        } else {
            self.decoder0.decode_size_hint()
        }
    }
}

#[derive(Debug)]
pub struct Chain<A, B> {
    a: A,
    b: B,
    i: usize,
}
impl<A, B> Chain<A, B> {
    pub(crate) fn new(a: A, b: B) -> Self {
        Chain { a, b, i: 0 }
    }
}
impl<A, B> Encode for Chain<A, B>
where
    A: Encode,
    B: Encode,
{
    type Item = (A::Item, B::Item);

    fn encode(&mut self, buf: &mut EncodeBuf) -> Result<()> {
        while !buf.is_empty() && self.i < 2 {
            match self.i {
                0 => track!(self.a.encode(buf))?,
                1 => track!(self.b.encode(buf))?,
                _ => unreachable!(),
            }
            if buf.is_completed() {
                self.i += 1;
            }
        }
        Ok(())
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<Option<Self::Item>> {
        if self.i == 2 {
            self.i = 0;
            track_assert!(
                track!(self.a.start_encoding(item.0))?.is_none(),
                ErrorKind::Other
            );
            track_assert!(
                track!(self.b.start_encoding(item.1))?.is_none(),
                ErrorKind::Other
            );
            Ok(None)
        } else {
            Ok(Some(item))
        }
    }

    fn encode_size_hint(&self) -> usize {
        let mut size = 0;
        if self.i <= 0 {
            size += self.a.encode_size_hint();
        }
        if self.i <= 1 {
            size += self.b.encode_size_hint();
        }
        size
    }
}
impl<A, B> Decode for Chain<Buffered<A>, Buffered<B>>
where
    A: Decode,
    B: Decode,
{
    type Item = (A::Item, B::Item);

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        while !buf.is_empty() && self.i < 2 {
            let buf_len = buf.len();
            match self.i {
                0 => debug_assert!(track!(self.a.decode(buf))?.is_none()),
                1 => debug_assert!(track!(self.b.decode(buf))?.is_none()),
                _ => unreachable!(),
            };
            if buf_len == buf.len() {
                self.i += 1;
            }
        }
        if self.i == 2 {
            self.i = 0;
            let item0 = track_assert_some!(self.a.take_item(), ErrorKind::Other);
            let item1 = track_assert_some!(self.b.take_item(), ErrorKind::Other);
            Ok(Some((item0, item1)))
        } else {
            Ok(None)
        }
    }

    fn decode_size_hint(&self) -> usize {
        let mut size = 0;
        if self.i <= 0 {
            size += self.a.decode_size_hint();
        }
        if self.i <= 1 {
            size += self.b.decode_size_hint();
        }
        size
    }
}

#[derive(Debug)]
pub struct Buffered<T: Decode> {
    decoder: T,
    buffer: Option<T::Item>,
}
impl<T: Decode> Buffered<T> {
    pub(crate) fn new(decoder: T) -> Self {
        Buffered {
            decoder,
            buffer: None,
        }
    }

    fn take_item(&mut self) -> Option<T::Item> {
        self.buffer.take()
    }
}
impl<T: Decode> Decode for Buffered<T> {
    type Item = T::Item;

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        if self.buffer.is_none() {
            if let Some(item) = track!(self.decoder.decode(buf))? {
                self.buffer = Some(item);
            }
        }
        Ok(None)
    }

    fn decode_size_hint(&self) -> usize {
        if self.buffer.is_some() {
            0
        } else {
            self.decoder.decode_size_hint()
        }
    }
}

#[derive(Debug)]
pub struct Flatten<T, I> {
    decoder: T,
    _item: PhantomData<I>,
}
impl<T, I> Flatten<T, I> {
    pub(crate) fn new(decoder: T) -> Self {
        Flatten {
            decoder,
            _item: PhantomData,
        }
    }
}
impl<T, A, B, C> Decode for Flatten<T, (A, B, C)>
where
    T: Decode<Item = ((A, B), C)>,
{
    type Item = (A, B, C);

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        let item = track!(self.decoder.decode(buf))?;
        Ok(item.map(|((a, b), c)| (a, b, c)))
    }

    fn decode_size_hint(&self) -> usize {
        self.decoder.decode_size_hint()
    }
}
impl<T, A, B, C, D> Decode for Flatten<T, (A, B, C, D)>
where
    T: Decode<Item = (((A, B), C), D)>,
{
    type Item = (A, B, C, D);

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        let item = track!(self.decoder.decode(buf))?;
        Ok(item.map(|(((a, b), c), d)| (a, b, c, d)))
    }

    fn decode_size_hint(&self) -> usize {
        self.decoder.decode_size_hint()
    }
}
impl<T, A, B, C, D, E> Decode for Flatten<T, (A, B, C, D, E)>
where
    T: Decode<Item = ((((A, B), C), D), E)>,
{
    type Item = (A, B, C, D, E);

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        let item = track!(self.decoder.decode(buf))?;
        Ok(item.map(|((((a, b), c), d), e)| (a, b, c, d, e)))
    }

    fn decode_size_hint(&self) -> usize {
        self.decoder.decode_size_hint()
    }
}
impl<T, A, B, C, D, E, F> Decode for Flatten<T, (A, B, C, D, E, F)>
where
    T: Decode<Item = (((((A, B), C), D), E), F)>,
{
    type Item = (A, B, C, D, E, F);

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        let item = track!(self.decoder.decode(buf))?;
        Ok(item.map(|(((((a, b), c), d), e), f)| (a, b, c, d, e, f)))
    }

    fn decode_size_hint(&self) -> usize {
        self.decoder.decode_size_hint()
    }
}
impl<T, A, B, C, D, E, F, G> Decode for Flatten<T, (A, B, C, D, E, F, G)>
where
    T: Decode<Item = ((((((A, B), C), D), E), F), G)>,
{
    type Item = (A, B, C, D, E, F, G);

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        let item = track!(self.decoder.decode(buf))?;
        Ok(item.map(|((((((a, b), c), d), e), f), g)| (a, b, c, d, e, f, g)))
    }

    fn decode_size_hint(&self) -> usize {
        self.decoder.decode_size_hint()
    }
}
impl<T, A, B, C, D, E, F, G, H> Decode for Flatten<T, (A, B, C, D, E, F, G, H)>
where
    T: Decode<Item = (((((((A, B), C), D), E), F), G), H)>,
{
    type Item = (A, B, C, D, E, F, G, H);

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        let item = track!(self.decoder.decode(buf))?;
        Ok(item.map(|(((((((a, b), c), d), e), f), g), h)| (a, b, c, d, e, f, g, h)))
    }

    fn decode_size_hint(&self) -> usize {
        self.decoder.decode_size_hint()
    }
}
