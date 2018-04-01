//! Encoders and decoders for combination.
//!
//! These are mainly created via the methods provided by `EncodeExt` or `DecodeExt` traits.
use std;
use std::cmp;
use std::io::Write;
use std::iter;
use std::marker::PhantomData;

pub use chain::{DecoderChain, EncoderChain};

use {Decode, DecodeBuf, Encode, EncodeBuf, Error, ErrorKind, ExactBytesEncode, Result};

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

    fn is_idle(&self) -> bool {
        self.decoder.is_idle()
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

    fn is_idle(&self) -> bool {
        self.codec.is_idle()
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

    fn requiring_bytes_hint(&self) -> Option<u64> {
        self.codec.requiring_bytes_hint()
    }

    fn is_idle(&self) -> bool {
        self.codec.is_idle()
    }
}
impl<C, F, E> ExactBytesEncode for MapErr<C, F, E>
where
    C: ExactBytesEncode,
    F: Fn(Error) -> E,
    Error: From<E>,
{
    fn requiring_bytes(&self) -> u64 {
        self.codec.requiring_bytes()
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

    fn is_idle(&self) -> bool {
        self.decoder1.is_none() && self.decoder0.is_idle()
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        if let Some(ref d) = self.decoder1 {
            d.requiring_bytes_hint()
        } else {
            self.decoder0.requiring_bytes_hint()
        }
    }
}

/// Combinator for converting items into ones that
/// suited to the inner encoder by calling the given function.
///
/// This is created by calling `EncodeExt::map_from` method.
#[derive(Debug)]
pub struct MapFrom<E, T, F> {
    encoder: E,
    _item: PhantomData<T>,
    from: F,
}
impl<E, T, F> MapFrom<E, T, F> {
    pub(crate) fn new(encoder: E, from: F) -> Self {
        MapFrom {
            encoder,
            _item: PhantomData,
            from,
        }
    }
}
impl<E, T, F> Encode for MapFrom<E, T, F>
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

    fn requiring_bytes_hint(&self) -> Option<u64> {
        self.encoder.requiring_bytes_hint()
    }

    fn is_idle(&self) -> bool {
        self.encoder.is_idle()
    }
}
impl<E, T, F> ExactBytesEncode for MapFrom<E, T, F>
where
    E: ExactBytesEncode,
    F: Fn(T) -> E::Item,
{
    fn requiring_bytes(&self) -> u64 {
        self.encoder.requiring_bytes()
    }
}

/// Combinator that tries to convert items into ones that
/// suited to the inner encoder by calling the given function.
///
/// This is created by calling `EncodeExt::try_map_from` method.
#[derive(Debug)]
pub struct TryMapFrom<C, T, E, F> {
    encoder: C,
    try_from: F,
    _phantom: PhantomData<(T, E)>,
}
impl<C, T, E, F> TryMapFrom<C, T, E, F> {
    pub(crate) fn new(encoder: C, try_from: F) -> Self {
        TryMapFrom {
            encoder,
            try_from,
            _phantom: PhantomData,
        }
    }
}
impl<C, T, E, F> Encode for TryMapFrom<C, T, E, F>
where
    C: Encode,
    F: Fn(T) -> std::result::Result<C::Item, E>,
    Error: From<E>,
{
    type Item = T;

    fn encode(&mut self, buf: &mut EncodeBuf) -> Result<()> {
        track!(self.encoder.encode(buf))
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        let item = track!((self.try_from)(item).map_err(Error::from))?;
        track!(self.encoder.start_encoding(item))
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        self.encoder.requiring_bytes_hint()
    }

    fn is_idle(&self) -> bool {
        self.encoder.is_idle()
    }
}
impl<C, T, E, F> ExactBytesEncode for TryMapFrom<C, T, E, F>
where
    C: ExactBytesEncode,
    F: Fn(T) -> std::result::Result<C::Item, E>,
    Error: From<E>,
{
    fn requiring_bytes(&self) -> u64 {
        self.encoder.requiring_bytes()
    }
}

/// Combinator for repeating encoding of `E::Item`.
///
/// This is created by calling `EncodeExt::repeat` method.
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
            track!(self.encoder.encode(buf))?;
            if self.encoder.is_idle() {
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
        track_assert!(self.is_idle(), ErrorKind::EncoderFull);
        self.items = Some(item);
        Ok(())
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        if self.is_idle() {
            Some(0)
        } else {
            None
        }
    }

    fn is_idle(&self) -> bool {
        self.items.is_none()
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

    fn is_idle(&self) -> bool {
        self.0.as_ref().map_or(true, |d| d.is_idle())
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        if let Some(ref d) = self.0 {
            d.requiring_bytes_hint()
        } else {
            Some(0)
        }
    }
}

/// Combinator for representing an optional encoder.
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

    fn requiring_bytes_hint(&self) -> Option<u64> {
        self.0.requiring_bytes_hint()
    }

    fn is_idle(&self) -> bool {
        self.0.is_idle()
    }
}
impl<E: ExactBytesEncode> ExactBytesEncode for Optional<E> {
    fn requiring_bytes(&self) -> u64 {
        self.0.requiring_bytes()
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
impl<D, T> Collect<D, T> {
    pub(crate) fn new(decoder: D) -> Self {
        Collect {
            decoder,
            items: None,
        }
    }
}
impl<D, T: Default> Decode for Collect<D, T>
where
    D: Decode,
    T: Extend<D::Item>,
{
    type Item = T;

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        if self.items.is_none() {
            self.items = Some(T::default());
        }
        {
            let items = self.items.as_mut().expect("Never fails");
            while !(buf.is_empty() && buf.is_eos()) && !self.decoder.has_terminated() {
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
        self.decoder.has_terminated()
    }

    fn is_idle(&self) -> bool {
        self.items.is_none()
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        self.decoder.requiring_bytes_hint()
    }
}

/// Combinator for consuming the specified number of bytes exactly.
///
/// This is created by calling `{DecodeExt, EncodeExt}::length` method.
#[derive(Debug)]
pub struct Length<C> {
    codec: C,
    expected_bytes: u64,
    remaining_bytes: u64,
}
impl<C> Length<C> {
    pub(crate) fn new(codec: C, expected_bytes: u64) -> Self {
        Length {
            codec,
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
    /// If the codec is in the middle of decoding an item, it willl return an `ErrorKind::Other` error.
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
        let old_buf_len = buf.len();
        let buf_len = cmp::min(buf.len() as u64, self.remaining_bytes) as usize;
        let expected_remaining_bytes = self.remaining_bytes - buf_len as u64;
        if let Some(remaining_bytes) = buf.remaining_bytes() {
            track_assert!(remaining_bytes >= expected_remaining_bytes, ErrorKind::UnexpectedEos;
                          remaining_bytes, expected_remaining_bytes);
        }
        let item = buf.with_limit_and_remaining_bytes(buf_len, expected_remaining_bytes, |buf| {
            track!(self.codec.decode(buf))
        })?;

        self.remaining_bytes -= (old_buf_len - buf.len()) as u64;
        if item.is_some() {
            track_assert_eq!(
                self.remaining_bytes,
                0,
                ErrorKind::Other,
                "Codec consumes too few bytes"
            );
            self.remaining_bytes = self.expected_bytes
        }
        Ok(item)
    }

    fn has_terminated(&self) -> bool {
        if self.remaining_bytes == self.expected_bytes {
            self.codec.has_terminated()
        } else {
            false
        }
    }

    fn is_idle(&self) -> bool {
        self.remaining_bytes == self.expected_bytes
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        if self.has_terminated() {
            Some(0)
        } else {
            Some(self.remaining_bytes)
        }
    }
}
impl<E: Encode> Encode for Length<E> {
    type Item = E::Item;

    fn encode(&mut self, buf: &mut EncodeBuf) -> Result<()> {
        if buf.is_eos() {
            track_assert!(buf.len() as u64 >= self.remaining_bytes, ErrorKind::UnexpectedEos;
                          buf.len(), self.remaining_bytes);
        }

        let original_buf_len = buf.len();
        let limit = cmp::min(buf.len() as u64, self.remaining_bytes) as usize;
        let eos = limit as u64 == self.remaining_bytes;
        if eos {
            buf.with_limit_and_eos(limit, |buf| track!(self.codec.encode(buf)))?;
        } else {
            buf.with_limit(limit, |buf| track!(self.codec.encode(buf)))?;
        }

        self.remaining_bytes -= (original_buf_len - buf.len()) as u64;
        if self.codec.is_idle() {
            track_assert_eq!(
                self.remaining_bytes,
                0,
                ErrorKind::InvalidInput,
                "Too small item"
            );
            self.remaining_bytes = self.expected_bytes;
        }
        Ok(())
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track_assert_eq!(
            self.remaining_bytes,
            self.expected_bytes,
            ErrorKind::EncoderFull
        );
        track!(self.codec.start_encoding(item))
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        Some(self.remaining_bytes)
    }

    fn is_idle(&self) -> bool {
        self.remaining_bytes == self.expected_bytes
    }
}
impl<E: Encode> ExactBytesEncode for Length<E> {
    fn requiring_bytes(&self) -> u64 {
        self.remaining_bytes
    }
}

/// Combinator for decoding the specified number of items.
///
/// This is created by calling `DecodeExt::take` method.
#[derive(Debug)]
pub struct Take<D> {
    decoder: D,
    limit: usize,
    decoded_items: usize,
}
impl<D> Take<D> {
    pub(crate) fn new(decoder: D, count: usize) -> Self {
        Take {
            decoder,
            limit: count,
            decoded_items: 0,
        }
    }
}
impl<D: Decode> Decode for Take<D> {
    type Item = D::Item;

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        track_assert_ne!(self.decoded_items, self.limit, ErrorKind::DecoderTerminated);
        if let Some(item) = track!(self.decoder.decode(buf))? {
            self.decoded_items += 1;
            Ok(Some(item))
        } else {
            Ok(None)
        }
    }

    fn has_terminated(&self) -> bool {
        self.decoder.has_terminated() || self.decoded_items == self.limit
    }

    fn is_idle(&self) -> bool {
        self.decoded_items == 0 || self.decoded_items == self.limit
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

    fn is_idle(&self) -> bool {
        self.decoder.is_idle()
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

    fn is_idle(&self) -> bool {
        self.item.is_none() && self.decoder.is_idle()
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
///
/// This is created by calling `{DecodeExt, EncodeExt}::max_bytes` method.
#[derive(Debug)]
pub struct MaxBytes<C> {
    codec: C,
    consumed_bytes: u64,
    max_bytes: u64,
}
impl<C> MaxBytes<C> {
    pub(crate) fn new(codec: C, max_bytes: u64) -> Self {
        MaxBytes {
            codec,
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
        let old_buf_len = buf.len();
        let actual_buf_len = cmp::min(buf.len() as u64, self.max_remaining_bytes()) as usize;
        let item = buf.with_limit(actual_buf_len, |buf| track!(self.codec.decode(buf)))?;
        self.consumed_bytes = (old_buf_len - buf.len()) as u64;
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
        self.codec.has_terminated()
    }

    fn is_idle(&self) -> bool {
        self.codec.is_idle()
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        self.codec.requiring_bytes_hint()
    }
}
impl<E: Encode> Encode for MaxBytes<E> {
    type Item = E::Item;

    fn encode(&mut self, buf: &mut EncodeBuf) -> Result<()> {
        let old_buf_len = buf.len();
        let actual_buf_len = cmp::min(buf.len() as u64, self.max_remaining_bytes()) as usize;
        buf.with_limit(actual_buf_len, |buf| track!(self.codec.encode(buf)))?;
        self.consumed_bytes = (old_buf_len - buf.len()) as u64;
        if self.consumed_bytes == self.max_bytes {
            track_assert!(self.is_idle(), ErrorKind::InvalidInput, "Max bytes limit exceeded";
                          self.max_bytes);
        }
        if self.is_idle() {
            self.consumed_bytes = 0;
        }
        Ok(())
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track!(self.codec.start_encoding(item))
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        self.codec.requiring_bytes_hint()
    }

    fn is_idle(&self) -> bool {
        self.codec.is_idle()
    }
}
impl<E: ExactBytesEncode> ExactBytesEncode for MaxBytes<E> {
    fn requiring_bytes(&self) -> u64 {
        self.codec.requiring_bytes()
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

    fn is_idle(&self) -> bool {
        self.decoder.is_idle()
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        self.decoder.requiring_bytes_hint()
    }
}

/// Combinator that keeps writing padding byte until it reaches EOS
/// after encoding of `E`'s item has been completed.
///
/// This is created by calling `EncodeExt::padding` method.
#[derive(Debug)]
pub struct Padding<E> {
    encoder: E,
    padding_byte: u8,
    eos_reached: bool,
}
impl<E> Padding<E> {
    pub(crate) fn new(encoder: E, padding_byte: u8) -> Self {
        Padding {
            encoder,
            padding_byte,
            eos_reached: true,
        }
    }
}
impl<E: Encode> Encode for Padding<E> {
    type Item = E::Item;

    fn encode(&mut self, buf: &mut EncodeBuf) -> Result<()> {
        if !self.encoder.is_idle() {
            self.encoder.encode(buf)?
        }
        while 0 != buf.write(&[self.padding_byte][..]).expect("Never fails") {}
        self.eos_reached = buf.is_eos();
        Ok(())
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track_assert!(self.is_idle(), ErrorKind::EncoderFull);
        self.eos_reached = false;
        track!(self.encoder.start_encoding(item))
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        None
    }

    fn is_idle(&self) -> bool {
        self.eos_reached
    }
}

#[cfg(test)]
mod test {
    use {Decode, DecodeBuf, DecodeExt, Encode, EncodeBuf, EncodeExt, ErrorKind};
    use bytes::{Utf8Decoder, Utf8Encoder};
    use fixnum::{U8Decoder, U8Encoder};

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
    fn decoder_length_works() {
        let mut decoder = Utf8Decoder::new().length(3);
        let mut input = DecodeBuf::with_remaining_bytes(b"foobarba", 0);

        let item = track_try_unwrap!(decoder.decode(&mut input));
        assert_eq!(item, Some("foo".to_owned()));

        let item = track_try_unwrap!(decoder.decode(&mut input));
        assert_eq!(item, Some("bar".to_owned()));

        let error = decoder.decode(&mut input).err().unwrap();
        assert_eq!(*error.kind(), ErrorKind::UnexpectedEos);
    }

    #[test]
    fn encoder_length_works() {
        let mut output = [0; 4];
        {
            let mut encoder = Utf8Encoder::new().length(3);
            encoder.start_encoding("hey").unwrap(); // OK
            encoder.encode(&mut EncodeBuf::new(&mut output)).unwrap();
        }
        assert_eq!(output.as_ref(), b"hey\x00");

        {
            let mut encoder = Utf8Encoder::new().length(3);
            encoder.start_encoding("hello").unwrap(); // Error (too long)
            let error = encoder
                .encode(&mut EncodeBuf::new(&mut output))
                .err()
                .unwrap();
            assert_eq!(*error.kind(), ErrorKind::UnexpectedEos);
        }

        {
            let mut encoder = Utf8Encoder::new().length(3);
            encoder.start_encoding("hi").unwrap(); // Error (too short)
            let error = encoder.encode(&mut EncodeBuf::new(&mut output)).err();
            assert_eq!(error.map(|e| *e.kind()), Some(ErrorKind::InvalidInput));
        }
    }

    #[test]
    fn padding_works() {
        let mut output = [0; 4];
        {
            let mut encoder = U8Encoder::new().padding(9).length(3);
            encoder.start_encoding(3).unwrap();
            encoder.encode(&mut EncodeBuf::new(&mut output)).unwrap();
        }
        assert_eq!(output.as_ref(), [3, 9, 9, 0]);
    }

    #[test]
    fn repeat_works() {
        let mut output = [0; 4];
        {
            let mut encoder = U8Encoder::new().repeat();
            encoder.start_encoding(0..4).unwrap();
            encoder.encode(&mut EncodeBuf::new(&mut output)).unwrap();
        }
        assert_eq!(output.as_ref(), [0, 1, 2, 3]);
    }
}
