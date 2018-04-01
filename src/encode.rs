use {EncodeBuf, Error, Result};
use combinator::{EncoderChain, MapErr, Optional, Repeat, StartEncodingFrom};

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
    ///   - The output byte sequence has reached the end in the middle of a encoding process
    /// - `ErrorKind::Other`:
    ///   - Other errors has occurred
    fn encode(&mut self, buf: &mut EncodeBuf) -> Result<()>;

    /// Tries to start encoding the given item.
    ///
    /// If the encoding has no items to be encoded (i.e., `is_completed()` returns `true`) and
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

    /// Returns the number of bytes required to encode all the items in the encoder.
    ///
    /// If the encoder does not known the value, it will return `None`.
    ///
    /// If there is no items to be encoded, the encoder **should** return `Ok(0)`.
    fn requiring_bytes_hint(&self) -> Option<u64>;

    /// Returns `true` if there are no items to be encoded in the encoder, otherwise `false`.
    ///
    /// The default implementation returns the result of `self.requiring_bytes_hint() == Some(0)`.
    fn is_completed(&self) -> bool {
        self.requiring_bytes_hint() == Some(0)
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

    fn is_completed(&self) -> bool {
        (**self).is_completed()
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

pub trait EncodeExt: Encode + Sized {
    fn with_item(item: Self::Item) -> Result<Self>
    where
        Self: Default,
    {
        let mut this = Self::default();
        track!(this.start_encoding(item))?;
        Ok(this)
    }

    fn map_err<F, E>(self, f: F) -> MapErr<Self, F, E>
    where
        F: Fn(Error) -> E,
        Error: From<E>,
    {
        MapErr::new(self, f)
    }

    // TODO: map_from, try_map_from
    fn start_encoding_from<T, F>(self, f: F) -> StartEncodingFrom<Self, T, F>
    where
        F: Fn(T) -> Self::Item,
    {
        StartEncodingFrom::new(self, f)
    }

    fn chain<E: Encode>(self, other: E) -> EncoderChain<Self, E, Self::Item> {
        EncoderChain::new(self, other)
    }

    // TODO: rename
    fn repeat<I>(self) -> Repeat<Self, I>
    where
        I: Iterator<Item = Self::Item>,
    {
        Repeat::new(self)
    }

    // padding
    // max_bytes, length
    fn optional(self) -> Optional<Self> {
        Optional::new(self)
    }
}
impl<T: Encode> EncodeExt for T {}
