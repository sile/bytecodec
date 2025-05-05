use crate::combinator::{
    AndThen, Collect, CollectN, Length, Map, MapErr, MaxBytes, MaybeEos, Omittable, Peekable,
    Slice, TryMap,
};
use crate::tuple::TupleDecoder;
use crate::{ByteCount, Eos, Error, ErrorKind, Result};

/// This trait allows for decoding items from a byte sequence incrementally.
pub trait Decode {
    /// The type of items to be decoded.
    type Item;

    /// Consumes the given buffer (a part of a byte sequence), and proceeds the decoding process.
    ///
    /// It returns the number of bytes consumed from the input buffer.
    ///
    /// If an item is completely decoded, the next invocation of `is_idle` method will return `true`.
    /// And if `is_idle` method returns `true`, `decode` method should consume no bytes.
    ///
    /// The decoder must consume as many bytes in the buffer as possible.
    /// If an item is not yet decoded but the number of consumed bytes in the last `decode` invocation
    /// is smaller than the length of `buf`, it means the decoder has been suspended its work in any reasons.
    /// In that case the decoder may require some instructions from clients to resume the work,
    /// but its concrete method is beyond the scope of this trait.
    ///
    /// # Errors
    ///
    /// The following errors may be returned by the decoder:
    /// - `ErrorKind::DecoderTerminated`:
    ///   - If all decodable items have been decoded,
    ///     the decoder must return this kind of error when `decode` method is called.
    /// - `ErrorKind::UnexpectedEos`:
    ///   - The invocation of `eos.is_reached()` returns `true` despite of
    ///     the decoder requires more bytes to decode the next item.
    /// - `ErrorKind::InvalidInput`:
    ///   - Decoded items have invalid values
    ///   - Invalid parameters were given to decoders
    /// - `ErrorKind::InconsistentState`:
    ///   - The state of the decoder bocame inconsistent
    ///   - This means the implementation contains a bug
    /// - `ErrorKind::Other`:
    ///   - Other errors
    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize>;

    /// Finishes the current decoding process and returns the decoded item.
    ///
    /// # Errors
    ///
    /// The following errors may be returned by the decoder:
    /// - `ErrorKind::IncompleteDecoding`:
    ///   - The decoding process has not been completed
    /// - `ErrorKind::DecoderTerminated`:
    ///   - The decoder has terminated (i.e., cannot decode any more items)
    /// - `ErrorKind::InconsistentState`:
    ///   - The state of the decoder bocame inconsistent
    ///   - This means the implementation contains a bug
    /// - `ErrorKind::Other`:
    ///   - Other errors
    fn finish_decoding(&mut self) -> Result<Self::Item>;

    /// Returns the lower bound of the number of bytes needed to decode the next item.
    ///
    /// If the decoder does not know the value, it will return `ByteCount::Unknown`
    /// (e.g., null-terminated strings have no pre-estimable length).
    ///
    /// If the decoder returns `ByteCount::Finite(0)`, it means one of the followings:
    /// - (a) There is an already decoded item
    ///   - The next invocation of `decode()` will return it without consuming any bytes
    /// - (b) There are no decodable items
    ///   - All decodable items have been decoded, and the decoder cannot do any further works
    ///   - In this case, the next invocation of `decode` method will fail.
    fn requiring_bytes(&self) -> ByteCount;

    /// Returns `true` if there are no items to be decoded by the decoder
    /// at the next invocation of `decode` method, otherwise `false`.
    ///
    /// Typically, `true` means the decoder already has a decoded item and
    /// it is waiting for `finish_decoding` to be called.
    ///
    /// The default implementation returns the result of `self.requiring_bytes() == ByteCount::Finite(0)`.
    fn is_idle(&self) -> bool {
        self.requiring_bytes() == ByteCount::Finite(0)
    }
}
impl<D: ?Sized + Decode> Decode for &mut D {
    type Item = D::Item;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        (**self).decode(buf, eos)
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        (**self).finish_decoding()
    }

    fn requiring_bytes(&self) -> ByteCount {
        (**self).requiring_bytes()
    }

    fn is_idle(&self) -> bool {
        (**self).is_idle()
    }
}
impl<D: ?Sized + Decode> Decode for Box<D> {
    type Item = D::Item;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        (**self).decode(buf, eos)
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        (**self).finish_decoding()
    }

    fn requiring_bytes(&self) -> ByteCount {
        (**self).requiring_bytes()
    }

    fn is_idle(&self) -> bool {
        (**self).is_idle()
    }
}

/// An extension of `Decode` trait.
pub trait DecodeExt: Decode + Sized {
    /// Creates a decoder that converts decoded values by calling the given function.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::{Decode, DecodeExt};
    /// use bytecodec::fixnum::U8Decoder;
    /// use bytecodec::io::IoDecodeExt;
    ///
    /// let mut decoder = U8Decoder::new().map(|b| b * 2);
    /// let item = decoder.decode_exact([10].as_ref()).unwrap();
    /// assert_eq!(item, 20);
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
    /// use bytecodec::{Decode, DecodeExt, ErrorKind, Result};
    /// use bytecodec::fixnum::U8Decoder;
    /// use bytecodec::io::IoDecodeExt;
    /// use trackable::{track, track_assert, track_assert_ne, track_panic};
    ///
    /// let mut decoder = U8Decoder::new().try_map(|b| -> Result<_> {
    ///     track_assert_ne!(b, 0, ErrorKind::InvalidInput);
    ///     Ok(b * 2)
    /// });
    ///
    /// let error = decoder.decode_exact([0].as_ref()).err().unwrap();
    /// assert_eq!(*error.kind(), ErrorKind::InvalidInput);
    ///
    /// let item = decoder.decode_exact([4].as_ref()).unwrap();
    /// assert_eq!(item, 8);
    /// ```
    fn try_map<T, E, F>(self, f: F) -> TryMap<Self, T, E, F>
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
    /// use bytecodec::{Decode, DecodeExt};
    /// use bytecodec::fixnum::U16beDecoder;
    /// use bytecodec::io::IoDecodeExt;
    /// use trackable::track;
    ///
    /// let mut decoder =
    ///     U16beDecoder::new().map_err(|e| track!(e, "oops!"));
    ///     // or `track_err!(U16beDecoder::new(), "oops!")`
    ///
    /// let input = [0]; // Insufficient bytes
    /// let error = track!(decoder.decode_exact(input.as_ref())).err().unwrap();
    ///
    /// assert_eq!(error.to_string(), "\
    /// UnexpectedEos (cause; assertion failed: `!eos.is_reached()`; \
    ///                self.offset=1, self.bytes.as_ref().len()=2)
    /// HISTORY:
    ///   [0] at src/bytes.rs:152
    ///   [1] at src/fixnum.rs:200
    ///   [2] at src/decode.rs:10 -- oops!
    ///   [3] at src/io.rs:45
    ///   [4] at src/decode.rs:14\n");
    /// ```
    fn map_err<E, F>(self, f: F) -> MapErr<Self, E, F>
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
    /// use bytecodec::{Decode, DecodeExt};
    /// use bytecodec::bytes::Utf8Decoder;
    /// use bytecodec::fixnum::U8Decoder;
    /// use bytecodec::io::IoDecodeExt;
    ///
    /// let mut decoder = U8Decoder::new().and_then(|len| Utf8Decoder::new().length(len as u64));
    /// let item = decoder.decode_exact(b"\x03foobar".as_ref()).unwrap();
    /// assert_eq!(item, "foo");
    /// ```
    fn and_then<D, F>(self, f: F) -> AndThen<Self, D, F>
    where
        F: Fn(Self::Item) -> D,
        D: Decode,
    {
        AndThen::new(self, f)
    }

    /// Creates a decoder for collecting decoded items.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::{Decode, DecodeExt};
    /// use bytecodec::fixnum::U8Decoder;
    /// use bytecodec::io::IoDecodeExt;
    ///
    /// let mut decoder = U8Decoder::new().collect::<Vec<_>>();
    /// let item = decoder.decode_exact(b"foo".as_ref()).unwrap();
    /// assert_eq!(item, vec![b'f', b'o', b'o']);
    /// ```
    fn collect<T>(self) -> Collect<Self, T>
    where
        T: Extend<Self::Item> + Default,
    {
        Collect::new(self)
    }

    /// Creates a decoder that decodes `n` items by using `self` and collecting the result.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::{Decode, DecodeExt};
    /// use bytecodec::fixnum::U8Decoder;
    /// use bytecodec::io::IoDecodeExt;
    ///
    /// let mut decoder = U8Decoder::new().collectn::<Vec<_>>(2);
    /// let item = decoder.decode_exact(b"foo".as_ref()).unwrap();
    /// assert_eq!(item, vec![b'f', b'o']);
    /// ```
    fn collectn<T>(self, n: usize) -> CollectN<Self, T>
    where
        T: Extend<Self::Item> + Default,
    {
        CollectN::new(self, n)
    }

    /// Creates a decoder that consumes the specified number of bytes exactly.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::{Decode, DecodeExt, ErrorKind};
    /// use bytecodec::bytes::Utf8Decoder;
    /// use bytecodec::io::IoDecodeExt;
    ///
    /// let mut decoder = Utf8Decoder::new().length(3);
    /// let mut input = &b"foobarba"[..];
    ///
    /// let item = decoder.decode_exact(&mut input).unwrap();
    /// assert_eq!(item, "foo");
    ///
    /// let item = decoder.decode_exact(&mut input).unwrap();
    /// assert_eq!(item, "bar");
    ///
    /// let error = decoder.decode_exact(&mut input).err().unwrap();
    /// assert_eq!(*error.kind(), ErrorKind::UnexpectedEos);
    /// ```
    fn length(self, expected_bytes: u64) -> Length<Self> {
        Length::new(self, expected_bytes)
    }

    /// Creates a decoder that will omit decoding items if `do_omit = true` is specified.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::{Decode, DecodeExt};
    /// use bytecodec::fixnum::U8Decoder;
    /// use bytecodec::io::IoDecodeExt;
    ///
    /// let mut input = &b"foo"[..];
    ///
    /// let mut decoder = U8Decoder::new().omit(true);
    /// let item = decoder.decode_exact(&mut input).unwrap();
    /// assert_eq!(item, None);
    ///
    /// let mut decoder = U8Decoder::new().omit(false);
    /// let item = decoder.decode_exact(&mut input).unwrap();
    /// assert_eq!(item, Some(b'f'));
    /// ```
    fn omit(self, do_omit: bool) -> Omittable<Self> {
        Omittable::new(self, do_omit)
    }

    /// Creates a decoder that will fail if the number of consumed bytes exceeds `bytes`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::{Decode, DecodeExt, ErrorKind};
    /// use bytecodec::bytes::Utf8Decoder;
    /// use bytecodec::io::IoDecodeExt;
    ///
    /// let mut decoder = Utf8Decoder::new().max_bytes(3);
    ///
    /// let item = decoder.decode_exact(b"foo".as_ref()).unwrap();
    /// assert_eq!(item, "foo"); // OK
    ///
    /// let error = decoder.decode_exact(b"hello".as_ref()).err();
    /// assert_eq!(error.map(|e| *e.kind()), Some(ErrorKind::InvalidInput)); // Error
    /// ```
    fn max_bytes(self, bytes: u64) -> MaxBytes<Self> {
        MaxBytes::new(self, bytes)
    }

    /// Takes two decoders and creates a new decoder that decodes both items in sequence.
    ///
    /// This is equivalent to call `TupleDecoder::new((self, other))`.
    fn chain<T: Decode>(self, other: T) -> TupleDecoder<(Self, T)> {
        TupleDecoder::new((self, other))
    }

    /// Creates a decoder that makes it possible to slice the input byte sequence in arbitrary units.
    ///
    /// Slicing an input byte sequence makes it easier to demultiplex multiple sequences from it.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::{Decode, DecodeExt, Eos};
    /// use bytecodec::bytes::Utf8Decoder;
    ///
    /// let mut decoder0 = Utf8Decoder::new().length(3).slice();
    /// let mut decoder1 = Utf8Decoder::new().length(3).slice();
    ///
    /// let eos = Eos::new(true);
    /// let input = b"fboaor";
    /// let mut offset = 0;
    ///
    /// for _ in 0..3 {
    ///     decoder0.set_consumable_bytes(1);
    ///     offset += decoder0.decode(&input[offset..], eos).unwrap();
    ///
    ///     decoder1.set_consumable_bytes(1);
    ///     offset += decoder1.decode(&input[offset..], eos).unwrap();
    /// }
    ///
    /// assert_eq!(offset, input.len());
    /// assert_eq!(decoder0.finish_decoding().unwrap(), "foo");
    /// assert_eq!(decoder1.finish_decoding().unwrap(), "bar");
    /// ```
    fn slice(self) -> Slice<Self> {
        Slice::new(self)
    }

    /// Creates a decoder that enables to peek decoded items before calling `finish_decoding` method.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::{Decode, DecodeExt, Eos};
    /// use bytecodec::fixnum::U8Decoder;
    /// use bytecodec::tuple::TupleDecoder;
    ///
    /// let mut decoder = TupleDecoder::new((
    ///     U8Decoder::new(),
    ///     U8Decoder::new(),
    ///     U8Decoder::new(),
    /// )).peekable();
    /// let size = decoder.decode(b"foo", Eos::new(false)).unwrap();
    /// assert_eq!(size, 3);
    /// assert_eq!(decoder.peek(), Some(&(b'f', b'o', b'o')));
    /// assert_eq!(decoder.finish_decoding().unwrap(), (b'f', b'o', b'o'));
    /// assert_eq!(decoder.peek(), None);
    /// ```
    fn peekable(self) -> Peekable<Self> {
        Peekable::new(self)
    }

    /// Creates a decoder that ignores EOS if there is no item being decoded.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::{Decode, DecodeExt, Eos};
    /// use bytecodec::fixnum::U16beDecoder;
    ///
    /// let mut decoder = U16beDecoder::new();
    /// assert!(decoder.decode(&[][..], Eos::new(true)).is_err()); // UnexpectedEos
    ///
    /// let mut decoder = U16beDecoder::new().maybe_eos();
    /// assert!(decoder.decode(&[][..], Eos::new(true)).is_ok()); // EOS is ignored
    ///
    /// let mut decoder = U16beDecoder::new().maybe_eos();
    /// assert!(decoder.decode(&[1][..], Eos::new(true)).is_err()); // UnexpectedEos
    /// ```
    fn maybe_eos(self) -> MaybeEos<Self> {
        MaybeEos::new(self)
    }

    /// Decodes an item by consuming the whole part of the given bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::DecodeExt;
    /// use bytecodec::fixnum::U16beDecoder;
    ///
    /// let mut decoder = U16beDecoder::new();
    /// assert_eq!(
    ///     decoder.decode_from_bytes(&[0x12, 0x34][..]).unwrap(),
    ///     0x1234
    /// );
    /// ```
    fn decode_from_bytes(&mut self, buf: &[u8]) -> Result<Self::Item> {
        let size = track!(self.decode(buf, Eos::new(true)))?;
        track_assert_eq!(size, buf.len(), ErrorKind::InvalidInput; self.is_idle());
        track!(self.finish_decoding())
    }
}
impl<T: Decode> DecodeExt for T {}

/// This trait allows for decoding tagged items from a byte sequence incrementally.
pub trait TaggedDecode: Decode {
    /// The type of tags prefixed to the items to be decoded.
    type Tag;

    /// Prepares to start decoding an item tagged by `tag`.
    ///
    /// # Errors
    ///
    /// The following errors may be returned by the decoder:
    /// - `ErrorKind::InvalidInput`:
    ///   - Unexpected tag was passed
    /// - `ErrorKind::IncompleteDecoding`:
    ///   - The previous decoding process has not been completed
    /// - `ErrorKind::DecoderTerminated`:
    ///   - The decoder has terminated (i.e., cannot decode any more items)
    /// - `ErrorKind::Other`:
    ///   - Other errors has occurred
    fn start_decoding(&mut self, tag: Self::Tag) -> Result<()>;
}
impl<D: ?Sized + TaggedDecode> TaggedDecode for &mut D {
    type Tag = D::Tag;

    fn start_decoding(&mut self, tag: Self::Tag) -> Result<()> {
        (**self).start_decoding(tag)
    }
}
impl<D: ?Sized + TaggedDecode> TaggedDecode for Box<D> {
    type Tag = D::Tag;

    fn start_decoding(&mut self, tag: Self::Tag) -> Result<()> {
        (**self).start_decoding(tag)
    }
}

/// This trait allows for decoding known-tagged or unknown-tagged items from a byte sequence incrementally.
pub trait TryTaggedDecode: Decode {
    /// The type of tags prefixed to the items to be decoded.
    type Tag;

    /// Tries to prepare to start decoding an item tagged by `tag`.
    ///
    /// If the given tag is unknown, it will return `Ok(false)`.
    ///
    /// # Errors
    ///
    /// The following errors may be returned by the decoder:
    /// - `ErrorKind::IncompleteDecoding`:
    ///   - The previous decoding process has not been completed
    /// - `ErrorKind::DecoderTerminated`:
    ///   - The decoder has terminated (i.e., cannot decode any more items)
    /// - `ErrorKind::Other`:
    ///   - Other errors has occurred
    fn try_start_decoding(&mut self, tag: Self::Tag) -> Result<bool>;
}
impl<D: ?Sized + TryTaggedDecode> TryTaggedDecode for &mut D {
    type Tag = D::Tag;

    fn try_start_decoding(&mut self, tag: Self::Tag) -> Result<bool> {
        (**self).try_start_decoding(tag)
    }
}
impl<D: ?Sized + TryTaggedDecode> TryTaggedDecode for Box<D> {
    type Tag = D::Tag;

    fn try_start_decoding(&mut self, tag: Self::Tag) -> Result<bool> {
        (**self).try_start_decoding(tag)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::fixnum::U16beDecoder;

    #[test]
    fn decode_from_bytes_works() {
        let mut decoder = U16beDecoder::new();
        assert_eq!(
            decoder.decode_from_bytes(&[0x12, 0x34][..]).unwrap(),
            0x1234
        );
    }
}
