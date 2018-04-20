//! Encoders and decoders for combination.
//!
//! These are mainly created via the methods provided by `EncodeExt` or `DecodeExt` traits.
use std;
use std::cmp;
use std::marker::PhantomData;

pub use chain::{Buffered, DecoderChain, EncoderChain};

use {ByteCount, Decode, Encode, Eos, Error, ErrorKind, ExactBytesEncode, Result};
use bytes::BytesEncoder;
use io::IoEncodeExt;
use marker::Never;

/// Combinator for converting decoded items to other values.
///
/// This is created by calling `DecodeExt::map` method.
#[derive(Debug)]
pub struct Map<D, T, F> {
    inner: D,
    map: F,
    _item: PhantomData<T>,
}
impl<D: Decode, T, F> Map<D, T, F> {
    pub(crate) fn new(inner: D, map: F) -> Self
    where
        F: Fn(D::Item) -> T,
    {
        Map {
            inner,
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

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        track!(self.inner.decode(buf, eos)).map(|(n, r)| (n, r.map(&self.map)))
    }

    fn has_terminated(&self) -> bool {
        self.inner.has_terminated()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.inner.requiring_bytes()
    }
}

/// Combinator for modifying encoding/decoding errors.
///
/// This is created by calling `{DecodeExt, EncodeExt}::map_err` method.
#[derive(Debug)]
pub struct MapErr<C, F, E> {
    inner: C,
    map_err: F,
    _error: PhantomData<E>,
}
impl<C, F, E> MapErr<C, F, E> {
    pub(crate) fn new(inner: C, map_err: F) -> Self
    where
        F: Fn(Error) -> E,
        Error: From<E>,
    {
        MapErr {
            inner,
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

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        self.inner
            .decode(buf, eos)
            .map_err(|e| (self.map_err)(e).into())
    }

    fn has_terminated(&self) -> bool {
        self.inner.has_terminated()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.inner.requiring_bytes()
    }
}
impl<C, F, E> Encode for MapErr<C, F, E>
where
    C: Encode,
    F: Fn(Error) -> E,
    Error: From<E>,
{
    type Item = C::Item;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        self.inner
            .encode(buf, eos)
            .map_err(|e| (self.map_err)(e).into())
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        self.inner
            .start_encoding(item)
            .map_err(|e| (self.map_err)(e).into())
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.inner.requiring_bytes()
    }

    fn is_idle(&self) -> bool {
        self.inner.is_idle()
    }
}
impl<C, F, E> ExactBytesEncode for MapErr<C, F, E>
where
    C: ExactBytesEncode,
    F: Fn(Error) -> E,
    Error: From<E>,
{
    fn exact_requiring_bytes(&self) -> u64 {
        self.inner.exact_requiring_bytes()
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
    inner0: D0,
    inner1: Option<D1>,
    and_then: F,
}
impl<D0: Decode, D1, F> AndThen<D0, D1, F> {
    pub(crate) fn new(inner0: D0, and_then: F) -> Self
    where
        F: Fn(D0::Item) -> D1,
    {
        AndThen {
            inner0,
            inner1: None,
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

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        let offset = if self.inner1.is_none() {
            let (size, item) = track!(self.inner0.decode(buf, eos))?;
            if let Some(d) = item.map(&self.and_then) {
                self.inner1 = Some(d);
            }
            size
        } else {
            0
        };
        if let Some(result) = self.inner1
            .as_mut()
            .map(|d| track!(d.decode(&buf[offset..], eos)))
        {
            let (size, item) = result?;
            if item.is_some() {
                self.inner1 = None;
            }
            Ok((offset + size, item))
        } else {
            Ok((offset, None))
        }
    }

    fn has_terminated(&self) -> bool {
        if let Some(ref d) = self.inner1 {
            d.has_terminated()
        } else {
            self.inner0.has_terminated()
        }
    }

    fn requiring_bytes(&self) -> ByteCount {
        if let Some(ref d) = self.inner1 {
            d.requiring_bytes()
        } else {
            self.inner0.requiring_bytes()
        }
    }
}

/// Combinator for converting items into ones that
/// suited to the inner encoder by calling the given function.
///
/// This is created by calling `EncodeExt::map_from` method.
#[derive(Debug)]
pub struct MapFrom<E, T, F> {
    inner: E,
    _item: PhantomData<T>,
    from: F,
}
impl<E, T, F> MapFrom<E, T, F> {
    pub(crate) fn new(inner: E, from: F) -> Self {
        MapFrom {
            inner,
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

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        track!(self.inner.encode(buf, eos))
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track!(self.inner.start_encoding((self.from)(item)))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.inner.requiring_bytes()
    }

    fn is_idle(&self) -> bool {
        self.inner.is_idle()
    }
}
impl<E, T, F> ExactBytesEncode for MapFrom<E, T, F>
where
    E: ExactBytesEncode,
    F: Fn(T) -> E::Item,
{
    fn exact_requiring_bytes(&self) -> u64 {
        self.inner.exact_requiring_bytes()
    }
}

/// Combinator that tries to convert items into ones that
/// suited to the inner encoder by calling the given function.
///
/// This is created by calling `EncodeExt::try_map_from` method.
#[derive(Debug)]
pub struct TryMapFrom<C, T, E, F> {
    inner: C,
    try_from: F,
    _phantom: PhantomData<(T, E)>,
}
impl<C, T, E, F> TryMapFrom<C, T, E, F> {
    pub(crate) fn new(inner: C, try_from: F) -> Self {
        TryMapFrom {
            inner,
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

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        track!(self.inner.encode(buf, eos))
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        let item = track!((self.try_from)(item).map_err(Error::from))?;
        track!(self.inner.start_encoding(item))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.inner.requiring_bytes()
    }

    fn is_idle(&self) -> bool {
        self.inner.is_idle()
    }
}
impl<C, T, E, F> ExactBytesEncode for TryMapFrom<C, T, E, F>
where
    C: ExactBytesEncode,
    F: Fn(T) -> std::result::Result<C::Item, E>,
    Error: From<E>,
{
    fn exact_requiring_bytes(&self) -> u64 {
        self.inner.exact_requiring_bytes()
    }
}

/// Combinator for repeating encoding of `E::Item`.
///
/// This is created by calling `EncodeExt::repeat` method.
#[derive(Debug, Default)]
pub struct Repeat<E, I> {
    inner: E,
    items: Option<I>,
}
impl<E, I> Repeat<E, I> {
    pub(crate) fn new(inner: E) -> Self {
        Repeat { inner, items: None }
    }
}
impl<E, I> Encode for Repeat<E, I>
where
    E: Encode,
    I: Iterator<Item = E::Item>,
{
    type Item = I;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        loop {
            while self.inner.is_idle() {
                if let Some(item) = self.items.as_mut().and_then(|iter| iter.next()) {
                    track!(self.inner.start_encoding(item))?;
                } else {
                    self.items = None;
                    return Ok(offset);
                }
            }

            let size = track!(self.inner.encode(&mut buf[offset..], eos))?;
            offset += size;
            if size == 0 {
                return Ok(offset);
            }
        }
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track_assert!(self.is_idle(), ErrorKind::EncoderFull);
        self.items = Some(item);
        Ok(())
    }

    fn requiring_bytes(&self) -> ByteCount {
        if self.is_idle() {
            ByteCount::Finite(0)
        } else {
            ByteCount::Unknown
        }
    }

    fn is_idle(&self) -> bool {
        self.items.is_none()
    }
}

/// Combinator for representing optional decoders.
///
/// This is created by calling `DecodeExt::omit` method.
#[derive(Debug, Default)]
pub struct Omit<D>(Option<D>);
impl<D> Omit<D> {
    pub(crate) fn new(inner: D, do_omit: bool) -> Self {
        if do_omit {
            Omit(None)
        } else {
            Omit(Some(inner))
        }
    }
}
impl<D: Decode> Decode for Omit<D> {
    type Item = Option<D::Item>;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        if let Some(ref mut d) = self.0 {
            match track!(d.decode(buf, eos))? {
                (size, Some(item)) => Ok((size, Some(Some(item)))),
                (size, None) => Ok((size, None)),
            }
        } else {
            Ok((0, Some(None)))
        }
    }

    fn has_terminated(&self) -> bool {
        if let Some(ref d) = self.0 {
            d.has_terminated()
        } else {
            false
        }
    }

    fn requiring_bytes(&self) -> ByteCount {
        if let Some(ref d) = self.0 {
            d.requiring_bytes()
        } else {
            ByteCount::Finite(0)
        }
    }
}

/// Combinator for representing an optional encoder.
#[derive(Debug, Default)]
pub struct Optional<E>(E);
impl<E> Optional<E> {
    pub(crate) fn new(inner: E) -> Self {
        Optional(inner)
    }
}
impl<E: Encode> Encode for Optional<E> {
    type Item = Option<E::Item>;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        track!(self.0.encode(buf, eos))
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        if let Some(item) = item {
            track!(self.0.start_encoding(item))?;
        }
        Ok(())
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.0.requiring_bytes()
    }

    fn is_idle(&self) -> bool {
        self.0.is_idle()
    }
}
impl<E: ExactBytesEncode> ExactBytesEncode for Optional<E> {
    fn exact_requiring_bytes(&self) -> u64 {
        self.0.exact_requiring_bytes()
    }
}

/// Combinator for collecting decoded items.
///
/// This is created by calling `DecodeExt::collect` method.
///
/// Note that this is a oneshot decoder (i.e., it decodes only one item).
#[derive(Debug, Default)]
pub struct Collect<D, T> {
    inner: D,
    items: Option<T>,
}
impl<D, T> Collect<D, T> {
    pub(crate) fn new(inner: D) -> Self {
        Collect { inner, items: None }
    }
}
impl<D, T: Default> Decode for Collect<D, T>
where
    D: Decode,
    T: Extend<D::Item>,
{
    type Item = T;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        if self.items.is_none() {
            self.items = Some(T::default());
        }
        if (buf.is_empty() && eos.is_reached()) || self.inner.has_terminated() {
            return Ok((0, self.items.take()));
        }

        let items = self.items.as_mut().expect("Never fails");
        let (size, item) = track!(self.inner.decode(buf, eos))?;
        items.extend(item);
        Ok((size, None))
    }

    fn has_terminated(&self) -> bool {
        self.inner.has_terminated()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.inner.requiring_bytes()
    }
}

/// Combinator for consuming the specified number of bytes exactly.
///
/// This is created by calling `{DecodeExt, EncodeExt}::length` method.
#[derive(Debug, Default)]
pub struct Length<C> {
    inner: C,
    expected_bytes: u64,
    remaining_bytes: u64,
}
impl<C> Length<C> {
    pub(crate) fn new(inner: C, expected_bytes: u64) -> Self {
        Length {
            inner,
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
    /// If it is in the middle of decoding an item, it willl return an `ErrorKind::Other` error.
    pub fn set_expected_bytes(&mut self, bytes: u64) -> Result<()> {
        track_assert_eq!(
            self.remaining_bytes,
            self.expected_bytes,
            ErrorKind::Other,
            "An item is being decoded"
        );
        self.expected_bytes = bytes;
        self.remaining_bytes = bytes;
        Ok(())
    }

    /// Returns the number of remaining bytes required to decode the next item.
    pub fn remaining_bytes(&self) -> u64 {
        self.remaining_bytes
    }

    /// Returns a reference to the inner encoder or decoder.
    pub fn inner_ref(&self) -> &C {
        &self.inner
    }

    /// Returns a mutable reference to the inner encoder or decoder.
    pub fn inner_mut(&mut self) -> &mut C {
        &mut self.inner
    }

    /// Takes ownership of this instance and returns the inner encoder or decoder.
    pub fn into_inner(self) -> C {
        self.inner
    }
}
impl<D: Decode> Decode for Length<D> {
    type Item = D::Item;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        let limit = cmp::min(buf.len() as u64, self.remaining_bytes) as usize;
        let required = self.remaining_bytes - limit as u64;
        let expected_eos = Eos::with_remaining_bytes(ByteCount::Finite(required));
        if let Some(remaining) = eos.remaining_bytes().to_u64() {
            track_assert!(remaining >= required, ErrorKind::UnexpectedEos; remaining, required);
        }
        let (size, item) = track!(self.inner.decode(&buf[..limit], expected_eos))?;
        self.remaining_bytes -= size as u64;
        if item.is_some() {
            track_assert_eq!(
                self.remaining_bytes,
                0,
                ErrorKind::Other,
                "Decoder consumes too few bytes"
            );
            self.remaining_bytes = self.expected_bytes
        }
        Ok((size, item))
    }

    fn has_terminated(&self) -> bool {
        if self.remaining_bytes == self.expected_bytes {
            self.inner.has_terminated()
        } else {
            false
        }
    }

    fn requiring_bytes(&self) -> ByteCount {
        if self.has_terminated() {
            ByteCount::Finite(0)
        } else {
            ByteCount::Finite(self.remaining_bytes)
        }
    }
}
impl<E: Encode> Encode for Length<E> {
    type Item = E::Item;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        if (buf.len() as u64) < self.remaining_bytes {
            track_assert!(!eos.is_reached(), ErrorKind::UnexpectedEos);
        }

        let (limit, eos) = if (buf.len() as u64) < self.remaining_bytes {
            (buf.len(), eos)
        } else {
            (self.remaining_bytes as usize, Eos::new(true))
        };
        let size = track!(self.inner.encode(&mut buf[..limit], eos))?;
        self.remaining_bytes -= size as u64;
        if self.inner.is_idle() {
            track_assert_eq!(
                self.remaining_bytes,
                0,
                ErrorKind::InvalidInput,
                "Too small item"
            );
        }
        Ok(size)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track_assert_eq!(
            self.remaining_bytes,
            self.expected_bytes,
            ErrorKind::EncoderFull
        );
        self.remaining_bytes = self.expected_bytes;
        track!(self.inner.start_encoding(item))
    }

    fn requiring_bytes(&self) -> ByteCount {
        ByteCount::Finite(self.remaining_bytes)
    }

    fn is_idle(&self) -> bool {
        self.remaining_bytes == 0
    }
}
impl<E: Encode> ExactBytesEncode for Length<E> {
    fn exact_requiring_bytes(&self) -> u64 {
        self.remaining_bytes
    }
}

/// Combinator for decoding the specified number of items.
///
/// This is created by calling `DecodeExt::take` method.
#[derive(Debug, Default)]
pub struct Take<D> {
    inner: D,
    limit: usize,
    decoded_items: usize,
}
impl<D> Take<D> {
    pub(crate) fn new(inner: D, count: usize) -> Self {
        Take {
            inner,
            limit: count,
            decoded_items: 0,
        }
    }
}
impl<D: Decode> Decode for Take<D> {
    type Item = D::Item;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        track_assert_ne!(self.decoded_items, self.limit, ErrorKind::DecoderTerminated);
        match track!(self.inner.decode(buf, eos))? {
            (size, Some(item)) => {
                self.decoded_items += 1;
                Ok((size, Some(item)))
            }
            (size, None) => Ok((size, None)),
        }
    }

    fn has_terminated(&self) -> bool {
        self.inner.has_terminated() || self.decoded_items == self.limit
    }

    fn requiring_bytes(&self) -> ByteCount {
        if self.has_terminated() {
            ByteCount::Finite(0)
        } else {
            self.inner.requiring_bytes()
        }
    }
}

/// Combinator which tries to convert decoded values by calling the specified function.
///
/// This is created by calling `DecodeExt::try_map` method.
#[derive(Debug)]
pub struct TryMap<D, F, T, E> {
    inner: D,
    try_map: F,
    _phantom: PhantomData<(T, E)>,
}
impl<D, F, T, E> TryMap<D, F, T, E> {
    pub(crate) fn new(inner: D, try_map: F) -> Self {
        TryMap {
            inner,
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

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        match track!(self.inner.decode(buf, eos))? {
            (size, Some(item)) => {
                let item = track!((self.try_map)(item).map_err(Error::from))?;
                Ok((size, Some(item)))
            }
            (size, None) => Ok((size, None)),
        }
    }

    fn has_terminated(&self) -> bool {
        self.inner.has_terminated()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.inner.requiring_bytes()
    }
}

/// Combinator for skipping the remaining bytes in an input byte sequence
/// after decoding an item by using `D`.
#[derive(Debug, Default)]
pub struct SkipRemaining<D: Decode> {
    inner: D,
    item: Option<D::Item>,
}
impl<D: Decode> SkipRemaining<D> {
    pub(crate) fn new(inner: D) -> Self {
        SkipRemaining { inner, item: None }
    }
}
impl<D: Decode> Decode for SkipRemaining<D> {
    type Item = D::Item;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        track_assert!(
            !eos.remaining_bytes().is_infinite(),
            ErrorKind::InvalidInput,
            "Cannot skip infinity byte stream"
        );

        if self.item.is_none() {
            let (size, item) = track!(self.inner.decode(buf, eos))?;
            self.item = item;
            Ok((size, None))
        } else if eos.is_reached() {
            Ok((buf.len(), self.item.take()))
        } else {
            Ok((buf.len(), None))
        }
    }

    fn has_terminated(&self) -> bool {
        if self.item.is_none() {
            self.inner.has_terminated()
        } else {
            false
        }
    }

    fn requiring_bytes(&self) -> ByteCount {
        if self.item.is_none() {
            self.inner.requiring_bytes()
        } else {
            ByteCount::Infinite
        }
    }
}

/// Combinator that will fail if the number of consumed bytes exceeds the specified size.
///
/// This is created by calling `{DecodeExt, EncodeExt}::max_bytes` method.
#[derive(Debug, Default)]
pub struct MaxBytes<C> {
    inner: C,
    consumed_bytes: u64,
    max_bytes: u64,
}
impl<C> MaxBytes<C> {
    pub(crate) fn new(inner: C, max_bytes: u64) -> Self {
        MaxBytes {
            inner,
            consumed_bytes: 0,
            max_bytes,
        }
    }

    /// Returns the number of bytes consumed for encoding/decoding the current item.
    pub fn consumed_bytes(&self) -> u64 {
        self.consumed_bytes
    }

    /// Returns the maximum number of bytes that can be consumed for encoding/decoding an item.
    pub fn max_bytes(&self) -> u64 {
        self.max_bytes
    }

    /// Sets the maximum number of bytes that can be consumed for encoding/decoding an item.
    pub fn set_max_bytes(&mut self, n: u64) {
        self.max_bytes = n;
    }

    /// Returns a reference to the inner encoder or decoder.
    pub fn inner_ref(&self) -> &C {
        &self.inner
    }

    /// Returns a mutable reference to the inner encoder or decoder.
    pub fn inner_mut(&mut self) -> &mut C {
        &mut self.inner
    }

    /// Takes ownership of this instance and returns the inner encoder or decoder.
    pub fn into_inner(self) -> C {
        self.inner
    }
}
impl<D: Decode> Decode for MaxBytes<D> {
    type Item = D::Item;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        let (size, item) = track!(self.inner.decode(buf, eos))?;
        self.consumed_bytes += size as u64;
        track_assert!(self.consumed_bytes <= self.max_bytes,
                      ErrorKind::InvalidInput, "Max bytes limit exceeded";
                      self.consumed_bytes, self.max_bytes);
        if item.is_some() {
            self.consumed_bytes = 0;
        }
        Ok((size, item))
    }

    fn has_terminated(&self) -> bool {
        self.inner.has_terminated()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.inner.requiring_bytes()
    }
}
impl<E: Encode> Encode for MaxBytes<E> {
    type Item = E::Item;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let size = track!(self.inner.encode(buf, eos))?;
        self.consumed_bytes += size as u64;
        track_assert!(self.consumed_bytes <= self.max_bytes,
                      ErrorKind::InvalidInput, "Max bytes limit exceeded";
                      self.consumed_bytes, self.max_bytes);
        if self.is_idle() {
            self.consumed_bytes = 0;
        }
        Ok(size)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track!(self.inner.start_encoding(item))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.inner.requiring_bytes()
    }

    fn is_idle(&self) -> bool {
        self.inner.is_idle()
    }
}
impl<E: ExactBytesEncode> ExactBytesEncode for MaxBytes<E> {
    fn exact_requiring_bytes(&self) -> u64 {
        self.inner.exact_requiring_bytes()
    }
}

/// Combinator for declaring an assertion about decoded items.
///
/// This created by calling `DecodeExt::assert` method.
#[derive(Debug)]
pub struct Assert<D, F> {
    inner: D,
    assert: F,
}
impl<D, F> Assert<D, F> {
    pub(crate) fn new(inner: D, assert: F) -> Self {
        Assert { inner, assert }
    }
}
impl<D: Decode, F> Decode for Assert<D, F>
where
    F: for<'a> Fn(&'a D::Item) -> bool,
{
    type Item = D::Item;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        let (size, item) = track!(self.inner.decode(buf, eos))?;
        if let Some(ref item) = item {
            track_assert!((self.assert)(item), ErrorKind::InvalidInput);
        }
        Ok((size, item))
    }

    fn has_terminated(&self) -> bool {
        self.inner.has_terminated()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.inner.requiring_bytes()
    }
}

/// Combinator that keeps writing padding byte until it reaches EOS
/// after encoding of `E`'s item has been completed.
///
/// This is created by calling `EncodeExt::padding` method.
#[derive(Debug, Default)]
pub struct Padding<E> {
    inner: E,
    padding_byte: u8,
    eos_reached: bool,
}
impl<E> Padding<E> {
    pub(crate) fn new(inner: E, padding_byte: u8) -> Self {
        Padding {
            inner,
            padding_byte,
            eos_reached: true,
        }
    }
}
impl<E: Encode> Encode for Padding<E> {
    type Item = E::Item;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        if !self.inner.is_idle() {
            self.inner.encode(buf, eos)
        } else {
            for b in buf.iter_mut() {
                *b = self.padding_byte;
            }
            self.eos_reached = eos.is_reached();
            Ok(buf.len())
        }
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track_assert!(self.is_idle(), ErrorKind::EncoderFull);
        self.eos_reached = false;
        track!(self.inner.start_encoding(item))
    }

    fn requiring_bytes(&self) -> ByteCount {
        ByteCount::Infinite
    }

    fn is_idle(&self) -> bool {
        self.eos_reached
    }
}

/// Combinator for adding prefix items.
///
/// This is created by calling `EncodeExt::with_prefix` method.
#[derive(Debug)]
pub struct WithPrefix<E0, E1, F> {
    body_encoder: E0,
    prefix_encoder: E1,
    with_prefix: F,
}
impl<E0, E1, F> WithPrefix<E0, E1, F> {
    pub(crate) fn new(body_encoder: E0, prefix_encoder: E1, with_prefix: F) -> Self {
        WithPrefix {
            body_encoder,
            prefix_encoder,
            with_prefix,
        }
    }
}
impl<E0, E1, F> Encode for WithPrefix<E0, E1, F>
where
    E0: Encode,
    E1: Encode,
    F: Fn(&E0) -> E1::Item,
{
    type Item = E0::Item;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        if !self.prefix_encoder.is_idle() {
            track!(self.prefix_encoder.encode(buf, eos))
        } else {
            track!(self.body_encoder.encode(buf, eos))
        }
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track_assert!(self.is_idle(), ErrorKind::EncoderFull);
        track!(self.body_encoder.start_encoding(item))?;
        let prefix_item = (self.with_prefix)(&self.body_encoder);
        track!(self.prefix_encoder.start_encoding(prefix_item))?;
        Ok(())
    }

    fn requiring_bytes(&self) -> ByteCount {
        let x = self.prefix_encoder.requiring_bytes();
        let y = self.body_encoder.requiring_bytes();
        match (x, y) {
            (ByteCount::Finite(x), ByteCount::Finite(y)) => ByteCount::Finite(x + y),
            (ByteCount::Infinite, _) | (_, ByteCount::Infinite) => ByteCount::Infinite,
            (ByteCount::Unknown, _) | (_, ByteCount::Unknown) => ByteCount::Unknown,
        }
    }

    fn is_idle(&self) -> bool {
        self.prefix_encoder.is_idle() && self.body_encoder.is_idle()
    }
}
impl<E0, E1, F> ExactBytesEncode for WithPrefix<E0, E1, F>
where
    E0: ExactBytesEncode,
    E1: ExactBytesEncode,
    F: Fn(&E0) -> E1::Item,
{
    fn exact_requiring_bytes(&self) -> u64 {
        self.prefix_encoder.exact_requiring_bytes() + self.body_encoder.exact_requiring_bytes()
    }
}

/// Combinator for pre-encoding items when `start_encoding` method is called.
///
/// This is created by calling `EncodeExt::pre_encode` method.
#[derive(Debug, Default)]
pub struct PreEncode<E> {
    inner: E,
    pre_encoded: BytesEncoder<Vec<u8>>,
}
impl<E> PreEncode<E> {
    pub(crate) fn new(inner: E) -> Self {
        PreEncode {
            inner,
            pre_encoded: BytesEncoder::new(),
        }
    }
}
impl<E: Encode> Encode for PreEncode<E> {
    type Item = E::Item;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        track!(self.pre_encoded.encode(buf, eos))
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        let mut buf = Vec::new();
        track!(self.inner.start_encoding(item))?;
        track!(self.inner.encode_all(&mut buf))?;
        track!(self.pre_encoded.start_encoding(buf))?;
        Ok(())
    }

    fn requiring_bytes(&self) -> ByteCount {
        ByteCount::Finite(self.exact_requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.pre_encoded.is_idle()
    }
}
impl<E: Encode> ExactBytesEncode for PreEncode<E> {
    fn exact_requiring_bytes(&self) -> u64 {
        self.pre_encoded.exact_requiring_bytes()
    }
}

/// Combinator for slicing an input/output byte sequence by the specified number of bytes.
///
/// This is created by calling `{DecodeExt, EncodeExt}::slice`.
#[derive(Debug, Default)]
pub struct Slice<T> {
    inner: T,
    consumable_bytes: u64,
}
impl<T> Slice<T> {
    pub(crate) fn new(inner: T) -> Self {
        Slice {
            inner,
            consumable_bytes: 0,
        }
    }

    /// Returns the number of remaining bytes consumable in this slice.
    ///
    /// The inner decoder or encoder will be suspended if the consumable bytes reaches to `0`.
    pub fn consumable_bytes(&self) -> u64 {
        self.consumable_bytes
    }

    /// Set the number of remaining bytes consumable in this slice.
    pub fn set_consumable_bytes(&mut self, n: u64) {
        self.consumable_bytes = n;
    }

    /// Returns `true` if the encoder or decoder cannot consume any more bytes, otherwise `false`.
    ///
    /// To resume its works, it is needed to reset the value of consumable bytes
    /// by calling `set_consumable_bytes` method.
    pub fn is_suspended(&self) -> bool {
        self.consumable_bytes == 0
    }

    /// Returns a reference to the inner encoder or decoder.
    pub fn inner_ref(&self) -> &T {
        &self.inner
    }

    /// Returns a mutable reference to the inner encoder or decoder.
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Takes ownership of this instance and returns the inner encoder or decoder.
    pub fn into_inner(self) -> T {
        self.inner
    }
}
impl<D: Decode> Decode for Slice<D> {
    type Item = D::Item;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        let limit = cmp::min(buf.len() as u64, self.consumable_bytes) as usize;
        let eos = eos.back((buf.len() - limit) as u64);
        let (size, item) = track!(self.inner.decode(&buf[..limit], eos))?;
        self.consumable_bytes -= size as u64;
        Ok((size, item))
    }

    fn has_terminated(&self) -> bool {
        self.inner.has_terminated()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.inner.requiring_bytes()
    }
}
impl<E: Encode> Encode for Slice<E> {
    type Item = E::Item;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let limit = cmp::min(buf.len() as u64, self.consumable_bytes) as usize;
        let eos = eos.back((buf.len() - limit) as u64);
        let size = track!(self.inner.encode(&mut buf[..limit], eos))?;
        self.consumable_bytes -= size as u64;
        Ok(size)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track!(self.inner.start_encoding(item))
    }

    fn is_idle(&self) -> bool {
        self.inner.is_idle()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.inner.requiring_bytes()
    }
}
impl<E: ExactBytesEncode> ExactBytesEncode for Slice<E> {
    fn exact_requiring_bytes(&self) -> u64 {
        self.inner.exact_requiring_bytes()
    }
}

/// Combinator for representing encoders that cannot accept any more items.
#[derive(Debug, Default)]
pub struct Last<E> {
    inner: E,
}
impl<E> Last<E> {
    pub(crate) fn new(inner: E) -> Self {
        Last { inner }
    }
}
impl<E: Encode> Encode for Last<E> {
    type Item = Never;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        track!(self.inner.encode(buf, eos))
    }

    fn start_encoding(&mut self, _item: Self::Item) -> Result<()> {
        unreachable!()
    }

    fn is_idle(&self) -> bool {
        self.inner.is_idle()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.inner.requiring_bytes()
    }
}
impl<E: ExactBytesEncode> ExactBytesEncode for Last<E> {
    fn exact_requiring_bytes(&self) -> u64 {
        self.inner.exact_requiring_bytes()
    }
}

#[cfg(test)]
mod test {
    use {Decode, DecodeExt, Encode, EncodeExt, Eos, ErrorKind};
    use bytes::{Utf8Decoder, Utf8Encoder};
    use fixnum::{U8Decoder, U8Encoder};
    use io::{IoDecodeExt, IoEncodeExt};

    #[test]
    fn collect_works() {
        let mut decoder = U8Decoder::new().collect::<Vec<_>>();
        let item = track_try_unwrap!(decoder.decode_exact(b"foo".as_ref()));
        assert_eq!(item, vec![b'f', b'o', b'o']);
    }

    #[test]
    fn take_works() {
        let mut decoder = U8Decoder::new().take(2).collect::<Vec<_>>();
        let item = track_try_unwrap!(decoder.decode_exact(b"foo".as_ref()));
        assert_eq!(item, vec![b'f', b'o']);
    }

    #[test]
    fn decoder_length_works() {
        let mut decoder = Utf8Decoder::new().length(3);
        let mut input = b"foobarba".as_ref();

        let item = track_try_unwrap!(decoder.decode_exact(&mut input));
        assert_eq!(item, "foo");

        let item = track_try_unwrap!(decoder.decode_exact(&mut input));
        assert_eq!(item, "bar");

        let error = decoder.decode_exact(&mut input).err().unwrap();
        assert_eq!(*error.kind(), ErrorKind::UnexpectedEos);
    }

    #[test]
    fn encoder_length_works() {
        let mut output = Vec::new();
        let mut encoder = Utf8Encoder::new().length(3);
        encoder.start_encoding("hey").unwrap(); // OK
        track_try_unwrap!(encoder.encode_all(&mut output));
        assert_eq!(output, b"hey");

        let mut output = Vec::new();
        let mut encoder = Utf8Encoder::new().length(3);
        encoder.start_encoding("hello").unwrap(); // Error (too long)
        let error = encoder.encode_all(&mut output).err().expect("too long");
        assert_eq!(*error.kind(), ErrorKind::UnexpectedEos);

        let mut output = Vec::new();
        let mut encoder = Utf8Encoder::new().length(3);
        encoder.start_encoding("hi").unwrap(); // Error (too short)
        let error = encoder.encode_all(&mut output).err().expect("too short");
        assert_eq!(*error.kind(), ErrorKind::InvalidInput);
    }

    #[test]
    fn padding_works() {
        let mut output = Vec::new();
        let mut encoder = U8Encoder::new().padding(9).length(3);
        encoder.start_encoding(3).unwrap();
        track_try_unwrap!(encoder.encode_all(&mut output));
        assert_eq!(output, [3, 9, 9]);
    }

    #[test]
    fn repeat_works() {
        let mut output = Vec::new();
        let mut encoder = U8Encoder::new().repeat();
        encoder.start_encoding(0..4).unwrap();
        track_try_unwrap!(encoder.encode_all(&mut output));
        assert_eq!(output, [0, 1, 2, 3]);
    }

    #[test]
    fn encoder_max_bytes_works() {
        let mut encoder = Utf8Encoder::new().max_bytes(3);

        let mut output = Vec::new();
        encoder.start_encoding("foo").unwrap(); // OK
        encoder.encode_all(&mut output).unwrap();
        assert_eq!(output, b"foo");

        let mut output = Vec::new();
        encoder.start_encoding("hello").unwrap(); // Error
        let error = encoder.encode_all(&mut output).err().unwrap();
        assert_eq!(*error.kind(), ErrorKind::InvalidInput);
    }

    #[test]
    fn decoder_slice_works() {
        let mut decoder0 = Utf8Decoder::new().length(3).slice();
        let mut decoder1 = Utf8Decoder::new().length(3).slice();

        let eos = Eos::new(true);
        let input = b"fboaor";
        let mut offset = 0;

        let mut last_item0 = None;
        let mut last_item1 = None;
        for _ in 0..3 {
            decoder0.set_consumable_bytes(1);
            let (size, item) = track_try_unwrap!(decoder0.decode(&input[offset..], eos));
            offset += size;
            last_item0 = item;

            decoder1.set_consumable_bytes(1);
            let (size, item) = track_try_unwrap!(decoder1.decode(&input[offset..], eos));
            offset += size;
            last_item1 = item;
        }

        assert_eq!(offset, input.len());
        assert_eq!(last_item0, Some("foo".to_owned()));
        assert_eq!(last_item1, Some("bar".to_owned()));
    }

    #[test]
    fn encoder_slice_works() {
        let mut encoder = Utf8Encoder::new().slice();
        encoder.start_encoding("foobarbazqux").unwrap();

        let eos = Eos::new(true);
        let mut output = [0; 12];
        let mut offset = 0;
        encoder.set_consumable_bytes(3);
        offset += track_try_unwrap!(encoder.encode(&mut output[offset..], eos));
        assert_eq!(offset, 3);
        assert!(!encoder.is_idle());

        offset += track_try_unwrap!(encoder.encode(&mut output[offset..], eos));
        assert_eq!(offset, 3);
        assert_eq!(encoder.is_suspended(), true);

        encoder.set_consumable_bytes(3);
        offset += track_try_unwrap!(encoder.encode(&mut output[offset..], eos));
        assert_eq!(offset, 6);

        encoder.set_consumable_bytes(6);
        offset += track_try_unwrap!(encoder.encode(&mut output[offset..], eos));
        assert_eq!(offset, 12);

        assert!(encoder.is_idle());
        assert_eq!(output.as_ref(), b"foobarbazqux");
    }

    #[test]
    fn and_then_works() {
        let mut decoder =
            U8Decoder::new().and_then(|len| Utf8Decoder::new().length(u64::from(len)));
        let (_, item) = track_try_unwrap!(decoder.decode(b"\x03foo", Eos::new(false)));
        assert_eq!(item, Some("foo".to_owned()));
    }
}
