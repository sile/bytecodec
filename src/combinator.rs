//! Encoders and decoders for combination.
//!
//! These are mainly created via the methods provided by `EncodeExt` or `DecodeExt` traits.
use std;
use std::cmp;
use std::fmt;
use std::iter;
use std::marker::PhantomData;
use std::mem;

use bytes::BytesEncoder;
use marker::Never;
use {ByteCount, Decode, Encode, EncodeExt, Eos, Error, ErrorKind, Result, SizedEncode};

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
    /// Returns a reference to the inner decoder.
    pub fn inner_ref(&self) -> &D {
        &self.inner
    }

    /// Returns a mutable reference to the inner decoder.
    pub fn inner_mut(&mut self) -> &mut D {
        &mut self.inner
    }

    /// Takes ownership of this instance and returns the inner decoder.
    pub fn into_inner(self) -> D {
        self.inner
    }

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

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        track!(self.inner.decode(buf, eos))
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        let item = track!(self.inner.finish_decoding())?;
        Ok((self.map)(item))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.inner.requiring_bytes()
    }
}

/// Combinator for modifying encoding/decoding errors.
///
/// This is created by calling `{DecodeExt, EncodeExt}::map_err` method.
#[derive(Debug)]
pub struct MapErr<C, E, F> {
    inner: C,
    map_err: F,
    _error: PhantomData<E>,
}
impl<C, E, F> MapErr<C, E, F> {
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
impl<D, E, F> Decode for MapErr<D, E, F>
where
    D: Decode,
    F: Fn(Error) -> E,
    Error: From<E>,
{
    type Item = D::Item;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        self.inner
            .decode(buf, eos)
            .map_err(|e| (self.map_err)(e).into())
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        self.inner
            .finish_decoding()
            .map_err(|e| (self.map_err)(e).into())
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.inner.requiring_bytes()
    }
}
impl<C, E, F> Encode for MapErr<C, E, F>
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

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        if self.inner1.is_none() {
            bytecodec_try_decode!(self.inner0, offset, buf, eos);
            let item = track!(self.inner0.finish_decoding())?;
            self.inner1 = Some((self.and_then)(item));
        }

        let inner1 = self.inner1.as_mut().expect("Never fails");
        bytecodec_try_decode!(inner1, offset, buf, eos);
        Ok(offset)
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        let mut d = track_assert_some!(self.inner1.take(), ErrorKind::IncompleteItem);
        track!(d.finish_decoding())
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
    /// Returns a reference to the inner encoder.
    pub fn inner_ref(&self) -> &E {
        &self.inner
    }

    /// Returns a mutable reference to the inner encoder.
    pub fn inner_mut(&mut self) -> &mut E {
        &mut self.inner
    }

    /// Takes ownership of this instance and returns the inner encoder.
    pub fn into_inner(self) -> E {
        self.inner
    }

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
    /// Returns a reference to the inner encoder.
    pub fn inner_ref(&self) -> &C {
        &self.inner
    }

    /// Returns a mutable reference to the inner encoder.
    pub fn inner_mut(&mut self) -> &mut C {
        &mut self.inner
    }

    /// Takes ownership of this instance and returns the inner encoder.
    pub fn into_inner(self) -> C {
        self.inner
    }

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

/// Combinator for repeating encoding of `E::Item`.
///
/// This is created by calling `EncodeExt::repeat` method.
#[derive(Debug)]
pub struct Repeat<E, I> {
    inner: E,
    items: Option<I>,
}
impl<E, I> Repeat<E, I> {
    /// Returns a reference to the inner encoder.
    pub fn inner_ref(&self) -> &E {
        &self.inner
    }

    /// Returns a mutable reference to the inner encoder.
    pub fn inner_mut(&mut self) -> &mut E {
        &mut self.inner
    }

    /// Takes ownership of this instance and returns the inner encoder.
    pub fn into_inner(self) -> E {
        self.inner
    }

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
impl<E: Default, I> Default for Repeat<E, I> {
    fn default() -> Self {
        Self::new(E::default())
    }
}

/// Combinator for representing optional decoders.
///
/// This is created by calling `DecodeExt::omit` method.
#[derive(Debug, Default)]
pub struct Omittable<D> {
    inner: D,
    do_omit: bool,
}
impl<D> Omittable<D> {
    /// Returns a reference to the inner decoder.
    pub fn inner_ref(&self) -> &D {
        &self.inner
    }

    /// Returns a mutable reference to the inner decoder.
    pub fn inner_mut(&mut self) -> &mut D {
        &mut self.inner
    }

    /// Takes ownership of this instance and returns the inner decoder.
    pub fn into_inner(self) -> D {
        self.inner
    }

    /// If `true` is specified, the decoder will consume no bytes and
    /// return `Ok((0, None))` when `decode` method is called.
    pub fn do_omit(&mut self, b: bool) {
        self.do_omit = b;
    }

    /// Returns `true` if the decoder will omit to decode items, otherwise `false`.
    pub fn will_omit(&self) -> bool {
        self.do_omit
    }

    pub(crate) fn new(inner: D, do_omit: bool) -> Self {
        Omittable { inner, do_omit }
    }
}
impl<D: Decode> Decode for Omittable<D> {
    type Item = Option<D::Item>;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        if self.do_omit {
            Ok(0)
        } else {
            track!(self.inner.decode(buf, eos))
        }
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        if self.do_omit {
            Ok(None)
        } else {
            track!(self.inner.finish_decoding()).map(Some)
        }
    }

    fn requiring_bytes(&self) -> ByteCount {
        if self.do_omit {
            ByteCount::Finite(0)
        } else {
            self.inner.requiring_bytes()
        }
    }
}

/// Combinator for representing an optional encoder.
#[derive(Debug, Default)]
pub struct Optional<E>(E);
impl<E> Optional<E> {
    /// Returns a reference to the inner encoder.
    pub fn inner_ref(&self) -> &E {
        &self.0
    }

    /// Returns a mutable reference to the inner encoder.
    pub fn inner_mut(&mut self) -> &mut E {
        &mut self.0
    }

    /// Takes ownership of this instance and returns the inner encoder.
    pub fn into_inner(self) -> E {
        self.0
    }

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

/// Combinator for collecting decoded items.
///
/// `Collect` decodes all items until it reaches EOS
/// and returns the collected items as the single decoded item.
///
/// This is created by calling `DecodeExt::collect` method.
#[derive(Debug, Default)]
pub struct Collect<D, T> {
    inner: D,
    items: T,
    eos: bool,
}
impl<D, T: Default> Collect<D, T> {
    /// Returns a reference to the inner decoder.
    pub fn inner_ref(&self) -> &D {
        &self.inner
    }

    /// Returns a mutable reference to the inner decoder.
    pub fn inner_mut(&mut self) -> &mut D {
        &mut self.inner
    }

    /// Takes ownership of this instance and returns the inner decoder.
    pub fn into_inner(self) -> D {
        self.inner
    }

    pub(crate) fn new(inner: D) -> Self {
        Collect {
            inner,
            items: T::default(),
            eos: false,
        }
    }
}
impl<D, T: Default> Decode for Collect<D, T>
where
    D: Decode,
    T: Extend<D::Item>,
{
    type Item = T;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        while offset < buf.len() {
            bytecodec_try_decode!(self.inner, offset, buf, eos);

            let item = track!(self.inner.finish_decoding())?;
            self.items.extend(iter::once(item));
        }
        if eos.is_reached() {
            track_assert!(self.inner.is_idle(), ErrorKind::UnexpectedEos);
            self.eos = true;
        }
        Ok(offset)
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        track_assert!(self.eos, ErrorKind::IncompleteItem);
        self.eos = false;
        let items = mem::replace(&mut self.items, T::default());
        Ok(items)
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

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        let limit = cmp::min(buf.len() as u64, self.remaining_bytes) as usize;
        let required = self.remaining_bytes - limit as u64;
        let expected_eos = Eos::with_remaining_bytes(ByteCount::Finite(required));
        if let Some(remaining) = eos.remaining_bytes().to_u64() {
            track_assert!(remaining >= required, ErrorKind::UnexpectedEos; remaining, required);
        }

        let size = track!(self.inner.decode(&buf[..limit], expected_eos))?;
        self.remaining_bytes -= size as u64;
        Ok(size)
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        track_assert_eq!(self.remaining_bytes, 0, ErrorKind::IncompleteItem);
        self.remaining_bytes = self.expected_bytes;

        track!(self.inner.finish_decoding())
    }

    fn requiring_bytes(&self) -> ByteCount {
        ByteCount::Finite(self.remaining_bytes)
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

/// Combinator for decoding the specified number of items and collecting the result.
///
/// This is created by calling `DecodeExt::collectn` method.
#[derive(Debug, Default)]
pub struct CollectN<D, T> {
    inner: D,
    remaining_items: usize,
    items: T,
}
impl<D, T: Default> CollectN<D, T> {
    /// Returns the number of remaining items expected to be decoded.
    pub fn remaining_items(&self) -> usize {
        self.remaining_items
    }

    /// Sets the number of remaining items expected to be decoded.
    pub fn set_remaining_items(&mut self, n: usize) {
        self.remaining_items = n;
    }

    /// Returns a reference to the inner decoder.
    pub fn inner_ref(&self) -> &D {
        &self.inner
    }

    /// Returns a mutable reference to the inner decoder.
    pub fn inner_mut(&mut self) -> &mut D {
        &mut self.inner
    }

    /// Takes ownership of this instance and returns the inner decoder.
    pub fn into_inner(self) -> D {
        self.inner
    }

    pub(crate) fn new(inner: D, count: usize) -> Self {
        CollectN {
            inner,
            remaining_items: count,
            items: T::default(),
        }
    }
}
impl<D, T> Decode for CollectN<D, T>
where
    D: Decode,
    T: Default + Extend<D::Item>,
{
    type Item = T;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        while self.remaining_items != 0 && offset < buf.len() {
            bytecodec_try_decode!(self.inner, offset, buf, eos);

            let item = track!(self.inner.finish_decoding())?;
            self.items.extend(iter::once(item));
            self.remaining_items -= 1;
        }
        if self.remaining_items != 0 {
            track_assert!(!eos.is_reached(), ErrorKind::UnexpectedEos);
        }
        Ok(offset)
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        track_assert_eq!(self.remaining_items, 0, ErrorKind::IncompleteItem);
        let items = mem::replace(&mut self.items, T::default());
        Ok(items)
    }

    fn requiring_bytes(&self) -> ByteCount {
        if self.remaining_items == 0 {
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
pub struct TryMap<D, T, E, F> {
    inner: D,
    try_map: F,
    _phantom: PhantomData<(T, E)>,
}
impl<D, T, E, F> TryMap<D, T, E, F> {
    /// Returns a reference to the inner decoder.
    pub fn inner_ref(&self) -> &D {
        &self.inner
    }

    /// Returns a mutable reference to the inner decoder.
    pub fn inner_mut(&mut self) -> &mut D {
        &mut self.inner
    }

    /// Takes ownership of this instance and returns the inner decoder.
    pub fn into_inner(self) -> D {
        self.inner
    }

    pub(crate) fn new(inner: D, try_map: F) -> Self {
        TryMap {
            inner,
            try_map,
            _phantom: PhantomData,
        }
    }
}
impl<D, T, E, F> Decode for TryMap<D, T, E, F>
where
    D: Decode,
    F: Fn(D::Item) -> std::result::Result<T, E>,
    Error: From<E>,
{
    type Item = T;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        track!(self.inner.decode(buf, eos))
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        let item = track!(self.inner.finish_decoding())?;
        track!((self.try_map)(item).map_err(Error::from))
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
    eos: bool,
}
impl<D: Decode> SkipRemaining<D> {
    pub(crate) fn new(inner: D) -> Self {
        SkipRemaining { inner, eos: false }
    }

    /// Returns a reference to the inner decoder.
    pub fn inner_ref(&self) -> &D {
        &self.inner
    }

    /// Returns a mutable reference to the inner decoder.
    pub fn inner_mut(&mut self) -> &mut D {
        &mut self.inner
    }

    /// Takes ownership of this instance and returns the inner decoder.
    pub fn into_inner(self) -> D {
        self.inner
    }
}
impl<D: Decode> Decode for SkipRemaining<D> {
    type Item = D::Item;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        track_assert!(
            !eos.remaining_bytes().is_infinite(),
            ErrorKind::InvalidInput,
            "Cannot skip infinity byte stream"
        );

        if self.eos {
            Ok(0)
        } else if self.inner.is_idle() {
            self.eos = eos.is_reached();
            Ok(buf.len())
        } else {
            track!(self.inner.decode(buf, eos))
        }
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        track_assert!(self.eos, ErrorKind::IncompleteItem);
        self.eos = false;

        track!(self.inner.finish_decoding())
    }

    fn requiring_bytes(&self) -> ByteCount {
        if self.eos {
            ByteCount::Finite(0)
        } else if self.inner.is_idle() {
            ByteCount::Infinite
        } else {
            self.inner.requiring_bytes()
        }
    }
}

/// Combinator that will fail if the number of consumed bytes exceeds the specified size.
///
/// This is created by calling `{DecodeExt, EncodeExt}::max_bytes` method.
///
/// Note that `MaxBytes` assumes the inner decoder will consume all the bytes in the target stream.
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
    ///
    /// # Error
    ///
    /// If `n` is smaller than `self.consumed_bytes()`, an `ErrorKind::InvalidInput` error will be returned.
    pub fn set_max_bytes(&mut self, n: u64) -> Result<()> {
        track_assert!(
            self.consumed_bytes <= n,
            ErrorKind::InvalidInput;
            self.consumed_bytes,
            n
        );
        self.max_bytes = n;
        Ok(())
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

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        match eos.remaining_bytes() {
            ByteCount::Infinite => {
                track_panic!(ErrorKind::InvalidInput, "Max bytes limit exceeded";
                             self.consumed_bytes, self.max_bytes)
            }
            ByteCount::Unknown => {
                let consumable_bytes = self.max_bytes - self.consumed_bytes;
                track_assert!((buf.len() as u64) <= consumable_bytes,
                              ErrorKind::InvalidInput, "Max bytes limit exceeded";
                              buf.len(), self.consumed_bytes, self.max_bytes);
            }
            ByteCount::Finite(remaining_bytes) => {
                let consumable_bytes = self.max_bytes - self.consumed_bytes;
                track_assert!((buf.len() as u64) + remaining_bytes <= consumable_bytes,
                              ErrorKind::InvalidInput, "Max bytes limit exceeded";
                              buf.len(), remaining_bytes, self.consumed_bytes, self.max_bytes)
            }
        }

        let size = track!(self.inner.decode(buf, eos))?;
        self.consumed_bytes += size as u64;
        Ok(size)
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        track!(self.inner.finish_decoding())
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
    /// Returns a reference to the inner encoder.
    pub fn inner_ref(&self) -> &E {
        &self.inner
    }

    /// Returns a mutable reference to the inner encoder.
    pub fn inner_mut(&mut self) -> &mut E {
        &mut self.inner
    }

    /// Takes ownership of this instance and returns the inner encoder.
    pub fn into_inner(self) -> E {
        self.inner
    }

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
        let mut offset = 0;
        bytecodec_try_encode!(self.inner, offset, buf, eos);

        for b in (&mut buf[offset..]).iter_mut() {
            *b = self.padding_byte;
        }
        self.eos_reached = eos.is_reached();
        Ok(buf.len())
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

/// Combinator for pre-encoding items when `start_encoding` method is called.
///
/// This is created by calling `EncodeExt::pre_encode` method.
#[derive(Debug, Default)]
pub struct PreEncode<E> {
    inner: E,
    pre_encoded: BytesEncoder<Vec<u8>>,
}
impl<E> PreEncode<E> {
    /// Returns a reference to the inner encoder.
    pub fn inner_ref(&self) -> &E {
        &self.inner
    }

    /// Returns a mutable reference to the inner encoder.
    pub fn inner_mut(&mut self) -> &mut E {
        &mut self.inner
    }

    /// Takes ownership of this instance and returns the inner encoder.
    pub fn into_inner(self) -> E {
        self.inner
    }

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
        let buf = track!(self.inner.encode_into_bytes(item))?;
        track!(self.pre_encoded.start_encoding(buf))?;
        Ok(())
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.pre_encoded.requiring_bytes()
    }

    fn is_idle(&self) -> bool {
        self.pre_encoded.is_idle()
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

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        let limit = cmp::min(buf.len() as u64, self.consumable_bytes) as usize;
        let eos = eos.back((buf.len() - limit) as u64);
        let size = track!(self.inner.decode(&buf[..limit], eos))?;
        self.consumable_bytes -= size as u64;
        Ok(size)
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        track!(self.inner.finish_decoding())
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

/// Combinator for representing encoders that accepts only one additional item.
///
/// This is created by calling `EncodeExt::last`.
#[derive(Debug, Default)]
pub struct Last<E: Encode> {
    inner: E,
    item: Option<E::Item>,
}
impl<E: Encode> Last<E> {
    /// Returns a reference to the inner encoder.
    pub fn inner_ref(&self) -> &E {
        &self.inner
    }

    /// Returns a mutable reference to the inner encoder.
    pub fn inner_mut(&mut self) -> &mut E {
        &mut self.inner
    }

    /// Takes ownership of this instance and returns the inner encoder.
    pub fn into_inner(self) -> E {
        self.inner
    }

    pub(crate) fn new(inner: E, item: E::Item) -> Self {
        Last {
            inner,
            item: Some(item),
        }
    }
}
impl<E: Encode> Encode for Last<E> {
    type Item = Never;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        if self.inner.is_idle() {
            if let Some(item) = self.item.take() {
                track!(self.inner.start_encoding(item))?;
            }
        }
        track!(self.inner.encode(buf, eos))
    }

    fn start_encoding(&mut self, _item: Self::Item) -> Result<()> {
        unreachable!()
    }

    fn is_idle(&self) -> bool {
        self.item.is_none() && self.inner.is_idle()
    }

    fn requiring_bytes(&self) -> ByteCount {
        if self.item.is_some() {
            ByteCount::Unknown
        } else {
            self.inner.requiring_bytes()
        }
    }
}

/// Combinator for ignoring EOS if there is no item being decoded.
///
/// This is created by calling `DecodeExt::maybe_eos`.
#[derive(Debug, Default)]
pub struct MaybeEos<D> {
    inner: D,
    started: bool,
}
impl<D> MaybeEos<D> {
    /// Returns a reference to the inner decoder.
    pub fn inner_ref(&self) -> &D {
        &self.inner
    }

    /// Returns a mutable reference to the inner decoder.
    pub fn inner_mut(&mut self) -> &mut D {
        &mut self.inner
    }

    /// Takes ownership of this instance and returns the inner decoder.
    pub fn into_inner(self) -> D {
        self.inner
    }

    pub(crate) fn new(inner: D) -> Self {
        MaybeEos {
            inner,
            started: false,
        }
    }
}
impl<D: Decode> Decode for MaybeEos<D> {
    type Item = D::Item;

    fn decode(&mut self, buf: &[u8], mut eos: Eos) -> Result<usize> {
        if !self.started && buf.is_empty() && eos.is_reached() {
            eos = Eos::new(false);
        }

        let size = track!(self.inner.decode(buf, eos))?;
        if size != 0 {
            self.started = true;
        }
        Ok(size)
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        self.started = false;
        track!(self.inner.finish_decoding())
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.inner.requiring_bytes()
    }
}

/// Combinator that gives `ExactBytesEncode` trait to `E` by calculating
/// the exact number of bytes required for encoding items in advance.
///
/// This is created by calling `EncodeExt::pre_calculate`.
#[derive(Debug, Default)]
pub struct PreCalculate<E>(Length<E>);
impl<E> PreCalculate<E> {
    /// Returns a reference to the inner encoder.
    pub fn inner_ref(&self) -> &E {
        &self.0.inner
    }

    /// Returns a mutable reference to the inner encoder.
    pub fn inner_mut(&mut self) -> &mut E {
        &mut self.0.inner
    }

    /// Takes ownership of this instance and returns the inner encoder.
    pub fn into_inner(self) -> E {
        self.0.inner
    }

    pub(crate) fn new(inner: E) -> Self {
        PreCalculate(Length::new(inner, 0))
    }
}
impl<E: SizedEncode> Encode for PreCalculate<E> {
    type Item = E::Item;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        track!(self.0.encode(buf, eos))
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        let size = self.0.inner_ref().encoded_size_of(&item);
        track!(self.0.start_encoding(item))?;
        track!(self.0.set_expected_bytes(size))?;
        Ok(())
    }

    fn is_idle(&self) -> bool {
        self.0.is_idle()
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.0.requiring_bytes()
    }
}

#[cfg(test)]
mod test {
    use bytes::{Utf8Decoder, Utf8Encoder};
    use fixnum::{U16beDecoder, U8Decoder, U8Encoder};
    use io::{IoDecodeExt, IoEncodeExt};
    use {Decode, DecodeExt, Encode, EncodeExt, Eos, ErrorKind};

    #[test]
    fn collect_works() {
        let mut decoder = U8Decoder::new().collect::<Vec<_>>();
        let item = track_try_unwrap!(decoder.decode_exact(b"foo".as_ref()));
        assert_eq!(item, vec![b'f', b'o', b'o']);
    }

    #[test]
    fn collectn_works() {
        let mut decoder = U8Decoder::new().collectn::<Vec<_>>(2);
        let item = track_try_unwrap!(decoder.decode_exact(b"foo".as_ref()));
        assert_eq!(item, vec![b'f', b'o']);

        let mut decoder = U8Decoder::new().collectn::<Vec<_>>(4);
        assert_eq!(
            decoder
                .decode_exact(b"foo".as_ref())
                .err()
                .map(|e| *e.kind()),
            Some(ErrorKind::UnexpectedEos)
        );
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

    #[test]
    fn maybe_eos_works() {
        let mut decoder = U16beDecoder::new();
        assert!(decoder.decode(&[][..], Eos::new(true)).is_err());

        let mut decoder = U16beDecoder::new().maybe_eos();
        assert!(decoder.decode(&[][..], Eos::new(true)).is_ok());

        let mut decoder = U16beDecoder::new().maybe_eos();
        assert!(decoder.decode(&[1][..], Eos::new(false)).is_ok());
        assert!(decoder.decode(&[][..], Eos::new(true)).is_err());
    }
}

// TODO: delete or rename to `Peek`
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

    /// Takes ownership of this instance and returns the inner decoder.
    pub fn into_inner(self) -> D {
        self.inner
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

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        if self.item.is_none() {
            let size = track!(self.inner.decode(buf, eos))?;
            if self.inner.is_idle() {
                self.item = Some(track!(self.finish_decoding())?);
            }
            Ok(size)
        } else {
            Ok(0)
        }
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        let item = track_assert_some!(self.item.take(), ErrorKind::IncompleteItem);
        Ok(item)
    }

    fn requiring_bytes(&self) -> ByteCount {
        if self.item.is_some() {
            ByteCount::Finite(0)
        } else {
            self.inner.requiring_bytes()
        }
    }
}
