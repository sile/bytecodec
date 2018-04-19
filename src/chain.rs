use std::fmt;
use std::marker::PhantomData;

use {ByteCount, Decode, Encode, Eos, ErrorKind, ExactBytesEncode, Result};

/// An object for starting a chain of decoders.
///
/// # Examples
///
/// ```
/// use bytecodec::{Decode, DecodeExt, StartDecoderChain};
/// use bytecodec::fixnum::U8Decoder;
/// use bytecodec::io::IoDecodeExt;
///
/// let mut decoder = StartDecoderChain
///     .chain(U8Decoder::new())
///     .chain(U8Decoder::new())
///     .chain(U8Decoder::new());
///
/// let mut input = &b"foobar"[..];
///
/// let item = decoder.decode_exact(&mut input).unwrap();
/// assert_eq!(item, (b'f', b'o', b'o'));
///
/// let item = decoder.decode_exact(&mut input).unwrap();
/// assert_eq!(item, (b'b', b'a', b'r'));
/// ```
#[derive(Debug)]
pub struct StartDecoderChain;
impl StartDecoderChain {
    /// Starts decoders chain.
    pub fn chain<D: Decode>(&self, decoder: D) -> DecoderChain<Self, D, ()> {
        DecoderChain::new(StartDecoderChain, decoder)
    }
}
impl Decode for StartDecoderChain {
    type Item = ();

    fn decode(&mut self, _buf: &[u8], _eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        track_panic!(ErrorKind::Other)
    }

    fn has_terminated(&self) -> bool {
        false
    }

    fn requiring_bytes(&self) -> ByteCount {
        ByteCount::Unknown
    }
}

/// Combinator for connecting decoders to a chain.
///
/// This is created by calling `StartDecoderChain::chain` or `DecodeExt::chain` methods.
#[derive(Debug)]
pub struct DecoderChain<D0: Decode, D1, T>(Chain<Buffered<D0>, D1>, PhantomData<T>);
impl<D0: Decode, D1, T> DecoderChain<D0, D1, T> {
    pub(crate) fn new(d0: D0, d1: D1) -> Self {
        DecoderChain(Chain::new(Buffered::new(d0), d1), PhantomData)
    }
}
impl<D> Decode for DecoderChain<StartDecoderChain, D, ()>
where
    D: Decode,
{
    type Item = (D::Item,);

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        self.0.b.decode(buf, eos).map(|(n, i)| (n, i.map(|i| (i,))))
    }

    fn has_terminated(&self) -> bool {
        self.0.b.has_terminated()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.0.b.requiring_bytes()
    }
}
impl<D0, D1, T0> Decode for DecoderChain<D0, D1, (T0,)>
where
    D0: Decode<Item = (T0,)>,
    D1: Decode,
{
    type Item = (T0, D1::Item);

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        self.0
            .decode(buf, eos)
            .map(|(n, x)| (n, x.map(|(t, i)| (t.0, i))))
    }

    fn has_terminated(&self) -> bool {
        self.0.has_terminated()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.0.requiring_bytes()
    }
}
impl<D0, D1, T0, T1> Decode for DecoderChain<D0, D1, (T0, T1)>
where
    D0: Decode<Item = (T0, T1)>,
    D1: Decode,
{
    type Item = (T0, T1, D1::Item);

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        self.0
            .decode(buf, eos)
            .map(|(n, x)| (n, x.map(|(t, i)| (t.0, t.1, i))))
    }

    fn has_terminated(&self) -> bool {
        self.0.has_terminated()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.0.requiring_bytes()
    }
}
impl<D0, D1, T0, T1, T2> Decode for DecoderChain<D0, D1, (T0, T1, T2)>
where
    D0: Decode<Item = (T0, T1, T2)>,
    D1: Decode,
{
    type Item = (T0, T1, T2, D1::Item);

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        self.0
            .decode(buf, eos)
            .map(|(n, x)| (n, x.map(|(t, i)| (t.0, t.1, t.2, i))))
    }

    fn has_terminated(&self) -> bool {
        self.0.has_terminated()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.0.requiring_bytes()
    }
}
impl<D0, D1, T0, T1, T2, T3> Decode for DecoderChain<D0, D1, (T0, T1, T2, T3)>
where
    D0: Decode<Item = (T0, T1, T2, T3)>,
    D1: Decode,
{
    type Item = (T0, T1, T2, T3, D1::Item);

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        self.0
            .decode(buf, eos)
            .map(|(n, x)| (n, x.map(|(t, i)| (t.0, t.1, t.2, t.3, i))))
    }

    fn has_terminated(&self) -> bool {
        self.0.has_terminated()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.0.requiring_bytes()
    }
}
impl<D0, D1, T0, T1, T2, T3, T4> Decode for DecoderChain<D0, D1, (T0, T1, T2, T3, T4)>
where
    D0: Decode<Item = (T0, T1, T2, T3, T4)>,
    D1: Decode,
{
    type Item = (T0, T1, T2, T3, T4, D1::Item);

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        self.0
            .decode(buf, eos)
            .map(|(n, x)| (n, x.map(|(t, i)| (t.0, t.1, t.2, t.3, t.4, i))))
    }

    fn has_terminated(&self) -> bool {
        self.0.has_terminated()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.0.requiring_bytes()
    }
}
impl<D0, D1, T0, T1, T2, T3, T4, T5> Decode for DecoderChain<D0, D1, (T0, T1, T2, T3, T4, T5)>
where
    D0: Decode<Item = (T0, T1, T2, T3, T4, T5)>,
    D1: Decode,
{
    type Item = (T0, T1, T2, T3, T4, T5, D1::Item);

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        self.0
            .decode(buf, eos)
            .map(|(n, x)| (n, x.map(|(t, i)| (t.0, t.1, t.2, t.3, t.4, t.5, i))))
    }

    fn has_terminated(&self) -> bool {
        self.0.has_terminated()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.0.requiring_bytes()
    }
}
impl<D0, D1, T0, T1, T2, T3, T4, T5, T6> Decode
    for DecoderChain<D0, D1, (T0, T1, T2, T3, T4, T5, T6)>
where
    D0: Decode<Item = (T0, T1, T2, T3, T4, T5, T6)>,
    D1: Decode,
{
    type Item = (T0, T1, T2, T3, T4, T5, T6, D1::Item);

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        self.0
            .decode(buf, eos)
            .map(|(n, x)| (n, x.map(|(t, i)| (t.0, t.1, t.2, t.3, t.4, t.5, t.6, i))))
    }

    fn has_terminated(&self) -> bool {
        self.0.has_terminated()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.0.requiring_bytes()
    }
}

/// An object for starting a chain of encoders.
///
/// # Examples
///
/// ```
/// use bytecodec::{Encode, EncodeExt, StartEncoderChain};
/// use bytecodec::bytes::Utf8Encoder;
/// use bytecodec::fixnum::U8Encoder;
/// use bytecodec::io::IoEncodeExt;
///
/// let mut output = Vec::new();
/// let mut encoder = StartEncoderChain
///     .chain(U8Encoder::new())
///     .chain(Utf8Encoder::new())
///     .map_from(|s: String| (s.len() as u8, s));
/// encoder.start_encoding("foo".to_owned()).unwrap();
/// encoder.encode_all(&mut output).unwrap();
/// assert_eq!(output, b"\x03foo");
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

    fn encode(&mut self, _buf: &mut [u8], _eos: Eos) -> Result<usize> {
        track_panic!(ErrorKind::Other)
    }

    fn start_encoding(&mut self, _item: Self::Item) -> Result<()> {
        track_panic!(ErrorKind::Other)
    }

    fn is_idle(&self) -> bool {
        true
    }

    fn requiring_bytes(&self) -> ByteCount {
        ByteCount::Finite(0)
    }

    fn cancel(&mut self) -> Result<()> {
        Ok(())
    }
}
impl ExactBytesEncode for StartEncoderChain {
    fn exact_requiring_bytes(&self) -> u64 {
        0
    }
}

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

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        self.inner.b.encode(buf, eos)
    }

    fn start_encoding(&mut self, t: Self::Item) -> Result<()> {
        self.inner.b.start_encoding(t.0)
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.inner.b.requiring_bytes()
    }

    fn is_idle(&self) -> bool {
        self.inner.b.is_idle()
    }

    fn cancel(&mut self) -> Result<()> {
        track!(self.inner.b.cancel())
    }
}
impl<E0, E1, T0> Encode for EncoderChain<E0, E1, (T0,)>
where
    E0: Encode<Item = (T0,)>,
    E1: Encode,
{
    type Item = (T0, E1::Item);

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        self.inner.encode(buf, eos)
    }

    fn start_encoding(&mut self, t: Self::Item) -> Result<()> {
        self.inner.start_encoding(((t.0,), t.1))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.inner.requiring_bytes()
    }

    fn is_idle(&self) -> bool {
        self.inner.is_idle()
    }

    fn cancel(&mut self) -> Result<()> {
        track!(self.inner.cancel())
    }
}
impl<E0, E1, T0, T1> Encode for EncoderChain<E0, E1, (T0, T1)>
where
    E0: Encode<Item = (T0, T1)>,
    E1: Encode,
{
    type Item = (T0, T1, E1::Item);

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        self.inner.encode(buf, eos)
    }

    fn start_encoding(&mut self, t: Self::Item) -> Result<()> {
        self.inner.start_encoding(((t.0, t.1), t.2))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.inner.requiring_bytes()
    }

    fn is_idle(&self) -> bool {
        self.inner.is_idle()
    }

    fn cancel(&mut self) -> Result<()> {
        track!(self.inner.cancel())
    }
}
impl<E0, E1, T0, T1, T2> Encode for EncoderChain<E0, E1, (T0, T1, T2)>
where
    E0: Encode<Item = (T0, T1, T2)>,
    E1: Encode,
{
    type Item = (T0, T1, T2, E1::Item);

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        self.inner.encode(buf, eos)
    }

    fn start_encoding(&mut self, t: Self::Item) -> Result<()> {
        self.inner.start_encoding(((t.0, t.1, t.2), t.3))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.inner.requiring_bytes()
    }

    fn is_idle(&self) -> bool {
        self.inner.is_idle()
    }

    fn cancel(&mut self) -> Result<()> {
        track!(self.inner.cancel())
    }
}
impl<E0, E1, T0, T1, T2, T3> Encode for EncoderChain<E0, E1, (T0, T1, T2, T3)>
where
    E0: Encode<Item = (T0, T1, T2, T3)>,
    E1: Encode,
{
    type Item = (T0, T1, T2, T3, E1::Item);

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        self.inner.encode(buf, eos)
    }

    fn start_encoding(&mut self, t: Self::Item) -> Result<()> {
        self.inner.start_encoding(((t.0, t.1, t.2, t.3), t.4))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.inner.requiring_bytes()
    }

    fn is_idle(&self) -> bool {
        self.inner.is_idle()
    }

    fn cancel(&mut self) -> Result<()> {
        track!(self.inner.cancel())
    }
}
impl<E0, E1, T0, T1, T2, T3, T4> Encode for EncoderChain<E0, E1, (T0, T1, T2, T3, T4)>
where
    E0: Encode<Item = (T0, T1, T2, T3, T4)>,
    E1: Encode,
{
    type Item = (T0, T1, T2, T3, T4, E1::Item);

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        self.inner.encode(buf, eos)
    }

    fn start_encoding(&mut self, t: Self::Item) -> Result<()> {
        self.inner.start_encoding(((t.0, t.1, t.2, t.3, t.4), t.5))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.inner.requiring_bytes()
    }

    fn is_idle(&self) -> bool {
        self.inner.is_idle()
    }

    fn cancel(&mut self) -> Result<()> {
        track!(self.inner.cancel())
    }
}
impl<E0, E1, T0, T1, T2, T3, T4, T5> Encode for EncoderChain<E0, E1, (T0, T1, T2, T3, T4, T5)>
where
    E0: Encode<Item = (T0, T1, T2, T3, T4, T5)>,
    E1: Encode,
{
    type Item = (T0, T1, T2, T3, T4, T5, E1::Item);

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        self.inner.encode(buf, eos)
    }

    fn start_encoding(&mut self, t: Self::Item) -> Result<()> {
        self.inner
            .start_encoding(((t.0, t.1, t.2, t.3, t.4, t.5), t.6))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.inner.requiring_bytes()
    }

    fn is_idle(&self) -> bool {
        self.inner.is_idle()
    }

    fn cancel(&mut self) -> Result<()> {
        track!(self.inner.cancel())
    }
}
impl<E0, E1, T0, T1, T2, T3, T4, T5, T6> Encode
    for EncoderChain<E0, E1, (T0, T1, T2, T3, T4, T5, T6)>
where
    E0: Encode<Item = (T0, T1, T2, T3, T4, T5, T6)>,
    E1: Encode,
{
    type Item = (T0, T1, T2, T3, T4, T5, T6, E1::Item);

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        self.inner.encode(buf, eos)
    }

    fn start_encoding(&mut self, t: Self::Item) -> Result<()> {
        self.inner
            .start_encoding(((t.0, t.1, t.2, t.3, t.4, t.5, t.6), t.7))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.inner.requiring_bytes()
    }

    fn is_idle(&self) -> bool {
        self.inner.is_idle()
    }

    fn cancel(&mut self) -> Result<()> {
        track!(self.inner.cancel())
    }
}
impl<E0, E1, T> ExactBytesEncode for EncoderChain<E0, E1, T>
where
    Self: Encode,
    E0: ExactBytesEncode,
    E1: ExactBytesEncode,
{
    fn exact_requiring_bytes(&self) -> u64 {
        self.inner.exact_requiring_bytes()
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
impl<A, B> Decode for Chain<Buffered<A>, B>
where
    A: Decode,
    B: Decode,
{
    type Item = (A::Item, B::Item);

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        let offset = if self.a.item.is_none() {
            let (size, _always_none) = track!(self.a.decode(buf, eos))?;
            if self.a.item.is_none() {
                return Ok((size, None));
            }
            size
        } else {
            0
        };

        let (size, item) = track!(self.b.decode(&buf[offset..], eos))?;
        let item = item.map(|b| {
            let a = self.a.item.take().expect("Never fails");
            (a, b)
        });
        Ok((offset + size, item))
    }

    fn has_terminated(&self) -> bool {
        self.a.has_terminated() || self.b.has_terminated()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.a
            .requiring_bytes()
            .add_for_decoding(self.b.requiring_bytes())
    }
}
impl<A, B> Encode for Chain<A, B>
where
    A: Encode,
    B: Encode,
{
    type Item = (A::Item, B::Item);

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        if !self.a.is_idle() {
            offset += track!(self.a.encode(buf, eos))?;
        }
        if self.a.is_idle() {
            offset += track!(self.b.encode(&mut buf[offset..], eos))?;
        }
        Ok(offset)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track!(self.a.start_encoding(item.0))?;
        track!(self.b.start_encoding(item.1))?;
        Ok(())
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.a
            .requiring_bytes()
            .add_for_decoding(self.b.requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.a.is_idle() && self.b.is_idle()
    }

    fn cancel(&mut self) -> Result<()> {
        track!(self.a.cancel())?;
        track!(self.b.cancel())?;
        Ok(())
    }
}
impl<E0, E1> ExactBytesEncode for Chain<E0, E1>
where
    E0: ExactBytesEncode,
    E1: ExactBytesEncode,
{
    fn exact_requiring_bytes(&self) -> u64 {
        self.a.exact_requiring_bytes() + self.b.exact_requiring_bytes()
    }
}

/// Combinator that gives a buffer to the decoder.
///
/// Thsi is created by calling `DecodeExt::buffer` method.
pub struct Buffered<D: Decode> {
    inner: D,
    item: Option<D::Item>,
}
impl<D: Decode> Buffered<D> {
    pub(crate) fn new(inner: D) -> Self {
        Buffered { inner, item: None }
    }

    /// Returns `true` if the decoder has a decoded item, other `false`.
    ///
    /// Note that the decoder cannot decode new items if this method returns `true`.
    pub fn has_item(&self) -> bool {
        self.item.is_some()
    }

    /// Returns a reference to the item decoded by the decoder in the last `decode` call.
    pub fn get_item(&self) -> Option<&D::Item> {
        self.item.as_ref()
    }

    /// Takes the item decoded by the decoder in the last `decode` call.
    pub fn take_item(&mut self) -> Option<D::Item> {
        self.item.take()
    }

    /// Returns a reference to the inner decoder.
    pub fn inner_ref(&self) -> &D {
        &self.inner
    }

    /// Returns a mutable reference to the inner decoder.
    pub fn inner_mut(&mut self) -> &mut D {
        &mut self.inner
    }
}
impl<D: Decode + fmt::Debug> fmt::Debug for Buffered<D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Buffered {{ inner: {:?}, item.is_some(): {:?} }}",
            self.inner,
            self.item.is_some()
        )
    }
}
impl<D: Decode + Default> Default for Buffered<D> {
    fn default() -> Self {
        Buffered {
            inner: D::default(),
            item: None,
        }
    }
}
impl<D: Decode> Decode for Buffered<D> {
    type Item = D::Item;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        if self.item.is_none() {
            let (size, item) = track!(self.inner.decode(buf, eos))?;
            self.item = item;
            Ok((size, None))
        } else {
            Ok((0, None))
        }
    }

    fn has_terminated(&self) -> bool {
        if self.item.is_some() {
            false
        } else {
            self.inner.has_terminated()
        }
    }

    fn requiring_bytes(&self) -> ByteCount {
        if self.item.is_some() {
            ByteCount::Finite(0)
        } else {
            self.inner.requiring_bytes()
        }
    }
}

#[cfg(test)]
mod test {
    use {Decode, DecodeExt, Encode, EncodeExt, Eos, StartDecoderChain, StartEncoderChain};
    use fixnum::{U8Decoder, U8Encoder};
    use io::IoDecodeExt;

    #[test]
    fn decoder_chain_works() {
        let mut decoder = StartDecoderChain
            .chain(U8Decoder::new())
            .chain(U8Decoder::new())
            .chain(U8Decoder::new());

        assert_eq!(
            track_try_unwrap!(decoder.decode_exact(b"foo".as_ref())),
            (b'f', b'o', b'o')
        );
    }

    #[test]
    fn encoder_chain_works() {
        let mut encoder = StartEncoderChain
            .chain(U8Encoder::new())
            .chain(U8Encoder::new())
            .chain(U8Encoder::new());
        track_try_unwrap!(encoder.start_encoding((1, 2, 3)));

        let mut output = [0; 3];
        track_try_unwrap!(encoder.encode(&mut output[..], Eos::new(true)));
        assert_eq!(output, [1, 2, 3]);
    }

    #[test]
    fn buffered_works() {
        let mut decoder = StartDecoderChain
            .chain(U8Decoder::new())
            .chain(U8Decoder::new())
            .chain(U8Decoder::new())
            .buffered();
        let (size, item) = track_try_unwrap!(decoder.decode(b"foo", Eos::new(false)));
        assert_eq!(size, 3);
        assert_eq!(item, None);
        assert_eq!(decoder.take_item(), Some((b'f', b'o', b'o')));
    }
}
