use std;

use combinator::{AndThen, Assert, Buffered, Collect, CollectN, DecoderChain, Length, Map, MapErr,
                 MaxBytes, MaybeEos, Omittable, SkipRemaining, Slice, TryMap};
use {ByteCount, Eos, Error, ErrorKind, Result};

/// This trait allows for decoding items from a byte sequence incrementally.
pub trait Decode {
    /// The type of items to be decoded.
    type Item;

    /// Consumes the given buffer (a part of a byte sequence), and decodes an item from it.
    ///
    /// The first element of a succeeded result is the number of bytes consumed
    /// from `buf` by the decoding process.
    ///
    /// If an item is successfully decoded, the decoder will return `Ok((_, Some(..)))`.
    ///
    /// If the buffer does not contain enough bytes to decode the next item,
    /// the decoder will return `Ok((_, None))`.
    /// In this case, the decoder must consume as many bytes in the buffer as possible.
    ///
    /// If an item is not yet decoded but the number of consumed bytes in the last `decode` invocation
    /// is smaller than the length of `buf`, it means the decoder has been suspended its work in any reasons.
    /// In that case the decoder may require some instructions from clients to resume the work,
    /// but its concrete method is beyond the scope of this trait.
    ///
    /// # Errors
    ///
    /// Decoders return the following kinds of errors as necessary:
    /// - `ErrorKind::DecoderTerminated`:
    ///   - If all decodable items have been decoded,
    ///     the decoder must return this kind of error when `decode` method is called.
    /// - `ErrorKind::UnexpectedEos`:
    ///   - The invocation of `eos.is_reached()` returns `true` despite of
    ///     the decoder requires more bytes to decode the next item.
    /// - `ErrorKind::InvalidInput`:
    ///   - Decoded items have invalid values
    ///   - Invalid parameters were given to decoders
    /// - `ErrorKind::Error`:
    ///   - Other errors
    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)>;

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
    fn requiring_bytes(&self) -> ByteCount;
}
impl<'a, D: ?Sized + Decode> Decode for &'a mut D {
    type Item = D::Item;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        (**self).decode(buf, eos)
    }

    fn requiring_bytes(&self) -> ByteCount {
        (**self).requiring_bytes()
    }
}
impl<D: ?Sized + Decode> Decode for Box<D> {
    type Item = D::Item;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        (**self).decode(buf, eos)
    }

    fn requiring_bytes(&self) -> ByteCount {
        (**self).requiring_bytes()
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
    /// extern crate bytecodec;
    /// #[macro_use]
    /// extern crate trackable;
    ///
    /// use bytecodec::{Decode, DecodeExt, ErrorKind, Result};
    /// use bytecodec::fixnum::U8Decoder;
    /// use bytecodec::io::IoDecodeExt;
    ///
    /// # fn main() {
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
    /// # }
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
    /// extern crate bytecodec;
    /// #[macro_use]
    /// extern crate trackable;
    ///
    /// use bytecodec::{Decode, DecodeExt};
    /// use bytecodec::fixnum::U16beDecoder;
    /// use bytecodec::io::IoDecodeExt;
    ///
    /// # fn main() {
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
    ///   [0] at src/bytes.rs:155
    ///   [1] at src/fixnum.rs:192
    ///   [2] at src/decode.rs:12 -- oops!
    ///   [3] at src/io.rs:44
    ///   [4] at src/decode.rs:16\n");
    /// # }
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

    /// Takes two decoders and creates a new decoder that decodes both items in sequence.
    ///
    /// Chains are started by calling `StartDecoderChain::chain` method.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::{Decode, DecodeExt, StartDecoderChain};
    /// use bytecodec::fixnum::U8Decoder;
    /// use bytecodec::io::IoDecodeExt;
    ///
    /// let mut decoder = StartDecoderChain
    ///     .chain(U8Decoder::new())
    ///     .chain(U8Decoder::new())
    ///     .chain(U8Decoder::new());
    ///
    /// let mut input = &b"foobar"[..];
    ///
    /// let item = decoder.decode_exact(&mut input).unwrap();
    /// assert_eq!(item, (b'f', b'o', b'o'));
    ///
    /// let item = decoder.decode_exact(&mut input).unwrap();
    /// assert_eq!(item, (b'b', b'a', b'r'));
    /// ```
    fn chain<D: Decode>(self, other: D) -> DecoderChain<Self, D, Self::Item> {
        DecoderChain::new(self, other)
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

    /// Creates a decoder for skipping the remaining bytes in an input byte sequence
    /// after decoding an item by using `self`.
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
    /// let mut decoder = U8Decoder::new().skip_remaining();
    /// let item = decoder.decode_exact(&mut input).unwrap();
    /// assert_eq!(item, b'f');
    /// assert!(input.is_empty());
    /// ```
    fn skip_remaining(self) -> SkipRemaining<Self> {
        SkipRemaining::new(self)
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

    /// Creates a decoder that will fail if the given assertion function returns `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::{Decode, DecodeExt, ErrorKind};
    /// use bytecodec::fixnum::U8Decoder;
    /// use bytecodec::io::IoDecodeExt;
    ///
    /// let mut decoder = U8Decoder::new().assert(|&b| b == 3);
    ///
    /// let mut input = &[3, 4][..];
    /// let item = decoder.decode_exact(&mut input).unwrap();
    /// assert_eq!(item, 3);
    ///
    /// let error = decoder.decode_exact(&mut input).err();
    /// assert_eq!(error.map(|e| *e.kind()), Some(ErrorKind::InvalidInput));
    /// ```
    fn assert<F>(self, f: F) -> Assert<Self, F>
    where
        F: for<'a> Fn(&'a Self::Item) -> bool,
    {
        Assert::new(self, f)
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
    /// let mut last_item0 = None;
    /// let mut last_item1 = None;
    /// for _ in 0..3 {
    ///     decoder0.set_consumable_bytes(1);
    ///     let (size, item) = decoder0.decode(&input[offset..], eos).unwrap();
    ///     offset += size;
    ///     last_item0 = item;
    ///
    ///     decoder1.set_consumable_bytes(1);
    ///     let (size, item) = decoder1.decode(&input[offset..], eos).unwrap();
    ///     offset += size;
    ///     last_item1 = item;
    /// }
    ///
    /// assert_eq!(offset, input.len());
    /// assert_eq!(last_item0, Some("foo".to_owned()));
    /// assert_eq!(last_item1, Some("bar".to_owned()));
    /// ```
    fn slice(self) -> Slice<Self> {
        Slice::new(self)
    }

    /// Creates a decoder that buffers the last decoded item.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::{Decode, DecodeExt, Eos, StartDecoderChain};
    /// use bytecodec::fixnum::U8Decoder;
    ///
    /// let mut decoder = StartDecoderChain
    ///     .chain(U8Decoder::new())
    ///     .chain(U8Decoder::new())
    ///     .chain(U8Decoder::new())
    ///     .buffered();
    /// let (size, item) = decoder.decode(b"foo", Eos::new(false)).unwrap();
    /// assert_eq!(size, 3);
    /// assert_eq!(item, None);
    /// assert_eq!(decoder.take_item(), Some((b'f', b'o', b'o')));
    /// assert_eq!(decoder.has_item(), false);
    /// ```
    fn buffered(self) -> Buffered<Self> {
        Buffered::new(self)
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
        let (size, item) = track!(self.decode(buf, Eos::new(true)))?;
        track_assert_eq!(size, buf.len(), ErrorKind::InvalidInput);
        let item = track_assert_some!(item, ErrorKind::InvalidInput);
        Ok(item)
    }
}
impl<T: Decode> DecodeExt for T {}

#[cfg(test)]
mod test {
    use super::*;
    use fixnum::U16beDecoder;

    #[test]
    fn decode_from_bytes_works() {
        let mut decoder = U16beDecoder::new();
        assert_eq!(
            decoder.decode_from_bytes(&[0x12, 0x34][..]).unwrap(),
            0x1234
        );
    }
}
