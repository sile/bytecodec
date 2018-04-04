use std;

use {Eos, Error, ErrorKind, Result};
use combinator::{AndThen, Assert, Collect, DecoderChain, Length, Map, MapErr, MaxBytes, Omit,
                 SkipRemaining, Take, TryMap};

/// This trait allows for decoding items from a byte sequence incrementally.
pub trait Decode {
    /// The type of items to be decoded.
    type Item;

    /// Consumes the given buffer (a part of a byte sequence), and decodes an item from it.
    ///
    /// TODO: update doc
    ///
    /// If an item is successfully decoded, the decoder will return `Ok(Some(..))`.
    ///
    /// If the buffer does not contain enough bytes to decode the next item,
    /// the decoder will return `Ok(None)`.
    /// In this case, the decoder **must** consume all the bytes in the buffer.
    ///
    /// # Errors
    ///
    /// Decoders return the following kinds of errors as necessary:
    /// - `ErrorKind::DecoderTerminated`:
    ///   - If all decodable items have been decoded,
    ///     the decoder **must** return this kind of error when `decode()` method is called.
    /// - `ErrorKind::UnexpectedEos`:
    ///   - `DecodeBuf::is_eos()` returns `true` despite of
    ///     the decoder requires more bytes to decode the next item.
    /// - `ErrorKind::InvalidInput`:
    ///   - Decoded items have invalid values
    ///   - Invalid parameters were given to decoders
    /// - `ErrorKind::Error`:
    ///   - Other errors
    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)>;

    /// Returns `true` if the decoder cannot decode items anymore, otherwise `false`.
    ///
    /// If it returns `true`, the next invocation of `decode` method
    /// **must** return an `ErrorKind::DecoderTerminated` error.
    fn has_terminated(&self) -> bool;

    /// Returns the lower bound of the number of bytes needed to decode the next item.
    ///
    /// If the decoder does not know the value, it will return `None`
    /// (e.g., null-terminated strings have no pre-estimable length).
    ///
    /// If the decoder returns `Some(0)`, it means one of the followings:
    /// - (a) There is an already decoded item
    ///   - The next invocation of `decode()` will return it without consuming any bytes
    /// - (b) There are no decodable items
    ///   - All decodable items have been decoded, and the decoder cannot do any further works
    ///
    /// The default implementation returns `Some(0)` if the decoder has terminated, otherwise `None`.
    fn requiring_bytes_hint(&self) -> Option<u64> {
        if self.has_terminated() {
            Some(0)
        } else {
            None
        }
    }
}
impl<'a, D: ?Sized + Decode> Decode for &'a mut D {
    type Item = D::Item;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        (**self).decode(buf, eos)
    }

    fn has_terminated(&self) -> bool {
        (**self).has_terminated()
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        (**self).requiring_bytes_hint()
    }
}
impl<D: ?Sized + Decode> Decode for Box<D> {
    type Item = D::Item;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        (**self).decode(buf, eos)
    }

    fn has_terminated(&self) -> bool {
        (**self).has_terminated()
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        (**self).requiring_bytes_hint()
    }
}

/// `DecodedValue` represents a decoded item.
///
/// It does not consume any bytes, and returns the given item when `decode()` was first called.
///
/// # Examples
///
/// ```
/// use bytecodec::{Decode, DecodeBuf, DecodedValue};
///
/// let mut decoder = DecodedValue::new(10);
///
/// let mut input = DecodeBuf::new(b"foo");
/// let item = decoder.decode(&mut input).unwrap();
/// assert_eq!(item, Some(10));
/// assert_eq!(input.len(), 3);
/// ```
#[derive(Debug)]
pub struct DecodedValue<T>(Option<T>);
impl<T> DecodedValue<T> {
    /// Makes a new `DecodedValue` instance.
    pub fn new(value: T) -> Self {
        DecodedValue(Some(value))
    }
}
impl<T> Decode for DecodedValue<T> {
    type Item = T;

    fn decode(&mut self, _buf: &[u8], _eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        let item = track_assert_some!(self.0.take(), ErrorKind::DecoderTerminated);
        Ok((0, Some(item)))
    }

    fn has_terminated(&self) -> bool {
        self.0.is_none()
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        Some(0)
    }
}

/// An extension of `Decode` trait.
pub trait DecodeExt: Decode + Sized {
    /// Returns a mutable reference to the decoder.
    fn by_ref(&mut self) -> &mut Self {
        self
    }

    /// Creates a decoder that converts decoded values by calling the given function.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::{Decode, DecodeBuf, DecodeExt};
    /// use bytecodec::fixnum::U8Decoder;
    ///
    /// let mut decoder = U8Decoder::new().map(|b| b * 2);
    /// let item = decoder.decode(&mut DecodeBuf::new(&[10][..])).unwrap();
    /// assert_eq!(item, Some(20));
    /// ```
    fn map<T, F>(self, f: F) -> Map<Self, T, F>
    where
        F: Fn(Self::Item) -> T,
    {
        Map::new(self, f)
    }

    /// Creates a decoder that tries to convert decoded values by calling the given function.
    ///
    /// # Examples
    ///
    /// ```
    /// extern crate bytecodec;
    /// #[macro_use]
    /// extern crate trackable;
    ///
    /// use bytecodec::{Decode, DecodeBuf, DecodeExt, ErrorKind, Result};
    /// use bytecodec::fixnum::U8Decoder;
    ///
    /// # fn main() {
    /// let mut decoder = U8Decoder::new().try_map(|b| -> Result<_> {
    ///     track_assert_ne!(b, 0, ErrorKind::InvalidInput);
    ///     Ok(b * 2)
    /// });
    /// let mut input = DecodeBuf::new(&[0, 4][..]);
    ///
    /// let error = decoder.decode(&mut input).err().unwrap();
    /// assert_eq!(*error.kind(), ErrorKind::InvalidInput);
    ///
    /// let item = decoder.decode(&mut input).unwrap();
    /// assert_eq!(item, Some(8));
    /// # }
    /// ```
    fn try_map<F, T, E>(self, f: F) -> TryMap<Self, F, T, E>
    where
        F: Fn(Self::Item) -> std::result::Result<T, E>,
        Error: From<E>,
    {
        TryMap::new(self, f)
    }

    /// Creates a decoder for modifying decoding errors produced by `self`.
    ///
    /// # Examples
    ///
    /// The following code shows the idiomatic way to track decoding errors:
    ///
    /// ```
    /// extern crate bytecodec;
    /// #[macro_use]
    /// extern crate trackable;
    ///
    /// use bytecodec::{Decode, DecodeBuf, DecodeExt};
    /// use bytecodec::fixnum::U16beDecoder;
    ///
    /// # fn main() {
    /// let mut decoder =
    ///     U16beDecoder::new().map_err(|e| track!(e, "oops!"));
    ///     // or `track_err!(U16beDecoder::new(), "oops!")`
    ///
    /// let mut input =
    ///     DecodeBuf::with_remaining_bytes(&[10][..], 0); // Insufficient bytes
    ///
    /// let error = track!(decoder.decode(&mut input)).err().unwrap();
    ///
    /// assert_eq!(error.to_string(), "\
    /// UnexpectedEos (cause; assertion failed: `!buf.is_eos()`)
    /// HISTORY:
    ///   [0] at src/bytes.rs:143
    ///   [1] at src/fixnum.rs:195
    ///   [2] at src/decode.rs:11 -- oops!
    ///   [3] at src/decode.rs:17\n");
    /// # }
    /// ```
    fn map_err<F, E>(self, f: F) -> MapErr<Self, F, E>
    where
        F: Fn(Error) -> E,
        Error: From<E>,
    {
        MapErr::new(self, f)
    }

    /// Creates a decoder that enables conditional decoding.
    ///
    /// If the first item is successfully decoded,
    /// it will start decoding the second item by using the decoder returned by `f` function.
    ///
    /// # Examples
    ///
    /// Decodes a length-prefixed string:
    ///
    /// ```
    /// use bytecodec::{Decode, DecodeBuf, DecodeExt};
    /// use bytecodec::bytes::Utf8Decoder;
    /// use bytecodec::fixnum::U8Decoder;
    ///
    /// let mut decoder = U8Decoder::new().and_then(|len| Utf8Decoder::new().length(len as u64));
    /// let mut input = DecodeBuf::new(b"\x03foobar");
    ///
    /// let item = decoder.decode(&mut input).unwrap();
    /// assert_eq!(item, Some("foo".to_owned()));
    /// ```
    fn and_then<D, F>(self, f: F) -> AndThen<Self, D, F>
    where
        F: Fn(Self::Item) -> D,
        D: Decode,
    {
        AndThen::new(self, f)
    }

    /// Takes two decoders and creates a new decoder that decodes both items in sequence.
    ///
    /// Chains are started by calling `StartDecoderChain::chain` method.
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
    fn chain<D: Decode>(self, other: D) -> DecoderChain<Self, D, Self::Item> {
        DecoderChain::new(self, other)
    }

    /// Creates a decoder for collecting decoded items.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::{Decode, DecodeBuf, DecodeExt};
    /// use bytecodec::fixnum::U8Decoder;
    ///
    /// let mut decoder = U8Decoder::new().collect::<Vec<_>>();
    /// let mut input = DecodeBuf::with_remaining_bytes(b"foo", 0);
    ///
    /// let item = decoder.decode(&mut input).unwrap();
    /// assert_eq!(item, Some(vec![b'f', b'o', b'o']));
    /// ```
    fn collect<T>(self) -> Collect<Self, T>
    where
        T: Extend<Self::Item> + Default,
    {
        Collect::new(self)
    }

    /// Creates a decoder that consumes the specified number of bytes exactly.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::{Decode, DecodeBuf, DecodeExt, ErrorKind};
    /// use bytecodec::bytes::Utf8Decoder;
    ///
    /// let mut decoder = Utf8Decoder::new().length(3);
    /// let mut input = DecodeBuf::with_remaining_bytes(b"foobarba", 0);
    ///
    /// let item = decoder.decode(&mut input).unwrap();
    /// assert_eq!(item, Some("foo".to_owned()));
    ///
    /// let item = decoder.decode(&mut input).unwrap();
    /// assert_eq!(item, Some("bar".to_owned()));
    ///
    /// let error = decoder.decode(&mut input).err().unwrap();
    /// assert_eq!(*error.kind(), ErrorKind::UnexpectedEos);
    /// ```
    fn length(self, expected_bytes: u64) -> Length<Self> {
        Length::new(self, expected_bytes)
    }

    /// Creates a decoder that decodes `n` items by using `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::{Decode, DecodeBuf, DecodeExt};
    /// use bytecodec::fixnum::U8Decoder;
    ///
    /// let mut decoder = U8Decoder::new().take(2).collect::<Vec<_>>();
    /// let mut input = DecodeBuf::new(b"foo");
    ///
    /// let item = decoder.decode(&mut input).unwrap();
    /// assert_eq!(item, Some(vec![b'f', b'o']));
    /// ```
    fn take(self, n: usize) -> Take<Self> {
        Take::new(self, n)
    }

    /// Creates a decoder that will omit decoding items if `do_omit = true` is specified.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::{Decode, DecodeBuf, DecodeExt};
    /// use bytecodec::fixnum::U8Decoder;
    ///
    /// let mut input = DecodeBuf::new(b"foo");
    ///
    /// let mut decoder = U8Decoder::new().omit(true);
    /// let item = decoder.decode(&mut input).unwrap();
    /// assert_eq!(item, Some(None));
    ///
    /// let mut decoder = U8Decoder::new().omit(false);
    /// let item = decoder.decode(&mut input).unwrap();
    /// assert_eq!(item, Some(Some(b'f')));
    /// ```
    fn omit(self, do_omit: bool) -> Omit<Self> {
        Omit::new(self, do_omit)
    }

    /// Creates a decoder for skipping the remaining bytes in an input byte sequence
    /// after decoding an item by using `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::{Decode, DecodeBuf, DecodeExt};
    /// use bytecodec::fixnum::U8Decoder;
    ///
    /// let mut input = DecodeBuf::with_remaining_bytes(b"foo", 0);
    ///
    /// let mut decoder = U8Decoder::new().skip_remaining();
    /// let item = decoder.decode(&mut input).unwrap();
    /// assert_eq!(item, Some(b'f'));
    /// assert!(input.is_empty() && input.is_eos());
    /// ```
    fn skip_remaining(self) -> SkipRemaining<Self> {
        SkipRemaining::new(self)
    }

    /// Creates a decoder that will fail if the number of consumed bytes exceeds `bytes`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::{Decode, DecodeBuf, DecodeExt, ErrorKind};
    /// use bytecodec::bytes::Utf8Decoder;
    ///
    /// let mut decoder = Utf8Decoder::new().max_bytes(3);
    ///
    /// let mut input = DecodeBuf::with_remaining_bytes(b"foo", 0);
    /// let item = decoder.decode(&mut input).unwrap();
    /// assert_eq!(item, Some("foo".to_owned())); // OK
    ///
    /// let mut input = DecodeBuf::with_remaining_bytes(b"hello", 0);
    /// let error = decoder.decode(&mut input).err();
    /// assert_eq!(error.map(|e| *e.kind()), Some(ErrorKind::InvalidInput)); // Error
    /// ```
    fn max_bytes(self, bytes: u64) -> MaxBytes<Self> {
        MaxBytes::new(self, bytes)
    }

    /// Creates a decoder that will fail if the given assertion function returns `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::{Decode, DecodeBuf, DecodeExt, ErrorKind};
    /// use bytecodec::fixnum::U8Decoder;
    ///
    /// let mut decoder = U8Decoder::new().assert(|&b| b == 3);
    ///
    /// let mut input = DecodeBuf::new(&[3, 4][..]);
    /// let item = decoder.decode(&mut input).unwrap();
    /// assert_eq!(item, Some(3));
    ///
    /// let error = decoder.decode(&mut input).err();
    /// assert_eq!(error.map(|e| *e.kind()), Some(ErrorKind::InvalidInput));
    /// ```
    fn assert<F>(self, f: F) -> Assert<Self, F>
    where
        F: for<'a> Fn(&'a Self::Item) -> bool,
    {
        Assert::new(self, f)
    }
}
impl<T: Decode> DecodeExt for T {}
