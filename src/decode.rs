use std;
use std::cmp;
use std::io::{self, Read};
use std::ops::Deref;

use {Error, ErrorKind, Result};
use combinator::{AndThen, Collect, DecoderChain, IgnoreRest, Length, Map, MapErr, Take, Validate};

/// This trait allows for decoding items from a byte sequence incrementally.
pub trait Decode {
    /// The type of items to be decoded.
    type Item;

    /// Consumes the given buffer (a part of a byte sequence), and decodes an item from it.
    ///
    /// If an item is successfully decoded, the decoder will return `Ok(Some(..))`.
    ///
    /// If the buffer does not contain enough bytes to decode the next item,
    /// the decoder will return `Ok(None)`.
    /// In this case, the decoder **must** consume all the bytes in the buffer.
    ///
    /// # Errors
    ///
    /// TODO
    ///
    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>>;

    /// Returns the lower bound of the number of bytes needed to decode the next item.
    ///
    /// If the decoder does not know the value, it will return `None`
    /// (e.g., null-terminated strings have no pre-estimable length).
    ///
    /// If the decoder returns `Some(0)`, it means one of the followings:
    /// - (a) There is an already decoded item
    ///   - The next invocation of `decode()` will return it without consuming any bytes
    /// - (b) There are no decodable items
    ///   - All decodable items have been decoded, and the decoder has no further works
    fn requiring_bytes_hint(&self) -> Option<u64>;
}
impl<D: ?Sized + Decode> Decode for Box<D> {
    type Item = D::Item;

    fn decode(&mut self, buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        (**self).decode(buf)
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

    fn decode(&mut self, _buf: &mut DecodeBuf) -> Result<Option<Self::Item>> {
        let item = track_assert_some!(self.0.take(), ErrorKind::DecoderTerminated);
        Ok(Some(item))
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        Some(0)
    }
}

/// An extension of `Decode` trait.
pub trait DecodeExt: Decode + Sized {
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

    /// Creates a decoder for modifying decoding errors produced by `self`.
    ///
    /// # Examples
    ///
    /// The following code shows the idiomatic way to track decoding errors:
    ///
    /// TODO: remove `no_run`
    ///
    /// ```no_run
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
    ///   [0] at src/bytes.rs:106
    ///   [1] at src/fixnum.rs:152
    ///   [2] at src/combinator.rs:69
    ///   [3] at src/decode.rs:11 -- oops!
    ///   [4] at src/decode.rs:17\n");
    /// # }
    /// ```
    fn map_err<F>(self, f: F) -> MapErr<Self, F>
    where
        F: Fn(Error) -> Error,
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
    fn take(self, n: usize) -> Take<Self> {
        Take::new(self, n)
    }

    // fn present(self, b: bool) -> Option<Self> {
    //     if b {
    //         Some(self)
    //     } else {
    //         None
    //     }
    // }

    // or skip_remainings
    fn ignore_rest(self) -> IgnoreRest<Self> {
        IgnoreRest::new(self)
    }

    fn validate<F, E>(self, f: F) -> Validate<Self, F, E>
    where
        F: for<'a> Fn(&'a Self::Item) -> std::result::Result<(), E>,
        Error: From<E>,
    {
        Validate::new(self, f)
    }

    // TODO: min, max
    // TODO: max_bytes
}
impl<T: Decode> DecodeExt for T {}

/// Decoding buffer.
///
/// A `DecodeBuf` represents a slice of a byte sequence.
/// Decoders consume consecutive buffers and decode items.
///
/// In addition, `DecodeBuf` optionally provides the number of bytes remaining in the sequence to decoders.
#[derive(Debug)]
pub struct DecodeBuf<'a> {
    buf: &'a [u8],
    offset: usize,
    remaining_bytes: Option<u64>,
}
impl DecodeBuf<'static> {
    /// Makes a `DecodeBuf` instance that represents the end-of-sequence
    /// (i.e., zero length slice and zero remaining bytes).
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::DecodeBuf;
    ///
    /// let buf = DecodeBuf::eos();
    /// assert_eq!(buf.len(), 0);
    /// assert_eq!(buf.remaining_bytes(), Some(0));
    /// assert!(buf.is_eos());
    /// ```
    pub fn eos() -> Self {
        DecodeBuf {
            buf: &[],
            offset: 0,
            remaining_bytes: Some(0),
        }
    }
}
impl<'a> DecodeBuf<'a> {
    /// Makes a new `DecodeBuf` instance without remaining bytes information.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::DecodeBuf;
    ///
    /// let buf = DecodeBuf::new(b"foo");
    /// assert_eq!(buf.as_ref(), b"foo");
    /// assert_eq!(buf.remaining_bytes(), None);
    /// ```
    pub fn new(buf: &'a [u8]) -> Self {
        DecodeBuf {
            buf,
            offset: 0,
            remaining_bytes: None,
        }
    }

    /// Makes a new `DecodeBuf` instance with the given number of remaining bytes.
    ///
    /// ```
    /// use bytecodec::DecodeBuf;
    ///
    /// let buf = DecodeBuf::with_remaining_bytes(b"foo", 10);
    /// assert_eq!(buf.as_ref(), b"foo");
    /// assert_eq!(buf.remaining_bytes(), Some(10));
    /// ```
    pub fn with_remaining_bytes(buf: &'a [u8], remaining_bytes: u64) -> Self {
        DecodeBuf {
            buf,
            offset: 0,
            remaining_bytes: Some(remaining_bytes),
        }
    }

    /// Returns the number of bytes remaining in the sequence.
    ///
    /// Note that it does not contain the number of the bytes in this buffer.
    ///
    /// `None` means there is no knowledge about the length of the byte sequence.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::DecodeBuf;
    ///
    /// let mut buf = DecodeBuf::with_remaining_bytes(b"foo", 10);
    /// assert_eq!(buf.len(), 3);
    /// assert_eq!(buf.remaining_bytes(), Some(10));
    ///
    /// buf.consume(2).unwrap();
    /// assert_eq!(buf.len(), 1);
    /// assert_eq!(buf.remaining_bytes(), Some(10));
    /// ```
    pub fn remaining_bytes(&self) -> Option<u64> {
        self.remaining_bytes
    }

    /// Returns `true` if it reaches the end of the sequence(EOS), otherwise `false`.
    ///
    /// Operationally, "EOS" means both the length of the current buffer and the remaining bytes are `0`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::DecodeBuf;
    ///
    /// // The buffer is not empty (not EOS)
    /// let mut buf = DecodeBuf::with_remaining_bytes(b"foo", 0);
    /// assert_eq!(buf.len(), 3);
    /// assert_eq!(buf.remaining_bytes(), Some(0));
    /// assert!(!buf.is_eos());
    ///
    /// // The buffer and remaining bytes are empty (EOS)
    /// buf.consume(3).unwrap();
    /// assert!(buf.is_empty());
    /// assert!(buf.is_eos());
    ///
    /// // There are some remaining bytes (not EOS)
    /// let buf = DecodeBuf::with_remaining_bytes(b"", 10);
    /// assert_eq!(buf.remaining_bytes(), Some(10));
    /// assert!(!buf.is_eos());
    ///
    /// // The number of remaining bytes is unknown (can not judge it is EOS)
    /// let buf = DecodeBuf::new(b"");
    /// assert_eq!(buf.remaining_bytes(), None);
    /// assert!(!buf.is_eos());
    /// ```
    pub fn is_eos(&self) -> bool {
        self.is_empty() && self.remaining_bytes().map_or(false, |n| n == 0)
    }

    /// Consumes the specified number of the bytes from the beginning of this buffer.
    ///
    /// Note the invocation of the `Read::read()` method automatically consumes the read bytes.
    ///
    /// # Errors
    ///
    /// If `size` exceeds the length of the buffer, it will return an `ErrorKind::InvalidInput` error.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Read;
    /// use bytecodec::DecodeBuf;
    ///
    /// let mut buf = DecodeBuf::new(b"foo");
    /// assert_eq!(buf.as_ref(), b"foo");
    ///
    /// buf.consume(1).unwrap();
    /// assert_eq!(buf.as_ref(), b"oo");
    ///
    /// buf.read_to_end(&mut Vec::new()).unwrap();
    /// assert_eq!(buf.as_ref(), b"");
    ///
    /// assert!(buf.consume(1).is_err());
    /// ```
    pub fn consume(&mut self, size: usize) -> Result<()> {
        track_assert!(self.offset + size <= self.buf.len(), ErrorKind::InvalidInput;
                      self.offset, size, self.buf.len());
        self.offset += size;
        Ok(())
    }
}
impl<'a> AsRef<[u8]> for DecodeBuf<'a> {
    fn as_ref(&self) -> &[u8] {
        &self.buf[self.offset..]
    }
}
impl<'a> Deref for DecodeBuf<'a> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
impl<'a> Read for DecodeBuf<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let size = cmp::min(self.len(), buf.len());
        (&mut buf[..size]).copy_from_slice(&self[..size]);
        self.offset += size;
        Ok(size)
    }
}
