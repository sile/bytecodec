//! I/O (i.e., `Read` and `Write` traits) related module.
use std::io::{self, Read, Write};

use {Decode, DecodeBuf, DecodeExt, Encode, EncodeBuf, Error, ErrorKind, Result};

/// Decodes an item from `reader` by using `decoder`.
pub fn decode_from_reader<D, R>(mut decoder: D, mut reader: R) -> Result<D::Item>
where
    D: Decode,
    R: Read,
{
    let mut buf = [0; 1024];
    let mut buf = ReadBuf::new(&mut buf[..]);
    loop {
        let n = decoder.requiring_bytes_hint().unwrap_or(1);
        let stream_state = track!(buf.fill(reader.by_ref().take(n)))?;
        if stream_state.would_block() {
            track!(stream_state.to_io_result().map_err(Error::from))?;
        }

        let old_buf_len = buf.len();
        // TODO: let item = track!(buf.consume(decoder.by_ref().eos(stream_state.is_eos())));
        let item = if stream_state.is_eos() {
            track!(buf.consume(decoder.by_ref().length(old_buf_len as u64)))?
        } else {
            track!(buf.consume(&mut decoder))?
        };
        if let Some(item) = item {
            track_assert_eq!(buf.len(), 0, ErrorKind::Other);
            return Ok(item);
        }
        track_assert_ne!(buf.len(), old_buf_len, ErrorKind::Other);
    }
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
}
impl<B: AsRef<[u8]> + AsMut<[u8]>> ReadBuf<B> {
    /// Makes a new `ReadBuf` instance.
    pub fn new(inner: B) -> Self {
        ReadBuf {
            inner,
            head: 0,
            tail: 0,
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

    /// Fills the buffer by reading bytes from `reader`.
    ///
    /// This returns `Ok(true)` if the read operation would block.
    pub fn fill<R: Read>(&mut self, mut reader: R) -> Result<StreamState> {
        let mut state = StreamState::Normal;
        if !self.is_full() {
            match reader.read(&mut self.inner.as_mut()[self.tail..]) {
                Err(e) => {
                    if e.kind() == io::ErrorKind::WouldBlock {
                        state = StreamState::WouldBlock;
                    } else {
                        return Err(track!(Error::from(e)));
                    }
                }
                Ok(0) => {
                    state = StreamState::Eos;
                }
                Ok(size) => {
                    self.tail += size;
                }
            }
        }
        Ok(state)
    }

    /// Consumes the buffer by using `decoder`.
    pub fn consume<D: Decode>(&mut self, mut decoder: D) -> Result<Option<D::Item>> {
        let mut buf = DecodeBuf::new(&self.inner.as_ref()[self.head..self.tail]);
        let item = track!(decoder.decode(&mut buf))?;
        self.head = self.tail - buf.len();
        if self.head == self.tail {
            self.head = 0;
            self.tail = 0;
        }
        Ok(item)
    }

    pub fn inner_ref(&self) -> &B {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut B {
        &mut self.inner
    }

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
        let capacity = self.buf.len();
        let mut buf = EncodeBuf::new(&mut self.buf[self.tail..]);
        track!(encoder.encode(&mut buf))?;
        self.tail = capacity - buf.len();
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
