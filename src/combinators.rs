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

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        self.inner.start_encoding(item).map_err(&self.map_err)
    }

    fn remaining_bytes(&self) -> Option<u64> {
        self.inner.remaining_bytes()
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
pub struct Chain2<A, B, I> {
    inner: Chain<A, B>,
    _item: PhantomData<I>,
}
impl<A, B, I> Chain2<A, B, I> {
    pub(crate) fn new(a: A, b: B) -> Self {
        Chain2 {
            inner: Chain::new(a, b),
            _item: PhantomData,
        }
    }
}
impl<A, B> Encode for Chain2<A, B, ()>
where
    A: Encode<Item = ()>,
    B: Encode,
{
    type Item = (B::Item,);

    fn encode(&mut self, buf: &mut EncodeBuf) -> Result<()> {
        self.inner.encode(buf)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        self.inner.start_encoding(((), item.0))
    }

    fn remaining_bytes(&self) -> Option<u64> {
        self.inner.remaining_bytes()
    }
}
impl<T0, T1, A> Encode for Chain2<T0, T1, (A,)>
where
    T0: Encode<Item = (A,)>,
    T1: Encode,
{
    type Item = (A, T1::Item);

    fn encode(&mut self, buf: &mut EncodeBuf) -> Result<()> {
        self.inner.encode(buf)
    }

    fn start_encoding(&mut self, (a, b): Self::Item) -> Result<()> {
        self.inner.start_encoding(((a,), b))
    }

    fn remaining_bytes(&self) -> Option<u64> {
        self.inner.remaining_bytes()
    }
}
impl<T0, T1, A, B> Encode for Chain2<T0, T1, (A, B)>
where
    T0: Encode<Item = (A, B)>,
    T1: Encode,
{
    type Item = (A, B, T1::Item);

    fn encode(&mut self, buf: &mut EncodeBuf) -> Result<()> {
        self.inner.encode(buf)
    }

    fn start_encoding(&mut self, (a, b, c): Self::Item) -> Result<()> {
        self.inner.start_encoding(((a, b), c))
    }

    fn remaining_bytes(&self) -> Option<u64> {
        self.inner.remaining_bytes()
    }
}
impl<T0, T1, A, B, C> Encode for Chain2<T0, T1, (A, B, C)>
where
    T0: Encode<Item = (A, B, C)>,
    T1: Encode,
{
    type Item = (A, B, C, T1::Item);

    fn encode(&mut self, buf: &mut EncodeBuf) -> Result<()> {
        self.inner.encode(buf)
    }

    fn start_encoding(&mut self, (a, b, c, d): Self::Item) -> Result<()> {
        self.inner.start_encoding(((a, b, c), d))
    }

    fn remaining_bytes(&self) -> Option<u64> {
        self.inner.remaining_bytes()
    }
}
impl<T0, T1, A, B, C, D> Encode for Chain2<T0, T1, (A, B, C, D)>
where
    T0: Encode<Item = (A, B, C, D)>,
    T1: Encode,
{
    type Item = (A, B, C, D, T1::Item);

    fn encode(&mut self, buf: &mut EncodeBuf) -> Result<()> {
        self.inner.encode(buf)
    }

    fn start_encoding(&mut self, (a, b, c, d, e): Self::Item) -> Result<()> {
        self.inner.start_encoding(((a, b, c, d), e))
    }

    fn remaining_bytes(&self) -> Option<u64> {
        self.inner.remaining_bytes()
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
            let old_buf_len = buf.len();
            match self.i {
                0 => track!(self.a.encode(buf))?,
                1 => track!(self.b.encode(buf))?,
                _ => unreachable!(),
            }
            if old_buf_len == buf.len() {
                self.i += 1;
            }
        }
        Ok(())
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track_assert_eq!(self.i, 2, ErrorKind::Full);
        self.i = 0;
        track!(self.a.start_encoding(item.0))?;
        track!(self.b.start_encoding(item.1))?;
        Ok(())
    }

    fn remaining_bytes(&self) -> Option<u64> {
        let mut size = Some(0);
        if self.i <= 0 {
            size = size.and_then(|x| self.a.remaining_bytes().map(|y| x + y));
        }
        if self.i <= 1 {
            size = size.and_then(|x| self.b.remaining_bytes().map(|y| x + y));
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
}

#[derive(Debug)]
pub struct MapFrom<T, U, F> {
    encoder: T,
    _item: PhantomData<U>,
    map_from: F,
}
impl<T, U, F> MapFrom<T, U, F> {
    pub(crate) fn new(encoder: T, map_from: F) -> Self {
        MapFrom {
            encoder,
            _item: PhantomData,
            map_from,
        }
    }
}
impl<T, U, F> Encode for MapFrom<T, U, F>
where
    T: Encode,
    F: Fn(U) -> T::Item,
{
    type Item = U;

    fn encode(&mut self, buf: &mut EncodeBuf) -> Result<()> {
        track!(self.encoder.encode(buf))
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track!(self.encoder.start_encoding((self.map_from)(item)))
    }

    fn remaining_bytes(&self) -> Option<u64> {
        self.encoder.remaining_bytes()
    }
}

#[derive(Debug)]
pub struct StartChain;
impl Encode for StartChain {
    type Item = ();

    fn encode(&mut self, _buf: &mut EncodeBuf) -> Result<()> {
        Ok(())
    }

    fn start_encoding(&mut self, _item: Self::Item) -> Result<()> {
        Ok(())
    }
}
