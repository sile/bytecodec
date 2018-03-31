use std;
use std::cmp;
use std::iter;
use std::marker::PhantomData;

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

    fn has_terminated(&self) -> bool {
        self.decoder.has_terminated()
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        self.decoder.requiring_bytes_hint()
    }
}

/// Combinator for modifying encoding/decoding errors.
///
/// This is created by calling `{DecodeExt, EncodeExt}::map_err` method.
#[derive(Debug)]
pub struct MapErr<C, F, E> {
    codec: C,
    map_err: F,
    _error: PhantomData<E>,
}
impl<C, F, E> MapErr<C, F, E> {
    pub(crate) fn new(codec: C, map_err: F) -> Self
    where
        F: Fn(Error) -> E,
        Error: From<E>,
    {
        MapErr {
            codec,
            map_err,
            _error: PhantomData,
        }
    }
}
impl<D, F, E> Decode for MapErr<D, F, E>
where
    D: Decode,
    F: Fn(Error) -> E,
    Error: From<E>,
{
    type Item = D::Item;

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        self.codec.decode(buf).map_err(|e| (self.map_err)(e).into())
    }

    fn has_terminated(&self) -> bool {
        self.codec.has_terminated()
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        self.codec.requiring_bytes_hint()
    }
}
impl<C, F, E> Encode for MapErr<C, F, E>
where
    C: Encode,
    F: Fn(Error) -> E,
    Error: From<E>,
{
    type Item = C::Item;

    fn encode(&mut self, buf: &mut EncodeBuf) -> Result<()> {
        self.codec.encode(buf).map_err(|e| (self.map_err)(e).into())
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        self.codec
            .start_encoding(item)
            .map_err(|e| (self.map_err)(e).into())
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

    fn has_terminated(&self) -> bool {
        if let Some(ref d) = self.decoder1 {
            d.has_terminated()
        } else {
            self.decoder0.has_terminated()
        }
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

/// Combinator for representing optional decoders.
///
/// This is created by calling `DecodeExt::omit` method.
#[derive(Debug)]
pub struct Omit<D>(Option<D>);
impl<D> Omit<D> {
    pub(crate) fn new(decoder: D, do_omit: bool) -> Self {
        if do_omit {
            Omit(None)
        } else {
            Omit(Some(decoder))
        }
    }
}
impl<D: Decode> Decode for Omit<D> {
    type Item = Option<D::Item>;

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        if let Some(ref mut d) = self.0 {
            if let Some(item) = track!(d.decode(buf))? {
                Ok(Some(Some(item)))
            } else {
                Ok(None)
            }
        } else {
            Ok(Some(None))
        }
    }

    fn has_terminated(&self) -> bool {
        if let Some(ref d) = self.0 {
            d.has_terminated()
        } else {
            false
        }
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        if let Some(ref d) = self.0 {
            d.requiring_bytes_hint()
        } else {
            Some(0)
        }
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
///
/// Note that this is a oneshot decoder (i.e., it decodes only one item).
#[derive(Debug)]
pub struct Collect<D, T> {
    decoder: D,
    items: Option<T>,
}
impl<D, T: Default> Collect<D, T> {
    pub(crate) fn new(decoder: D) -> Self {
        Collect {
            decoder,
            items: Some(T::default()),
        }
    }
}
impl<D, T> Decode for Collect<D, T>
where
    D: Decode,
    T: Extend<D::Item>,
{
    type Item = T;

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        {
            let items = track_assert_some!(self.items.as_mut(), ErrorKind::DecoderTerminated);
            while !buf.is_eos() && !self.decoder.has_terminated() {
                if let Some(item) = track!(self.decoder.decode(buf))? {
                    items.extend(iter::once(item));
                } else {
                    return Ok(None);
                }
            }
        }
        Ok(self.items.take())
    }

    fn has_terminated(&self) -> bool {
        self.items.is_none() || self.decoder.has_terminated()
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        if self.items.is_none() {
            Some(0)
        } else {
            self.decoder.requiring_bytes_hint()
        }
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

    fn has_terminated(&self) -> bool {
        if self.remaining_bytes == self.expected_bytes {
            self.decoder.has_terminated()
        } else {
            false
        }
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        if self.has_terminated() {
            Some(0)
        } else {
            Some(self.remaining_bytes)
        }
    }
}

/// Combinator for decoding the specified number of items.
///
/// This is created by calling `DecodeExt::takeExact` method.
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

    /// Returns the number of remaining items to be decoded.
    pub fn remaining_items(&self) -> usize {
        self.remaining_items
    }

    /// Sets the number of remaining items to be decoded.
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

    fn has_terminated(&self) -> bool {
        self.remaining_items == 0
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        if self.has_terminated() {
            Some(0)
        } else {
            self.decoder.requiring_bytes_hint()
        }
    }
}

/// Combinator which tries to convert decoded values by calling the specified function.
///
/// This is created by calling `DecodeExt::try_map` method.
#[derive(Debug)]
pub struct TryMap<D, F, T, E> {
    decoder: D,
    try_map: F,
    _phantom: PhantomData<(T, E)>,
}
impl<D, F, T, E> TryMap<D, F, T, E> {
    pub(crate) fn new(decoder: D, try_map: F) -> Self {
        TryMap {
            decoder,
            try_map,
            _phantom: PhantomData,
        }
    }
}
impl<D, F, T, E> Decode for TryMap<D, F, T, E>
where
    D: Decode,
    F: Fn(D::Item) -> std::result::Result<T, E>,
    Error: From<E>,
{
    type Item = T;

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        if let Some(item) = track!(self.decoder.decode(buf))? {
            let item = track!((self.try_map)(item).map_err(Error::from))?;
            Ok(Some(item))
        } else {
            Ok(None)
        }
    }

    fn has_terminated(&self) -> bool {
        self.decoder.has_terminated()
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        self.decoder.requiring_bytes_hint()
    }
}

/// Combinator for skipping the remaining bytes in an input byte sequence
/// after decoding an item by using `D`.
#[derive(Debug)]
pub struct SkipRemaining<D: Decode> {
    decoder: D,
    item: Option<D::Item>,
}
impl<D: Decode> SkipRemaining<D> {
    pub(crate) fn new(decoder: D) -> Self {
        SkipRemaining {
            decoder,
            item: None,
        }
    }
}
impl<D: Decode> Decode for SkipRemaining<D> {
    type Item = D::Item;

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        track_assert!(
            buf.remaining_bytes().is_some(),
            ErrorKind::InvalidInput,
            "Cannot skip infinity byte stream"
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

    fn has_terminated(&self) -> bool {
        if self.item.is_none() {
            self.decoder.has_terminated()
        } else {
            false
        }
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        if self.item.is_none() {
            self.decoder.requiring_bytes_hint()
        } else {
            None
        }
    }
}

/// Combinator that will fail if the number of consumed bytes exceeds the specified size.
#[derive(Debug)]
pub struct MaxBytes<D> {
    decoder: D,
    consumed_bytes: u64,
    max_bytes: u64,
}
impl<D> MaxBytes<D> {
    pub(crate) fn new(decoder: D, max_bytes: u64) -> Self {
        MaxBytes {
            decoder,
            consumed_bytes: 0,
            max_bytes,
        }
    }

    fn max_remaining_bytes(&self) -> u64 {
        self.max_bytes - self.consumed_bytes
    }
}
impl<D: Decode> Decode for MaxBytes<D> {
    type Item = D::Item;

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        let actual_buf_len = cmp::min(buf.len() as u64, self.max_remaining_bytes()) as usize;
        let (item, consumed_len) = {
            let mut actual_buf = if let Some(remaining_bytes) = buf.remaining_bytes() {
                let actual_remaining_bytes = remaining_bytes + (buf.len() - actual_buf_len) as u64;
                DecodeBuf::with_remaining_bytes(&buf[..actual_buf_len], actual_remaining_bytes)
            } else {
                DecodeBuf::new(&buf[..actual_buf_len])
            };
            let item = track!(self.decoder.decode(&mut actual_buf))?;
            let consumed_len = actual_buf_len - actual_buf.len();
            (item, consumed_len)
        };

        self.consumed_bytes += consumed_len as u64;
        track!(buf.consume(consumed_len))?;
        if self.consumed_bytes == self.max_bytes {
            track_assert!(item.is_some(), ErrorKind::InvalidInput, "Max bytes limit exceeded";
                          self.max_bytes);
        }
        if item.is_some() {
            self.consumed_bytes = 0;
        }
        Ok(item)
    }

    fn has_terminated(&self) -> bool {
        self.decoder.has_terminated()
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        self.decoder.requiring_bytes_hint()
    }
}

/// Combinator for declaring an assertion about decoded items.
///
/// This created by calling `DecodeExt::assert` method.
#[derive(Debug)]
pub struct Assert<D, F> {
    decoder: D,
    assert: F,
}
impl<D, F> Assert<D, F> {
    pub(crate) fn new(decoder: D, assert: F) -> Self {
        Assert { decoder, assert }
    }
}
impl<D: Decode, F> Decode for Assert<D, F>
where
    F: for<'a> Fn(&'a D::Item) -> bool,
{
    type Item = D::Item;

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        if let Some(item) = track!(self.decoder.decode(buf))? {
            track_assert!((self.assert)(&item), ErrorKind::InvalidInput);
            Ok(Some(item))
        } else {
            Ok(None)
        }
    }

    fn has_terminated(&self) -> bool {
        self.decoder.has_terminated()
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        self.decoder.requiring_bytes_hint()
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
