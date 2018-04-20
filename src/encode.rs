use std;

use {ByteCount, Eos, Error, Result};
use combinator::{EncoderChain, Last, Length, MapErr, MapFrom, MaxBytes, Optional, Padding,
                 PreEncode, Repeat, Slice, TryMapFrom, WithPrefix};

/// This trait allows for encoding items into a byte sequence incrementally.
pub trait Encode {
    /// The type of items to be encoded.
    type Item;

    /// Encodes the items in the encoder and writes the encoded bytes to the given buffer.
    ///
    /// It returns the number of bytes written to the given buffer.
    ///
    /// If the encoded bytes are larger than the length of `buf`,
    /// the encoder must consume as many bytes in the buffer as possible.
    ///
    /// The completion of the encoding can be detected by using `is_idle` method.
    ///
    /// If `self.is_idle()` returns `false` but the number of written bytes in the last `encode` invocation
    /// is smaller than the length of `buf`, it means the encoder has been suspended its work in any reasons.
    /// In that case the encoder may require some instructions from clients to resume the work,
    /// but its concrete method is beyond the scope of this trait.
    ///
    /// The encoded bytes that could not be written to the given buffer is held by
    /// the encoder until the next invocation of the `encode` method.
    ///
    /// # Errors
    ///
    /// Encoders return the following kinds of errors as necessary:
    /// - `ErrorKind::InvalidInput`:
    ///   - An item that the encoder could not encode was passed
    /// - `ErrorKind::UnexpectedEos`:
    ///   - The output byte stream has reached the end in the middle of an encoding process
    /// - `ErrorKind::Other`:
    ///   - Other errors has occurred
    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize>;

    /// Tries to start encoding the given item.
    ///
    /// If the encoder has no items to be encoded and the passed item is valid, it must accept the item.
    ///
    /// # Errors
    ///
    /// - `ErrorKind::EncoderFull`:
    ///   - The encoder currently cannot accept any more items
    /// - `ErrorKind::InvalidInput`:
    ///   - An invalid item was passed
    /// - `ErrorKind::Other`:
    ///   - Other errors has occurred
    fn start_encoding(&mut self, item: Self::Item) -> Result<()>;

    /// Returns `true` if there are no items to be encoded in the encoder, otherwise `false`.
    fn is_idle(&self) -> bool;

    /// Returns the number of bytes required to encode all the items in the encoder.
    ///
    /// If there are no items to be encoded, the encoder must return `ByteCount::Finite(0)`.
    fn requiring_bytes(&self) -> ByteCount;
}
impl<'a, E: ?Sized + Encode> Encode for &'a mut E {
    type Item = E::Item;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        (**self).encode(buf, eos)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        (**self).start_encoding(item)
    }

    fn requiring_bytes(&self) -> ByteCount {
        (**self).requiring_bytes()
    }

    fn is_idle(&self) -> bool {
        (**self).is_idle()
    }
}
impl<E: ?Sized + Encode> Encode for Box<E> {
    type Item = E::Item;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        (**self).encode(buf, eos)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        (**self).start_encoding(item)
    }

    fn requiring_bytes(&self) -> ByteCount {
        (**self).requiring_bytes()
    }

    fn is_idle(&self) -> bool {
        (**self).is_idle()
    }
}

/// This trait indicates that the encoder always known the exact bytes required to encode remaining items.
pub trait ExactBytesEncode: Encode {
    /// Returns the exact number of bytes required to encode all the items remaining in the encoder.
    fn exact_requiring_bytes(&self) -> u64;
}
impl<E: ?Sized + ExactBytesEncode> ExactBytesEncode for Box<E> {
    fn exact_requiring_bytes(&self) -> u64 {
        (**self).exact_requiring_bytes()
    }
}

/// An extension of `Encode` trait.
pub trait EncodeExt: Encode + Sized {
    /// Creates a new encoder instance that has the given initial item.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::{Encode, EncodeExt};
    /// use bytecodec::fixnum::U8Encoder;
    /// use bytecodec::io::IoEncodeExt;
    ///
    /// let mut output = Vec::new();
    /// let mut encoder = U8Encoder::with_item(7).unwrap();
    /// encoder.encode_all(&mut output).unwrap();
    /// assert_eq!(output, [7]);
    /// assert!(encoder.is_idle());
    /// ```
    fn with_item(item: Self::Item) -> Result<Self>
    where
        Self: Default,
    {
        let mut this = Self::default();
        track!(this.start_encoding(item))?;
        Ok(this)
    }

    /// Creates an encoder for modifying encoding errors produced by `self`.
    ///
    /// # Examples
    ///
    /// The following code shows the idiomatic way to track encoding errors:
    ///
    /// ```
    /// extern crate bytecodec;
    /// #[macro_use]
    /// extern crate trackable;
    ///
    /// use bytecodec::{Encode, EncodeExt, Eos};
    /// use bytecodec::fixnum::U8Encoder;
    ///
    /// # fn main() {
    /// let encoder = U8Encoder::with_item(7).unwrap();
    /// let mut encoder = encoder.map_err(|e| track!(e, "oops!")); // or track_err!(encoder, "oops!")
    /// let error = track!(encoder.encode(&mut [][..], Eos::new(true))).err().unwrap();
    ///
    /// assert_eq!(error.to_string(), "\
    /// UnexpectedEos (cause; assertion failed: `!eos.is_reached()`; \
    ///                buf.len()=0, size=0, self.offset=0, b.as_ref().len()=1)
    /// HISTORY:
    ///   [0] at src/bytes.rs:54
    ///   [1] at src/fixnum.rs:113
    ///   [2] at src/encode.rs:11 -- oops!
    ///   [3] at src/encode.rs:12\n");
    /// # }
    /// ```
    fn map_err<F, E>(self, f: F) -> MapErr<Self, F, E>
    where
        F: Fn(Error) -> E,
        Error: From<E>,
    {
        MapErr::new(self, f)
    }

    /// Creates an encoder that converts items into ones that
    /// suited to the `self` encoder by calling the given function.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::{Encode, EncodeExt};
    /// use bytecodec::fixnum::U8Encoder;
    /// use bytecodec::io::IoEncodeExt;
    ///
    /// let mut output = Vec::new();
    /// let mut encoder = U8Encoder::new().map_from(|s: String| s.len() as u8);
    /// let item = "Hello World!".to_owned();
    /// encoder.start_encoding(item).unwrap();
    /// encoder.encode_all(&mut output).unwrap();
    /// assert_eq!(output, [12]);
    /// ```
    fn map_from<T, F>(self, f: F) -> MapFrom<Self, T, F>
    where
        F: Fn(T) -> Self::Item,
    {
        MapFrom::new(self, f)
    }

    /// Creates an encoder that tries to convert items into ones that
    /// suited to the `self` encoder by calling the given function.
    ///
    /// # Examples
    ///
    /// ```
    /// extern crate bytecodec;
    /// #[macro_use]
    /// extern crate trackable;
    ///
    /// use bytecodec::{Encode, EncodeExt, ErrorKind, Result};
    /// use bytecodec::fixnum::U8Encoder;
    /// use bytecodec::io::IoEncodeExt;
    ///
    /// # fn main() {
    /// let mut output = Vec::new();
    /// let mut encoder = U8Encoder::new().try_map_from(|s: String| -> Result<_> {
    ///     track_assert!(s.len() <= 0xFF, ErrorKind::InvalidInput);
    ///     Ok(s.len() as u8)
    /// });
    /// let item = "Hello World!".to_owned();
    /// encoder.start_encoding(item).unwrap();
    /// encoder.encode_all(&mut output).unwrap();
    /// assert_eq!(output, [12]);
    /// # }
    /// ```
    fn try_map_from<T, E, F>(self, f: F) -> TryMapFrom<Self, T, E, F>
    where
        F: Fn(T) -> std::result::Result<Self::Item, E>,
        Error: From<E>,
    {
        TryMapFrom::new(self, f)
    }

    /// Takes two encoders and creates a new encoder that encodes both items in sequence.
    ///
    /// Chains are started by calling `StartEncoderChain::chain` method.
    ///
    /// # Examples
    ///
    /// Encodes a length-prefixed UTF-8 string:
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
    fn chain<E: Encode>(self, other: E) -> EncoderChain<Self, E, Self::Item> {
        EncoderChain::new(self, other)
    }

    /// Creates an encoder that represents an optional encoder.
    ///
    /// It takes `Option<Self::Item>` items.
    /// If `Some(_)` is passed as an argument for `start_encoding` method, it will be encoded as ordinally.
    /// On the other hand, if `None` is passed, it will be ignored completely.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::{Encode, EncodeExt};
    /// use bytecodec::fixnum::U8Encoder;
    /// use bytecodec::io::IoEncodeExt;
    ///
    /// let mut output = Vec::new();
    /// let mut encoder = U8Encoder::new().optional();
    ///
    /// encoder.start_encoding(None).unwrap();
    /// encoder.encode_all(&mut output).unwrap();
    ///
    /// encoder.start_encoding(Some(9)).unwrap();
    /// encoder.encode_all(&mut output).unwrap();
    ///
    /// assert_eq!(output, [9]);
    /// ```
    fn optional(self) -> Optional<Self> {
        Optional::new(self)
    }

    /// Creates an encoder that will fail if the number of encoded bytes of an item exceeds `n`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::{Encode, EncodeExt, ErrorKind};
    /// use bytecodec::bytes::Utf8Encoder;
    /// use bytecodec::io::IoEncodeExt;
    ///
    /// let mut output = Vec::new();
    /// let mut encoder = Utf8Encoder::new().max_bytes(3);
    ///
    /// encoder.start_encoding("foo").unwrap(); // OK
    /// encoder.encode_all(&mut output).unwrap();
    /// assert_eq!(output, b"foo");
    ///
    /// encoder.start_encoding("hello").unwrap(); // Error
    /// let error = encoder.encode_all(&mut output).err().unwrap();
    /// assert_eq!(*error.kind(), ErrorKind::InvalidInput);
    /// ```
    fn max_bytes(self, n: u64) -> MaxBytes<Self> {
        MaxBytes::new(self, n)
    }

    /// Creates an encoder that required to encode each item exactly at the specified number of bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::{Encode, EncodeExt, ErrorKind};
    /// use bytecodec::bytes::Utf8Encoder;
    /// use bytecodec::io::IoEncodeExt;
    ///
    /// let mut output = Vec::new();
    /// let mut encoder = Utf8Encoder::new().length(3);
    /// encoder.start_encoding("hey").unwrap(); // OK
    /// encoder.encode_all(&mut output).unwrap();
    /// assert_eq!(output, b"hey");
    ///
    /// let mut encoder = Utf8Encoder::new().length(3);
    /// encoder.start_encoding("hello").unwrap(); // Error (too long)
    /// let error = encoder.encode_all(&mut output).err().unwrap();
    /// assert_eq!(*error.kind(), ErrorKind::UnexpectedEos);
    ///
    /// let mut encoder = Utf8Encoder::new().length(3);
    /// encoder.start_encoding("hi").unwrap(); // Error (too short)
    /// let error = encoder.encode_all(&mut output).err().unwrap();
    /// assert_eq!(*error.kind(), ErrorKind::InvalidInput);
    /// ```
    fn length(self, n: u64) -> Length<Self> {
        Length::new(self, n)
    }

    /// Creates an encoder that keeps writing padding byte until it reaches EOS
    /// after encoding of `self`'s item has been completed.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::{Encode, EncodeExt, ErrorKind};
    /// use bytecodec::fixnum::U8Encoder;
    /// use bytecodec::io::IoEncodeExt;
    ///
    /// let mut output = Vec::new();
    /// let mut encoder = U8Encoder::new().padding(9).length(3);
    /// encoder.start_encoding(3).unwrap();
    /// encoder.encode_all(&mut output).unwrap();
    /// assert_eq!(output, [3, 9, 9]);
    /// ```
    fn padding(self, padding_byte: u8) -> Padding<Self> {
        Padding::new(self, padding_byte)
    }

    /// Creates an encoder that repeats encoding of `Self::Item`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::{Encode, EncodeExt, ErrorKind};
    /// use bytecodec::fixnum::U8Encoder;
    /// use bytecodec::io::IoEncodeExt;
    ///
    /// let mut output = Vec::new();
    /// let mut encoder = U8Encoder::new().repeat();
    /// encoder.start_encoding(0..4).unwrap();
    /// encoder.encode_all(&mut output).unwrap();
    /// assert_eq!(output, [0, 1, 2, 3]);
    /// ```
    fn repeat<I>(self) -> Repeat<Self, I>
    where
        I: Iterator<Item = Self::Item>,
    {
        Repeat::new(self)
    }

    /// Creates an encoder that has a prefixed item encoded by `E`.
    ///
    /// # Examples
    ///
    /// Encodes a length prefixed UTF-8 string:
    ///
    /// ```
    /// use bytecodec::{Encode, EncodeExt, ExactBytesEncode};
    /// use bytecodec::bytes::Utf8Encoder;
    /// use bytecodec::fixnum::U8Encoder;
    /// use bytecodec::io::IoEncodeExt;
    ///
    /// let mut output = Vec::new();
    /// let mut encoder =
    ///     Utf8Encoder::new().with_prefix(U8Encoder::new(), |body| body.exact_requiring_bytes() as u8);
    /// encoder.start_encoding("foo").unwrap();
    /// encoder.encode_all(&mut output).unwrap();
    /// assert_eq!(output, [3, b'f', b'o', b'o']);
    /// ```
    fn with_prefix<E, F>(self, prefix: E, f: F) -> WithPrefix<Self, E, F>
    where
        F: Fn(&Self) -> E::Item,
        E: Encode,
    {
        WithPrefix::new(self, prefix, f)
    }

    /// Creates an encoder that pre-encodes items when `start_encoding` method is called.
    ///
    /// Although the number of memory copies increases,
    /// pre-encoding will enable to acquire the exact size of encoded items.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::{Encode, EncodeExt, ExactBytesEncode};
    /// use bytecodec::fixnum::U8Encoder;
    /// use bytecodec::io::IoEncodeExt;
    ///
    /// let mut output = Vec::new();
    /// let mut encoder =
    ///     U8Encoder::new()
    ///         .repeat()
    ///         .pre_encode()
    ///         .with_prefix(U8Encoder::new(), |body| body.exact_requiring_bytes() as u8);
    ///
    /// encoder.start_encoding(0..3).unwrap();
    /// encoder.encode_all(&mut output).unwrap();
    /// assert_eq!(output, [3, 0, 1, 2]);
    /// ```
    fn pre_encode(self) -> PreEncode<Self> {
        PreEncode::new(self)
    }

    /// Creates an encoder that makes it possible to slice the encoded byte sequence in arbitrary units.
    ///
    /// Slicing encoded byte sequences makes it easier to multiplex them into a single sequence.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::{Encode, EncodeExt, Eos};
    /// use bytecodec::bytes::Utf8Encoder;
    ///
    /// let mut encoder = Utf8Encoder::new().slice();
    /// encoder.start_encoding("foobarbaz").unwrap();
    ///
    /// let eos = Eos::new(true);
    /// let mut output = [0; 9];
    /// let mut offset = 0;
    ///
    /// encoder.set_consumable_bytes(3);
    /// offset += encoder.encode(&mut output[offset..], eos).unwrap();
    /// assert_eq!(offset, 3);
    /// assert_eq!(encoder.is_idle(), false);
    /// assert_eq!(encoder.consumable_bytes(), 0);
    ///
    /// offset += encoder.encode(&mut output[offset..], eos).unwrap();
    /// assert_eq!(offset, 3);
    ///
    /// encoder.set_consumable_bytes(6);
    /// offset += encoder.encode(&mut output[offset..], eos).unwrap();
    /// assert_eq!(offset, 9);
    /// assert_eq!(encoder.is_idle(), true);
    /// assert_eq!(output.as_ref(), b"foobarbaz");
    /// ```
    fn slice(self) -> Slice<Self> {
        Slice::new(self)
    }

    /// Creates an encoder that cannot accept any more items.
    fn last(self) -> Last<Self> {
        Last::new(self)
    }
}
impl<T: Encode> EncodeExt for T {}
