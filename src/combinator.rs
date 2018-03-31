use std;
use std::cmp;
use std::iter;
use std::marker::PhantomData;
use std::mem;

pub use chain::{DecoderChain, EncoderChain};

use {Decode, DecodeBuf, Encode, EncodeBuf, Error, ErrorKind, Result};

/// Combinator for converting decoded items to other values.
///
/// This is created by calling `DecodeExt::map` method.
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

/// Combinator for modifying encoding/decoding errors.
///
/// This is created by calling `{DecodeExt, EncodeExt}::map_err` method.
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

/// Combinator for conditional decoding.
///
/// If the first item is successfully decoded,
/// it will start decoding the second item by using the decoder returned by `f` function.
///
/// This is created by calling `DecodeExt::and_then` method.
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

/// Combinator for collecting decoded items.
///
/// This is created by calling `DecodeExt::collect` method.
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
            if let Some(item) = track!(self.decoder.decode(buf))? {
                self.items.extend(iter::once(item));
            } else {
                break;
            }
        }

        if buf.is_eos() || self.decoder.requiring_bytes_hint() == Some(0) {
            Ok(Some(mem::replace(&mut self.items, T::default())))
        } else {
            Ok(None)
        }
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        self.decoder.requiring_bytes_hint()
    }
}

/// Combinator for consuming the specified number of bytes exactly.
///
/// This is created by calling `DecodeExt::length` method.
#[derive(Debug)]
pub struct Length<D> {
    decoder: D,
    expected_bytes: u64,
    remaining_bytes: u64,
}
impl<D> Length<D> {
    pub(crate) fn new(decoder: D, expected_bytes: u64) -> Self {
        Length {
            decoder,
            expected_bytes,
            remaining_bytes: expected_bytes,
        }
    }

    /// Returns the number of bytes expected to be consumed for decoding an item.
    pub fn expected_bytes(&self) -> u64 {
        self.expected_bytes
    }

    /// Sets the number of bytes expected to be consumed for decoding an item.
    ///
    /// # Errors
    ///
    /// If the decoder is in the middle of decoding an item, it willl return an `ErrorKind::Other` error.
    pub fn set_expected_bytes(&mut self, bytes: u64) -> Result<()> {
        track_assert_eq!(
            self.remaining_bytes,
            self.expected_bytes,
            ErrorKind::Other,
            "An item is being decoded"
        );
        self.expected_bytes = bytes;
        Ok(())
    }

    /// Returns the number of remaining bytes required to decode the next item.
    pub fn remaining_bytes(&self) -> u64 {
        self.remaining_bytes
    }
}
impl<D: Decode> Decode for Length<D> {
    type Item = D::Item;

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        let buf_len = cmp::min(buf.len() as u64, self.remaining_bytes) as usize;
        let expected_remaining_bytes = self.remaining_bytes - buf_len as u64;
        if let Some(remaining_bytes) = buf.remaining_bytes() {
            track_assert!(remaining_bytes >= expected_remaining_bytes, ErrorKind::UnexpectedEos;
                          remaining_bytes, expected_remaining_bytes);
        }
        let (item, consumed_len) = {
            let mut actual_buf =
                DecodeBuf::with_remaining_bytes(&buf[..buf_len], expected_remaining_bytes);
            let item = track!(self.decoder.decode(&mut actual_buf))?;
            let consumed_len = buf_len - actual_buf.len();
            (item, consumed_len)
        };

        self.remaining_bytes -= consumed_len as u64;
        track!(buf.consume(consumed_len))?;
        if item.is_some() {
            track_assert_eq!(
                self.remaining_bytes,
                0,
                ErrorKind::Other,
                "Decoder consumes too few bytes"
            );
            self.remaining_bytes = self.expected_bytes
        }
        Ok(item)
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        Some(self.remaining_bytes)
    }
}

#[derive(Debug)]
pub struct Take<D> {
    decoder: D,
    remaining_items: usize,
}
impl<D> Take<D> {
    pub(crate) fn new(decoder: D, remaining_items: usize) -> Self {
        Take {
            decoder,
            remaining_items,
        }
    }

    pub fn remaining_items(&self) -> usize {
        self.remaining_items
    }

    pub fn set_remaining_items(&mut self, n: usize) {
        self.remaining_items = n;
    }
}
impl<D: Decode> Decode for Take<D> {
    type Item = D::Item;

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        track_assert_ne!(self.remaining_items, 0, ErrorKind::DecoderTerminated);
        if let Some(item) = track!(self.decoder.decode(buf))? {
            self.remaining_items -= 1;
            Ok(Some(item))
        } else {
            Ok(None)
        }
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        if self.remaining_items == 0 {
            Some(0)
        } else {
            self.decoder.requiring_bytes_hint()
        }
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

#[cfg(test)]
mod test {
    use {Decode, DecodeBuf, DecodeExt, ErrorKind};
    use bytes::Utf8Decoder;
    use fixnum::U8Decoder;

    #[test]
    fn collect_works() {
        let mut decoder = U8Decoder::new().collect::<Vec<_>>();
        let mut input = DecodeBuf::with_remaining_bytes(b"foo", 0);

        let item = track_try_unwrap!(decoder.decode(&mut input));
        assert_eq!(item, Some(vec![b'f', b'o', b'o']));
    }

    #[test]
    fn take_works() {
        let mut decoder = U8Decoder::new().take(2).collect::<Vec<_>>();
        let mut input = DecodeBuf::new(b"foo");

        let item = track_try_unwrap!(decoder.decode(&mut input));
        assert_eq!(item, Some(vec![b'f', b'o']));
    }

    #[test]
    fn length_works() {
        let mut decoder = Utf8Decoder::new().length(3);
        let mut input = DecodeBuf::with_remaining_bytes(b"foobarba", 0);

        let item = track_try_unwrap!(decoder.decode(&mut input));
        assert_eq!(item, Some("foo".to_owned()));

        let item = track_try_unwrap!(decoder.decode(&mut input));
        assert_eq!(item, Some("bar".to_owned()));

        let error = decoder.decode(&mut input).err().unwrap();
        assert_eq!(*error.kind(), ErrorKind::UnexpectedEos);
    }
}
