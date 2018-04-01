use std::marker::PhantomData;

use {Decode, DecodeBuf, Encode, EncodeBuf, ErrorKind, ExactBytesEncode, Result};

/// An object for starting a chain of decoders.
///
/// # Examples
///
/// ```
/// use bytecodec::{Decode, DecodeBuf, DecodeExt, StartDecoderChain};
/// use bytecodec::fixnum::U8Decoder;
///
/// let mut decoder = StartDecoderChain
///     .chain(U8Decoder::new())
///     .chain(U8Decoder::new())
///     .chain(U8Decoder::new());
///
/// let mut input = DecodeBuf::new(b"foobar");
///
/// let item = decoder.decode(&mut input).unwrap();
/// assert_eq!(item, Some((b'f', b'o', b'o')));
///
/// let item = decoder.decode(&mut input).unwrap();
/// assert_eq!(item, Some((b'b', b'a', b'r')));
/// ```
#[derive(Debug)]
pub struct StartDecoderChain;
impl StartDecoderChain {
    /// Starts decoders chain.
    pub fn chain<D: Decode>(&self, decoder: D) -> DecoderChain<Self, D, ()> {
        DecoderChain::new(StartDecoderChain, decoder)
    }
}

/// Combinator for connecting decoders to a chain.
///
/// This is created by calling `StartDecoderChain::chain` or `DecodeExt::chain` methods.
#[derive(Debug)]
pub struct DecoderChain<D0, D1, T>(Chain<Buffered<D0, T>, D1>);
impl<D0, D1, T> DecoderChain<D0, D1, T> {
    pub(crate) fn new(d0: D0, d1: D1) -> Self {
        DecoderChain(Chain::new(Buffered::new(d0), d1))
    }
}
impl<D> Decode for DecoderChain<StartDecoderChain, D, ()>
where
    D: Decode,
{
    type Item = (D::Item,);

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        Ok(self.0.b.decode(buf)?.map(|i| (i,)))
    }

    fn has_terminated(&self) -> bool {
        self.0.b.has_terminated()
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        self.0.b.requiring_bytes_hint()
    }
}
impl<D0, D1, T0> Decode for DecoderChain<D0, D1, (T0,)>
where
    D0: Decode<Item = (T0,)>,
    D1: Decode,
{
    type Item = (T0, D1::Item);

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        Ok(self.0.decode(buf)?.map(|(t, i)| (t.0, i)))
    }

    fn has_terminated(&self) -> bool {
        self.0.has_terminated()
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        self.0.requiring_bytes_hint()
    }
}
impl<D0, D1, T0, T1> Decode for DecoderChain<D0, D1, (T0, T1)>
where
    D0: Decode<Item = (T0, T1)>,
    D1: Decode,
{
    type Item = (T0, T1, D1::Item);

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        Ok(self.0.decode(buf)?.map(|(t, i)| (t.0, t.1, i)))
    }

    fn has_terminated(&self) -> bool {
        self.0.has_terminated()
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        self.0.requiring_bytes_hint()
    }
}
impl<D0, D1, T0, T1, T2> Decode for DecoderChain<D0, D1, (T0, T1, T2)>
where
    D0: Decode<Item = (T0, T1, T2)>,
    D1: Decode,
{
    type Item = (T0, T1, T2, D1::Item);

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        Ok(self.0.decode(buf)?.map(|(t, i)| (t.0, t.1, t.2, i)))
    }

    fn has_terminated(&self) -> bool {
        self.0.has_terminated()
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        self.0.requiring_bytes_hint()
    }
}
impl<D0, D1, T0, T1, T2, T3> Decode for DecoderChain<D0, D1, (T0, T1, T2, T3)>
where
    D0: Decode<Item = (T0, T1, T2, T3)>,
    D1: Decode,
{
    type Item = (T0, T1, T2, T3, D1::Item);

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        Ok(self.0.decode(buf)?.map(|(t, i)| (t.0, t.1, t.2, t.3, i)))
    }

    fn has_terminated(&self) -> bool {
        self.0.has_terminated()
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        self.0.requiring_bytes_hint()
    }
}
impl<D0, D1, T0, T1, T2, T3, T4> Decode for DecoderChain<D0, D1, (T0, T1, T2, T3, T4)>
where
    D0: Decode<Item = (T0, T1, T2, T3, T4)>,
    D1: Decode,
{
    type Item = (T0, T1, T2, T3, T4, D1::Item);

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        Ok(self.0
            .decode(buf)?
            .map(|(t, i)| (t.0, t.1, t.2, t.3, t.4, i)))
    }

    fn has_terminated(&self) -> bool {
        self.0.has_terminated()
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        self.0.requiring_bytes_hint()
    }
}
impl<D0, D1, T0, T1, T2, T3, T4, T5> Decode for DecoderChain<D0, D1, (T0, T1, T2, T3, T4, T5)>
where
    D0: Decode<Item = (T0, T1, T2, T3, T4, T5)>,
    D1: Decode,
{
    type Item = (T0, T1, T2, T3, T4, T5, D1::Item);

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        Ok(self.0
            .decode(buf)?
            .map(|(t, i)| (t.0, t.1, t.2, t.3, t.4, t.5, i)))
    }

    fn has_terminated(&self) -> bool {
        self.0.has_terminated()
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        self.0.requiring_bytes_hint()
    }
}
impl<D0, D1, T0, T1, T2, T3, T4, T5, T6> Decode
    for DecoderChain<D0, D1, (T0, T1, T2, T3, T4, T5, T6)>
where
    D0: Decode<Item = (T0, T1, T2, T3, T4, T5, T6)>,
    D1: Decode,
{
    type Item = (T0, T1, T2, T3, T4, T5, T6, D1::Item);

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        Ok(self.0
            .decode(buf)?
            .map(|(t, i)| (t.0, t.1, t.2, t.3, t.4, t.5, t.6, i)))
    }

    fn has_terminated(&self) -> bool {
        self.0.has_terminated()
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        self.0.requiring_bytes_hint()
    }
}

/// An object for starting a chain of encoders.
///
/// # Examples
///
/// ```
/// use bytecodec::{Encode, EncodeBuf, EncodeExt, StartEncoderChain};
/// use bytecodec::bytes::Utf8Encoder;
/// use bytecodec::fixnum::U8Encoder;
///
/// let mut output = [0; 4];
/// let mut encoder = StartEncoderChain
///     .chain(U8Encoder::new())
///     .chain(Utf8Encoder::new())
///     .map_from(|s: String| (s.len() as u8, s));
/// {
///     encoder.start_encoding("foo".to_owned()).unwrap();
///     encoder.encode(&mut EncodeBuf::new(&mut output)).unwrap();
/// }
/// assert_eq!(output.as_ref(), b"\x03foo");
/// ```
#[derive(Debug)]
pub struct StartEncoderChain;
impl StartEncoderChain {
    /// Starts encoders chain.
    pub fn chain<E: Encode>(&self, encoder: E) -> EncoderChain<Self, E, ()> {
        EncoderChain::new(StartEncoderChain, encoder)
    }
}
impl Encode for StartEncoderChain {
    type Item = ();

    fn encode(&mut self, _buf: &mut EncodeBuf) -> Result<()> {
        track_panic!(ErrorKind::Other)
    }

    fn start_encoding(&mut self, _item: Self::Item) -> Result<()> {
        track_panic!(ErrorKind::Other)
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        Some(0)
    }
}
impl ExactBytesEncode for StartEncoderChain {}

/// Combinator for connecting encoders to a chain.
///
/// This is created by calling `StartEncoderChain::chain` or `EncodeExt::chain` methods.
#[derive(Debug)]
pub struct EncoderChain<E0, E1, T> {
    inner: Chain<E0, E1>,
    _item: PhantomData<T>,
}
impl<E0, E1, T> EncoderChain<E0, E1, T> {
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

    fn requiring_bytes_hint(&self) -> Option<u64> {
        self.inner.b.requiring_bytes_hint()
    }

    fn is_completed(&self) -> bool {
        self.inner.b.is_completed()
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

    fn requiring_bytes_hint(&self) -> Option<u64> {
        self.inner.requiring_bytes_hint()
    }

    fn is_completed(&self) -> bool {
        self.inner.is_completed()
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

    fn requiring_bytes_hint(&self) -> Option<u64> {
        self.inner.requiring_bytes_hint()
    }

    fn is_completed(&self) -> bool {
        self.inner.is_completed()
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

    fn requiring_bytes_hint(&self) -> Option<u64> {
        self.inner.requiring_bytes_hint()
    }

    fn is_completed(&self) -> bool {
        self.inner.is_completed()
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

    fn requiring_bytes_hint(&self) -> Option<u64> {
        self.inner.requiring_bytes_hint()
    }

    fn is_completed(&self) -> bool {
        self.inner.is_completed()
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

    fn requiring_bytes_hint(&self) -> Option<u64> {
        self.inner.requiring_bytes_hint()
    }

    fn is_completed(&self) -> bool {
        self.inner.is_completed()
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

    fn requiring_bytes_hint(&self) -> Option<u64> {
        self.inner.requiring_bytes_hint()
    }

    fn is_completed(&self) -> bool {
        self.inner.is_completed()
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

    fn requiring_bytes_hint(&self) -> Option<u64> {
        self.inner.requiring_bytes_hint()
    }

    fn is_completed(&self) -> bool {
        self.inner.is_completed()
    }
}
impl<E0, E1, T> ExactBytesEncode for EncoderChain<E0, E1, T>
where
    Self: Encode,
    E0: ExactBytesEncode,
    E1: ExactBytesEncode,
{
    fn requiring_bytes(&self) -> u64 {
        self.inner.requiring_bytes()
    }
}

#[derive(Debug)]
struct Chain<A, B> {
    a: A,
    b: B,
}
impl<A, B> Chain<A, B> {
    fn new(a: A, b: B) -> Self {
        Chain { a, b }
    }
}
impl<A, B> Decode for Chain<Buffered<A, A::Item>, B>
where
    A: Decode,
    B: Decode,
{
    type Item = (A::Item, B::Item);

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        let item = loop {
            if self.a.item.is_none() {
                track!(self.a.decode(buf))?;
                if self.a.item.is_none() {
                    break None;
                }
            }
            if let Some(b) = track!(self.b.decode(buf))? {
                let a = self.a.item.take().expect("Never fails");
                break Some((a, b));
            } else {
                break None;
            }
        };
        Ok(item)
    }

    fn has_terminated(&self) -> bool {
        if self.a.item.is_none() {
            self.a.has_terminated()
        } else {
            self.b.has_terminated()
        }
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        let a = self.a
            .item
            .as_ref()
            .map(|_| 0)
            .or_else(|| self.a.requiring_bytes_hint());
        let b = self.b.requiring_bytes_hint();
        match (a, b) {
            (Some(a), Some(b)) => Some(a + b),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        }
    }
}
impl<A, B> Encode for Chain<A, B>
where
    A: Encode,
    B: Encode,
{
    type Item = (A::Item, B::Item);

    fn encode(&mut self, buf: &mut EncodeBuf) -> Result<()> {
        while !buf.is_empty() && !self.is_completed() {
            if !self.a.is_completed() {
                track!(self.a.encode(buf))?;
            } else {
                track!(self.b.encode(buf))?;
            }
        }
        Ok(())
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track_assert!(self.a.is_completed(), ErrorKind::EncoderFull);
        track_assert!(self.b.is_completed(), ErrorKind::EncoderFull);
        track!(self.a.start_encoding(item.0))?;
        track!(self.b.start_encoding(item.1))?;
        Ok(())
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        self.a
            .requiring_bytes_hint()
            .and_then(|a| self.b.requiring_bytes_hint().map(|b| a + b))
    }

    fn is_completed(&self) -> bool {
        self.b.is_completed()
    }
}
impl<E0, E1> ExactBytesEncode for Chain<E0, E1>
where
    E0: ExactBytesEncode,
    E1: ExactBytesEncode,
{
    fn requiring_bytes(&self) -> u64 {
        self.a.requiring_bytes() + self.b.requiring_bytes()
    }
}

#[derive(Debug)]
struct Buffered<T, I> {
    decoder: T,
    item: Option<I>,
}
impl<T, I> Buffered<T, I> {
    fn new(decoder: T) -> Self {
        Buffered {
            decoder,
            item: None,
        }
    }
}
impl<T: Decode> Decode for Buffered<T, T::Item> {
    type Item = T::Item;

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        if self.item.is_none() {
            self.item = track!(self.decoder.decode(buf))?;
        }
        Ok(None)
    }

    fn has_terminated(&self) -> bool {
        if self.item.is_some() {
            true
        } else {
            self.decoder.has_terminated()
        }
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        if self.item.is_some() {
            Some(0)
        } else {
            self.decoder.requiring_bytes_hint()
        }
    }
}

#[cfg(test)]
mod test {
    use {Decode, DecodeBuf, DecodeExt, StartDecoderChain};
    use fixnum::U8Decoder;

    #[test]
    fn it_works() {
        let mut decoder = StartDecoderChain
            .chain(U8Decoder::new())
            .chain(U8Decoder::new())
            .chain(U8Decoder::new());

        assert_eq!(
            track_try_unwrap!(decoder.decode(&mut DecodeBuf::new(b"foo"))),
            Some((b'f', b'o', b'o'))
        );
    }
}
