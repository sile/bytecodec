//! I/O (i.e., `Read` and `Write` traits) related module.
use std::io::{self, Read, Write};

use {Decode, DecodeBuf, Encode, EncodeBuf, Error, ErrorKind, Result};

/// `IoDecoder` wraps a `Decode` implementor and enables to decode items from `Read` implementors.
///
/// # Examples
///
/// ```
/// use std::io::Cursor;
/// use bytecodec::fixnum::U8Decoder;
/// use bytecodec::io::IoDecoder;
///
/// let mut decoder = IoDecoder::new(U8Decoder::new());
/// let mut reader = Cursor::new(vec![0, 1, 2]);
///
/// assert_eq!(decoder.decode(&mut reader).unwrap(), (false, Some(0)));
/// assert_eq!(decoder.decode(&mut reader).unwrap(), (false, Some(1)));
/// assert_eq!(decoder.decode(&mut reader).unwrap(), (false, Some(2)));
/// assert_eq!(decoder.decode(&mut reader).unwrap(), (true, None));
/// ```
#[derive(Debug)]
pub struct IoDecoder<D: Decode> {
    inner: D,
    buf: Vec<u8>,
    head: usize,
    tail: usize,
}
impl<D: Decode> IoDecoder<D> {
    /// Makes a new `IoDecoder` instance.
    ///
    /// This is equivalent to `IoDecoder::with_buffer_wize(inner, 4096)`.
    pub fn new(inner: D) -> Self {
        Self::with_buffer_wize(inner, 4096)
    }

    /// Makes a new `IoDecoder` instance with the reading buffer of the given size.
    pub fn with_buffer_wize(inner: D, size: usize) -> Self {
        IoDecoder {
            inner,
            buf: vec![0; size],
            head: 0,
            tail: 0,
        }
    }

    /// Decodes an item from the given reader.
    ///
    /// The first element of the resulting tuple indicates whether the stream has reached EOS.
    /// If it is `true`, the stream has reached EOS.
    ///
    /// If the reader returns a `std::io::ErrorKind::WouldBlock` error while reading bytes,
    /// the decoding process will be suspended and `Ok((false, None))` will be returned as the result.
    pub fn decode<R: Read>(&mut self, mut reader: R) -> Result<(bool, Option<D::Item>)> {
        let mut item = None;
        while item.is_none() {
            if self.tail != 0 {
                let mut buf = DecodeBuf::new(&self.buf[self.head..self.tail]);
                item = track!(self.inner.decode(&mut buf))?;
                if buf.is_empty() {
                    self.head = 0;
                    self.tail = 0;
                } else {
                    self.head = self.tail - buf.len();
                }
            } else {
                match reader.read(&mut self.buf) {
                    Err(e) => {
                        if e.kind() == io::ErrorKind::WouldBlock {
                            break;
                        }
                        return Err(track!(Error::from(e)));
                    }
                    Ok(0) => {
                        if !self.inner.is_idle() {
                            let mut buf = DecodeBuf::new_as_eos(&[]);
                            item = track!(self.inner.decode(&mut buf))?;
                        }
                        return Ok((true, item));
                    }
                    Ok(size) => {
                        self.tail = size;
                    }
                }
            }
        }
        Ok((false, item))
    }

    /// Returns a reference to the inner decoder.
    pub fn inner_ref(&self) -> &D {
        &self.inner
    }

    /// Returns a mutable reference to the inner decoder.
    pub fn inner_mut(&mut self) -> &mut D {
        &mut self.inner
    }

    /// Takes ownership of `IoDecoder` and returns the inner decoder.
    pub fn into_inner(self) -> D {
        self.inner
    }
}

/// `IoEncoder` wraps an `Encode` implementor and enables to encode items to `Write` implementors.
///
/// # Examples
///
/// ```
/// use std::io::Cursor;
/// use bytecodec::fixnum::U8Encoder;
/// use bytecodec::io::IoEncoder;
///
/// let mut encoder = IoEncoder::new(U8Encoder::new());
/// let mut writer = Vec::new();
///
/// for i in 0..3 {
///     encoder.start_encoding(i).unwrap();
///     encoder.encode(&mut writer).unwrap();
/// }
/// assert_eq!(writer, [0, 1, 2]);
/// ```
#[derive(Debug)]
pub struct IoEncoder<E: Encode> {
    inner: E,
    buf: Vec<u8>,
    head: usize,
    tail: usize,
}
impl<E: Encode> IoEncoder<E> {
    /// Makes a new `IoEncoder` instance.
    ///
    /// This is equivalent to `IoEncoder::with_buffer_wize(inner, 4096)`.
    pub fn new(inner: E) -> Self {
        Self::with_buffer_wize(inner, 4096)
    }

    /// Makes a new `IoEncoder` instance with the writing buffer of the given size.
    pub fn with_buffer_wize(inner: E, size: usize) -> Self {
        IoEncoder {
            inner,
            buf: vec![0; size],
            head: 0,
            tail: 0,
        }
    }

    /// Encodes the current item and writes the encoded bytes to the given writer.
    ///
    /// The result indicates whether the stream has reached EOS.
    /// If it is `true`, the stream has reached EOS.
    ///
    /// If the writer returns a `std::io::ErrorKind::WouldBlock` error while writing bytes,
    /// the encoding process will be suspended and `Ok(false)` will be returned as the result.
    pub fn encode<W: Write>(&mut self, mut writer: W) -> Result<bool> {
        loop {
            if self.tail != 0 {
                match writer.write(&self.buf[self.head..self.tail]) {
                    Err(e) => {
                        if e.kind() == io::ErrorKind::WouldBlock {
                            return Ok(false);
                        }
                        return Err(track!(Error::from(e)));
                    }
                    Ok(size) => {
                        track_assert_ne!(size, 0, ErrorKind::UnexpectedEos);
                        self.head += size;
                        if self.head == self.tail {
                            self.head = 0;
                            self.tail = 0;
                        }
                    }
                }
            } else if self.inner.is_idle() {
                return Ok(false);
            } else {
                let mut buf = EncodeBuf::new(&mut self.buf[self.tail..]);
                let old_buf_len = buf.len();
                track!(self.inner.encode(&mut buf))?;
                self.tail += (old_buf_len - buf.len());
            }
        }
    }

    /// Tries to start encoding the given item.
    pub fn start_encoding(&mut self, item: E::Item) -> Result<()> {
        track!(self.inner.start_encoding(item))
    }

    /// Returns a reference to the inner encoder.
    pub fn inner_ref(&self) -> &E {
        &self.inner
    }

    /// Returns a mutable reference to the inner encoder.
    pub fn inner_mut(&mut self) -> &mut E {
        &mut self.inner
    }

    /// Takes ownership of `IoEncoder` and returns the inner encoder.
    pub fn into_inner(self) -> E {
        self.inner
    }
}
