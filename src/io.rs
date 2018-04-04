//! I/O (i.e., `Read` and `Write` traits) related module.
#![allow(missing_docs)] // TODO: delete
use std::io::{self, Read, Write};

use {Decode, DecodeBuf, Encode, Eos, Error, ErrorKind, Result};

/// An extension of `Decode` trait to aid decodings involving I/O.
pub trait IoDecodeExt: Decode {
    /// Decodes an item from the given read buffer.
    fn decode_from_read_buf<B>(&mut self, buf: &mut ReadBuf<B>) -> Result<Option<Self::Item>>
    where
        B: AsRef<[u8]>,
    {
        let mut decode_buf = DecodeBuf::with_eos(&buf.inner.as_ref()[buf.head..buf.tail], buf.eos);
        let item = track!(self.decode(&mut decode_buf))?;
        buf.head = buf.tail - decode_buf.len();
        if buf.head == buf.tail {
            buf.head = 0;
            buf.tail = 0;
        }
        Ok(item)
    }

    /// Decodes an item from the given reader.
    ///
    /// This returns `Ok(None)` only if the reader has reached EOS and there is no item being processed.
    fn blocking_decode_from_reader<R: Read>(
        &mut self,
        mut reader: R,
    ) -> Result<Option<Self::Item>> {
        let mut buf = [0; 1024];
        let mut buf = ReadBuf::new(&mut buf[..]);
        loop {
            let n = self.requiring_bytes_hint().unwrap_or(1);
            if n != 0 {
                let stream_state = track!(buf.read_from(reader.by_ref().take(n)))?;
                if stream_state.would_block() {
                    track!(stream_state.to_io_result().map_err(Error::from))?;
                }
            }

            let old_buf_len = buf.len();
            let item = track!(self.decode_from_read_buf(&mut buf))?;
            if let Some(item) = item {
                track_assert_eq!(buf.len(), 0, ErrorKind::Other);
                return Ok(Some(item));
            } else if buf.is_empty() && buf.is_eos() {
                track_assert!(self.is_idle(), ErrorKind::UnexpectedEos);
                return Ok(None);
            }
            track_assert_ne!(buf.len(), old_buf_len, ErrorKind::Other);
        }
    }
}

/// An extension of `Encode` trait to aid encodings involving I/O.
pub trait IoEncodeExt: Encode {
    /// Encodes the items in the encoder to the given writer buffer.
    fn encode_to_write_buf(&mut self, buf: &mut WriteBuf) -> Result<()>;

    /// Encodes the items in the encoder to the given writer.
    ///
    /// This returns `Ok(())` only if the encoder completes to encode (and write) all of the items.
    fn blocking_encode_to_writer<W: Write>(&mut self, writer: W) -> Result<()>;
}

/// Encodes `item` by using `encoder` and writes the encoded bytes to `writer`.
pub fn encode_to_writer<E, W>(mut encoder: E, item: E::Item, mut writer: W) -> Result<()>
where
    E: Encode,
    W: Write,
{
    track!(encoder.start_encoding(item))?;
    let mut buf = WriteBuf::new(1024);
    while !encoder.is_idle() {
        track!(buf.fill(&mut encoder))?;
        if track!(buf.consume(&mut writer))? {
            let e = io::Error::from(io::ErrorKind::WouldBlock);
            return Err(track!(Error::from(e)));
        }
    }
    Ok(())
}

/// State of I/O stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[allow(missing_docs)]
pub enum StreamState {
    Normal,
    Eos,
    WouldBlock,
}
impl StreamState {
    /// Returns `true` if the state is `Normal`, otherwise `false`.
    pub fn is_normal(&self) -> bool {
        *self == StreamState::Normal
    }

    /// Returns `true` if the state is `Eos`, otherwise `false`.
    pub fn is_eos(&self) -> bool {
        *self == StreamState::Eos
    }

    /// Returns `true` if the state is `WouldBlock`, otherwise `false`.
    pub fn would_block(&self) -> bool {
        *self == StreamState::WouldBlock
    }

    /// Converts to the corresponding I/O result.
    pub fn to_io_result(&self) -> io::Result<()> {
        let kind = match *self {
            StreamState::WouldBlock => io::ErrorKind::WouldBlock,
            StreamState::Eos => io::ErrorKind::UnexpectedEof,
            StreamState::Normal => return Ok(()),
        };
        Err(io::Error::from(kind))
    }
}

/// Read buffer.
#[derive(Debug)]
pub struct ReadBuf<B> {
    inner: B,
    head: usize,
    tail: usize,
    eos: bool,
}
impl<B: AsRef<[u8]> + AsMut<[u8]>> ReadBuf<B> {
    /// Makes a new `ReadBuf` instance.
    pub fn new(inner: B) -> Self {
        ReadBuf {
            inner,
            head: 0,
            tail: 0,
            eos: false,
        }
    }

    /// Returns the number of filled bytes in the buffer.
    pub fn len(&self) -> usize {
        self.tail - self.head
    }

    /// Returns `true` if the buffer is empty, otherwise `false`.
    pub fn is_empty(&self) -> bool {
        self.tail == 0
    }

    /// Returns `true` if the buffer is full, otherwise `false`.
    pub fn is_full(&self) -> bool {
        self.tail == self.inner.as_ref().len()
    }

    pub fn is_eos(&self) -> bool {
        self.eos
    }

    pub fn read_from<R: Read>(&mut self, mut reader: R) -> Result<StreamState> {
        let mut state = StreamState::Normal;
        if !self.is_full() {
            match reader.read(&mut self.inner.as_mut()[self.tail..]) {
                Err(e) => {
                    self.eos = false;
                    if e.kind() == io::ErrorKind::WouldBlock {
                        state = StreamState::WouldBlock;
                    } else {
                        return Err(track!(Error::from(e)));
                    }
                }
                Ok(0) => {
                    state = StreamState::Eos;
                    self.eos = true;
                }
                Ok(size) => {
                    self.tail += size;
                    self.eos = false;
                }
            }
        }
        Ok(state)
    }

    /// Returns a reference to the inner bytes of the buffer.
    pub fn inner_ref(&self) -> &B {
        &self.inner
    }

    /// Returns a mutable reference to the inner bytes of the buffer.
    pub fn inner_mut(&mut self) -> &mut B {
        &mut self.inner
    }

    /// Takes ownership of `ReadBuf` and returns the inner bytes of the buffer.
    pub fn into_inner(self) -> B {
        self.inner
    }
}

/// Write buffer.
#[derive(Debug)]
pub struct WriteBuf {
    buf: Vec<u8>,
    head: usize,
    tail: usize,
}
impl WriteBuf {
    pub fn new(size: usize) -> Self {
        WriteBuf {
            buf: vec![0; size],
            head: 0,
            tail: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.tail == 0
    }

    pub fn is_full(&self) -> bool {
        self.tail == self.buf.len()
    }

    pub fn fill<E: Encode>(&mut self, mut encoder: E) -> Result<()> {
        let eos = false; // TODO
        let size = track!(encoder.encode(&mut self.buf[self.tail..], Eos::new(eos)))?;
        self.tail += size;
        Ok(())
    }

    pub fn consume<W: Write>(&mut self, mut writer: W) -> Result<bool> {
        while !self.is_empty() {
            match writer.write(&self.buf[self.head..self.tail]) {
                Err(e) => {
                    if e.kind() == io::ErrorKind::WouldBlock {
                        return Ok(true);
                    } else {
                        return Err(track!(Error::from(e)));
                    }
                }
                Ok(0) => {
                    track_panic!(ErrorKind::UnexpectedEos);
                }
                Ok(size) => {
                    self.head += size;
                    if self.head == self.tail {
                        self.head = 0;
                        self.tail = 0;
                    }
                }
            }
        }
        Ok(false)
    }
}
