use std::marker::PhantomData;

use {Decode, DecodeBuf, Encode, EncodeBuf, ErrorKind, Result};

#[derive(Debug)]
pub struct StartDecoderChain;

#[derive(Debug)]
pub struct StartEncoderChain;
impl StartEncoderChain {
    pub fn chain<E: Encode>(&self, encoder: E) -> EncoderChain<Self, E, ()> {
        EncoderChain::new(StartEncoderChain, encoder)
    }
}

#[derive(Debug)]
pub struct EncoderChain<E0, E1, T> {
    inner: Chain<E0, E1>,
    _item: PhantomData<T>,
}
impl<E0, E1, I> EncoderChain<E0, E1, I> {
    pub(crate) fn new(e0: E0, e1: E1) -> Self {
        EncoderChain {
            inner: Chain::new(e0, e1),
            _item: PhantomData,
        }
    }
}
impl<E> Encode for EncoderChain<StartEncoderChain, E, ()>
where
    E: Encode,
{
    type Item = (E::Item,);

    fn encode(&mut self, buf: &mut EncodeBuf) -> Result<()> {
        self.inner.b.encode(buf)
    }

    fn start_encoding(&mut self, t: Self::Item) -> Result<()> {
        self.inner.b.start_encoding(t.0)
    }

    fn remaining_bytes(&self) -> Option<u64> {
        self.inner.b.remaining_bytes()
    }
}
impl<E0, E1, T0> Encode for EncoderChain<E0, E1, (T0,)>
where
    E0: Encode<Item = (T0,)>,
    E1: Encode,
{
    type Item = (T0, E1::Item);

    fn encode(&mut self, buf: &mut EncodeBuf) -> Result<()> {
        self.inner.encode(buf)
    }

    fn start_encoding(&mut self, t: Self::Item) -> Result<()> {
        self.inner.start_encoding(((t.0,), t.1))
    }

    fn remaining_bytes(&self) -> Option<u64> {
        self.inner.remaining_bytes()
    }
}
impl<E0, E1, T0, T1> Encode for EncoderChain<E0, E1, (T0, T1)>
where
    E0: Encode<Item = (T0, T1)>,
    E1: Encode,
{
    type Item = (T0, T1, E1::Item);

    fn encode(&mut self, buf: &mut EncodeBuf) -> Result<()> {
        self.inner.encode(buf)
    }

    fn start_encoding(&mut self, t: Self::Item) -> Result<()> {
        self.inner.start_encoding(((t.0, t.1), t.2))
    }

    fn remaining_bytes(&self) -> Option<u64> {
        self.inner.remaining_bytes()
    }
}
impl<E0, E1, T0, T1, T2> Encode for EncoderChain<E0, E1, (T0, T1, T2)>
where
    E0: Encode<Item = (T0, T1, T2)>,
    E1: Encode,
{
    type Item = (T0, T1, T2, E1::Item);

    fn encode(&mut self, buf: &mut EncodeBuf) -> Result<()> {
        self.inner.encode(buf)
    }

    fn start_encoding(&mut self, t: Self::Item) -> Result<()> {
        self.inner.start_encoding(((t.0, t.1, t.2), t.3))
    }

    fn remaining_bytes(&self) -> Option<u64> {
        self.inner.remaining_bytes()
    }
}
impl<E0, E1, T0, T1, T2, T3> Encode for EncoderChain<E0, E1, (T0, T1, T2, T3)>
where
    E0: Encode<Item = (T0, T1, T2, T3)>,
    E1: Encode,
{
    type Item = (T0, T1, T2, T3, E1::Item);

    fn encode(&mut self, buf: &mut EncodeBuf) -> Result<()> {
        self.inner.encode(buf)
    }

    fn start_encoding(&mut self, t: Self::Item) -> Result<()> {
        self.inner.start_encoding(((t.0, t.1, t.2, t.3), t.4))
    }

    fn remaining_bytes(&self) -> Option<u64> {
        self.inner.remaining_bytes()
    }
}
impl<E0, E1, T0, T1, T2, T3, T4> Encode for EncoderChain<E0, E1, (T0, T1, T2, T3, T4)>
where
    E0: Encode<Item = (T0, T1, T2, T3, T4)>,
    E1: Encode,
{
    type Item = (T0, T1, T2, T3, T4, E1::Item);

    fn encode(&mut self, buf: &mut EncodeBuf) -> Result<()> {
        self.inner.encode(buf)
    }

    fn start_encoding(&mut self, t: Self::Item) -> Result<()> {
        self.inner.start_encoding(((t.0, t.1, t.2, t.3, t.4), t.5))
    }

    fn remaining_bytes(&self) -> Option<u64> {
        self.inner.remaining_bytes()
    }
}
impl<E0, E1, T0, T1, T2, T3, T4, T5> Encode for EncoderChain<E0, E1, (T0, T1, T2, T3, T4, T5)>
where
    E0: Encode<Item = (T0, T1, T2, T3, T4, T5)>,
    E1: Encode,
{
    type Item = (T0, T1, T2, T3, T4, T5, E1::Item);

    fn encode(&mut self, buf: &mut EncodeBuf) -> Result<()> {
        self.inner.encode(buf)
    }

    fn start_encoding(&mut self, t: Self::Item) -> Result<()> {
        self.inner
            .start_encoding(((t.0, t.1, t.2, t.3, t.4, t.5), t.6))
    }

    fn remaining_bytes(&self) -> Option<u64> {
        self.inner.remaining_bytes()
    }
}
impl<E0, E1, T0, T1, T2, T3, T4, T5, T6> Encode
    for EncoderChain<E0, E1, (T0, T1, T2, T3, T4, T5, T6)>
where
    E0: Encode<Item = (T0, T1, T2, T3, T4, T5, T6)>,
    E1: Encode,
{
    type Item = (T0, T1, T2, T3, T4, T5, T6, E1::Item);

    fn encode(&mut self, buf: &mut EncodeBuf) -> Result<()> {
        self.inner.encode(buf)
    }

    fn start_encoding(&mut self, t: Self::Item) -> Result<()> {
        self.inner
            .start_encoding(((t.0, t.1, t.2, t.3, t.4, t.5, t.6), t.7))
    }

    fn remaining_bytes(&self) -> Option<u64> {
        self.inner.remaining_bytes()
    }
}

#[derive(Debug)]
struct Chain<A, B> {
    a: A,
    b: B,
    i: usize,
}
impl<A, B> Chain<A, B> {
    fn new(a: A, b: B) -> Self {
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
    // pub(crate) fn new(decoder: T) -> Self {
    //     Buffered {
    //         decoder,
    //         buffer: None,
    //     }
    // }

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