use std;

use {EncodeBuf, Error, Result};
use combinator::{EncoderChain, Length, MapErr, MapFrom, MaxBytes, Optional, Padding, Repeat,
                 TryMapFrom, WithPrefix};

/// This trait allows for encoding items into a byte sequence incrementally.
pub trait Encode {
    /// The type of items to be encoded.
    type Item;

    /// Encodes the current item and writes the encoded bytes to the given buffer as many as possible.
    ///
    /// If the encoded bytes are larger than the length of `buf`,
    /// the encoder **must** consume all the bytes in the buffer.
    /// The encoded bytes that could not be written is held by the encoder
    /// until the next invocation of the `encode()` method.
    ///
    /// # Errors
    ///
    /// Encoders return the following kinds of errors as necessary:
    /// - `ErrorKind::InvalidInput`:
    ///   - An item that the encoder could not encode was passed
    /// - `ErrorKind::UnexpectedEos`:
    ///   - The output byte sequence has reached the end in the middle of an encoding process
    /// - `ErrorKind::Other`:
    ///   - Other errors has occurred
    fn encode(&mut self, buf: &mut EncodeBuf) -> Result<()>;

    /// Tries to start encoding the given item.
    ///
    /// If the encoding has no items to be encoded (i.e., `is_idle()` returns `true`) and
    /// the item is valid, the encoder **should** accept it.
    ///
    /// # Errors
    ///
    /// - `ErrorKind::EncoderFull`:
    ///   - The encoder cannot accept any more items
    /// - `ErrorKind::InvalidInput`:
    ///   - An invalid item was passed
    /// - `ErrorKind::Other`:
    ///   - Other errors has occurred
    fn start_encoding(&mut self, item: Self::Item) -> Result<()>;

    /// Returns `true` if there are no items to be encoded in the encoder, otherwise `false`.
    fn is_idle(&self) -> bool;

    /// Returns the number of bytes required to encode all the items in the encoder.
    ///
    /// If the encoder does not known the value, it will return `None`.
    ///
    /// If there is no items to be encoded, the encoder **should** return `Ok(0)`.
    ///
    /// The default implementation returns `Some(0)` if the encoder is idle, otherwise `None`.
    fn requiring_bytes_hint(&self) -> Option<u64> {
        if self.is_idle() {
            Some(0)
        } else {
            None
        }
    }
}
impl<E: ?Sized + Encode> Encode for Box<E> {
    type Item = E::Item;

    fn encode(&mut self, buf: &mut EncodeBuf) -> Result<()> {
        (**self).encode(buf)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        (**self).start_encoding(item)
    }

    fn requiring_bytes_hint(&self) -> Option<u64> {
        (**self).requiring_bytes_hint()
    }

    fn is_idle(&self) -> bool {
        (**self).is_idle()
    }
}

/// This trait indicates that the encoder always known the exact bytes required to encode remaining items.
///
/// By implementing this trait, the user of those encoders can implement length-prefixed protocols easily.
pub trait ExactBytesEncode: Encode {
    /// Returns the number of bytes required to encode all the items in the encoder.
    fn requiring_bytes(&self) -> u64 {
        self.requiring_bytes_hint()
            .expect("Must be a `Some(_)` value")
    }
}
impl<E: ?Sized + ExactBytesEncode> ExactBytesEncode for Box<E> {
    fn requiring_bytes(&self) -> u64 {
        (**self).requiring_bytes()
    }
}

/// An extension of `Encode` trait.
pub trait EncodeExt: Encode + Sized {
    /// Creates a new encoder instance that has the given initial item.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::{Encode, EncodeBuf, EncodeExt};
    /// use bytecodec::fixnum::U8Encoder;
    ///
    /// let mut output = [0; 1];
    /// let mut encoder = U8Encoder::with_item(7).unwrap();
    /// {
    ///     encoder.encode(&mut EncodeBuf::new(&mut output)).unwrap();
    /// }
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
    /// use bytecodec::{Encode, EncodeBuf, EncodeExt};
    /// use bytecodec::fixnum::U8Encoder;
    ///
    /// # fn main() {
    /// let mut buf = EncodeBuf::with_eos(&mut [][..], true); // Empty and EOS buffer
    ///
    /// let encoder = U8Encoder::with_item(7).unwrap();
    /// let mut encoder = encoder.map_err(|e| track!(e, "oops!")); // or track_err!(encoder, "oops!")
    /// let error = track!(encoder.encode(&mut buf)).err().unwrap();
    ///
    /// assert_eq!(error.to_string(), "\
    /// UnexpectedEos (cause; assertion failed: `!buf.is_eos()`; self.offset=0, b.as_ref().len()=1)
    /// HISTORY:
    ///   [0] at src/bytes.rs:55
    ///   [1] at src/fixnum.rs:117
    ///   [2] at src/encode.rs:13 -- oops!
    ///   [3] at src/encode.rs:14\n");
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
    /// use bytecodec::{Encode, EncodeBuf, EncodeExt};
    /// use bytecodec::fixnum::U8Encoder;
    ///
    /// let mut output = [0; 1];
    /// let mut encoder = U8Encoder::new().map_from(|s: String| s.len() as u8);
    /// {
    ///     let item = "Hello World!".to_owned();
    ///     encoder.start_encoding(item).unwrap();
    ///     encoder.encode(&mut EncodeBuf::new(&mut output)).unwrap();
    /// }
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
    /// use bytecodec::{Encode, EncodeBuf, EncodeExt, ErrorKind, Result};
    /// use bytecodec::fixnum::U8Encoder;
    ///
    /// # fn main() {
    /// let mut output = [0; 1];
    /// let mut encoder = U8Encoder::new().try_map_from(|s: String| -> Result<_> {
    ///     track_assert!(s.len() <= 0xFF, ErrorKind::InvalidInput);
    ///     Ok(s.len() as u8)
    /// });
    /// {
    ///     let item = "Hello World!".to_owned();
    ///     encoder.start_encoding(item).unwrap();
    ///     encoder.encode(&mut EncodeBuf::new(&mut output)).unwrap();
    /// }
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
    /// use bytecodec::{Encode, EncodeBuf, EncodeExt};
    /// use bytecodec::fixnum::U8Encoder;
    ///
    /// let mut output = [0; 1];
    /// let mut encoder = U8Encoder::new().optional();
    /// {
    ///     let mut buf = EncodeBuf::new(&mut output);
    ///     assert_eq!(buf.len(), 1);
    ///
    ///     encoder.start_encoding(None).unwrap();
    ///     encoder.encode(&mut buf).unwrap();
    ///     assert_eq!(buf.len(), 1);
    ///
    ///     encoder.start_encoding(Some(9)).unwrap();
    ///     encoder.encode(&mut buf).unwrap();
    ///     assert_eq!(buf.len(), 0);
    /// }
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
    /// use bytecodec::{Encode, EncodeBuf, EncodeExt, ErrorKind};
    /// use bytecodec::bytes::Utf8Encoder;
    ///
    /// let mut output = [0; 3];
    /// let mut encoder = Utf8Encoder::new().max_bytes(3);
    ///
    /// {
    ///     encoder.start_encoding("foo").unwrap(); // OK
    ///     encoder.encode(&mut EncodeBuf::new(&mut output)).unwrap();
    /// }
    /// assert_eq!(output.as_ref(), b"foo");
    ///
    /// {
    ///     encoder.start_encoding("hello").unwrap(); // Error
    ///     let error = encoder.encode(&mut EncodeBuf::new(&mut output)).err().unwrap();
    ///     assert_eq!(*error.kind(), ErrorKind::InvalidInput);
    /// }
    /// ```
    fn max_bytes(self, n: u64) -> MaxBytes<Self> {
        MaxBytes::new(self, n)
    }

    /// Creates an encoder that required to encode each item exactly at the specified number of bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::{Encode, EncodeBuf, EncodeExt, ErrorKind};
    /// use bytecodec::bytes::Utf8Encoder;
    ///
    /// let mut output = [0; 4];
    /// {
    ///     let mut encoder = Utf8Encoder::new().length(3);
    ///     encoder.start_encoding("hey").unwrap(); // OK
    ///     encoder.encode(&mut EncodeBuf::new(&mut output)).unwrap();
    /// }
    /// assert_eq!(output.as_ref(), b"hey\x00");
    ///
    /// {
    ///     let mut encoder = Utf8Encoder::new().length(3);
    ///     encoder.start_encoding("hello").unwrap(); // Error (too long)
    ///     let error = encoder.encode(&mut EncodeBuf::new(&mut output)).err().unwrap();
    ///     assert_eq!(*error.kind(), ErrorKind::UnexpectedEos);
    /// }
    ///
    /// {
    ///     let mut encoder = Utf8Encoder::new().length(3);
    ///     encoder.start_encoding("hi").unwrap(); // Error (too short)
    ///     let error = encoder.encode(&mut EncodeBuf::new(&mut output)).err().unwrap();
    ///     assert_eq!(*error.kind(), ErrorKind::InvalidInput);
    /// }
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
    /// use bytecodec::{Encode, EncodeBuf, EncodeExt, ErrorKind};
    /// use bytecodec::fixnum::U8Encoder;
    ///
    /// let mut output = [0; 4];
    /// {
    ///     let mut encoder = U8Encoder::new().padding(9).length(3);
    ///     encoder.start_encoding(3).unwrap();
    ///     encoder.encode(&mut EncodeBuf::new(&mut output)).unwrap();
    /// }
    /// assert_eq!(output.as_ref(), [3, 9, 9, 0]);
    /// ```
    fn padding(self, padding_byte: u8) -> Padding<Self> {
        Padding::new(self, padding_byte)
    }

    /// Creates an encoder that repeats encoding of `Self::Item`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::{Encode, EncodeBuf, EncodeExt, ErrorKind};
    /// use bytecodec::fixnum::U8Encoder;
    ///
    /// let mut output = [0; 4];
    /// {
    ///     let mut encoder = U8Encoder::new().repeat();
    ///     encoder.start_encoding(0..4).unwrap();
    ///     encoder.encode(&mut EncodeBuf::new(&mut output)).unwrap();
    /// }
    /// assert_eq!(output.as_ref(), [0, 1, 2, 3]);
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
    /// use bytecodec::{Encode, EncodeBuf, EncodeExt, ExactBytesEncode};
    /// use bytecodec::bytes::Utf8Encoder;
    /// use bytecodec::fixnum::U8Encoder;
    ///
    /// let mut output = [0; 4];
    /// {
    ///     let mut encoder =
    ///         Utf8Encoder::new().with_prefix(U8Encoder::new(), |body| body.requiring_bytes() as u8);
    ///     encoder.start_encoding("foo").unwrap();
    ///     encoder.encode(&mut EncodeBuf::new(&mut output)).unwrap();
    /// }
    /// assert_eq!(output.as_ref(), [3, b'f', b'o', b'o']);
    /// ```
    fn with_prefix<E, F>(self, prefix: E, f: F) -> WithPrefix<Self, E, F>
    where
        F: Fn(&Self) -> E::Item,
        E: Encode,
    {
        WithPrefix::new(self, prefix, f)
    }
}
impl<T: Encode> EncodeExt for T {}
