//! Monolithic encoder and decoder.
use std::io::{self, Read, Write};

use {Decode, Encode, Eos, ErrorKind, Result};
use bytes::BytesEncoder;

/// This trait allows for decoding items monolithically from a source byte stream.
///
/// Although this has less flexibility than `Decode` trait, it has the merit of being easy to implement.
pub trait MonolithicDecode {
    /// The type of items to be monolithically decoded.
    type Item;

    /// Decodes an item from the given reader.
    fn monolithic_decode<R: Read>(&self, reader: R) -> Result<Self::Item>;
}

/// Monolithic decoder that implements `Decode` trait.
#[derive(Debug, Default)]
pub struct MonolithicDecoder<D> {
    inner: D,
    buf: Vec<u8>,
}
impl<D: MonolithicDecode> MonolithicDecoder<D> {
    /// Makes a new `MonolithicDecoder` instance.
    pub fn new(inner: D) -> Self {
        MonolithicDecoder {
            inner,
            buf: Vec::new(),
        }
    }

    /// Returns a reference to the inner decoder.
    pub fn inner_ref(&self) -> &D {
        &self.inner
    }

    /// Returns a mutable reference to the inner decoder.
    pub fn inner_mut(&mut self) -> &mut D {
        &mut self.inner
    }

    /// Takes ownership of `MonolithicDecoder` and returns the inner decoder.
    pub fn into_inner(self) -> D {
        self.inner
    }
}
impl<D: MonolithicDecode> Decode for MonolithicDecoder<D> {
    type Item = D::Item;

    fn decode(&mut self, mut buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        if eos.is_eos() {
            let original_len = buf.len();
            let item = track!(
                self.inner
                    .monolithic_decode(self.buf.as_slice().chain(buf.by_ref()))
            )?;
            self.buf.clear();
            Ok((original_len - buf.len(), Some(item)))
        } else {
            self.buf.extend_from_slice(buf);
            Ok((buf.len(), None))
        }
    }

    fn has_terminated(&self) -> bool {
        false
    }

    fn is_idle(&self) -> bool {
        self.buf.is_empty()
    }
}

/// This trait allows for encoding items monolithically to a destination byte stream.
///
/// Although this has less flexibility than `Encode` trait, it has the merit of being easy to implement.
pub trait MonolithicEncode {
    /// The type of items to be monolithically encoded.
    type Item;

    /// Encodes the item and writes the encoded bytes to the given writer.
    fn monolithic_encode<W: Write>(&self, item: &Self::Item, writer: W) -> Result<()>;
}

/// Monolithic encoder that implements `Encode` trait.
#[derive(Debug, Default)]
pub struct MonolithicEncoder<E: MonolithicEncode> {
    inner: E,
    item: Option<E::Item>,
    buf: BytesEncoder<Vec<u8>>,
}
impl<E: MonolithicEncode> MonolithicEncoder<E> {
    /// Makes a new `MonolithicEncoder` instance.
    pub fn new(inner: E) -> Self {
        MonolithicEncoder {
            inner,
            item: None,
            buf: BytesEncoder::new(),
        }
    }

    /// Returns a reference to the inner encoder.
    pub fn inner_ref(&self) -> &E {
        &self.inner
    }

    /// Returns a mutable reference to the inner encoder.
    pub fn inner_mut(&mut self) -> &mut E {
        &mut self.inner
    }

    /// Takes ownership of `MonolithicEncoder` and returns the inner encoder.
    pub fn into_inner(self) -> E {
        self.inner
    }
}
impl<E: MonolithicEncode> Encode for MonolithicEncoder<E> {
    type Item = E::Item;

    fn encode(&mut self, mut buf: &mut [u8], eos: Eos) -> Result<usize> {
        if let Some(item) = self.item.take() {
            let mut extra = Vec::new();
            {
                let writer = WriterChain::new(&mut buf, &mut extra);
                track!(self.inner.monolithic_encode(&item, writer))?;
            }
            if extra.is_empty() {
                track!(self.buf.start_encoding(extra))?;
            }
        }
        track!(self.buf.encode(buf, eos))
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track_assert!(self.is_idle(), ErrorKind::EncoderFull);
        self.item = Some(item);
        Ok(())
    }

    fn is_idle(&self) -> bool {
        self.item.is_none() && self.buf.is_idle()
    }
}

#[derive(Debug)]
struct WriterChain<A, B> {
    a: A,
    b: B,
}
impl<A, B> WriterChain<A, B> {
    fn new(a: A, b: B) -> Self {
        WriterChain { a, b }
    }
}
impl<A: Write, B: Write> Write for WriterChain<A, B> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.a.write(buf)? {
            0 => self.b.write(buf),
            n => Ok(n),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
