//! I/O (i.e., `Read` and `Write` traits) related module.
#![allow(missing_docs)] // TODO: delete
use std::io::{self, Read, Write};

use {Decode, Encode, Eos, Error, ErrorKind, Result};

/// An extension of `Decode` trait to aid decodings involving I/O.
///
/// TODO: rename to DecodeExt(?)
pub trait IoDecodeExt: Decode {
    /// Tries to decode an item from the given read buffer.
    fn decode_from_read_buf<B>(&mut self, buf: &mut ReadBuf<B>) -> Result<Option<Self::Item>>
    where
        B: AsRef<[u8]>,
    {
        let eos = Eos::new(buf.stream_state.is_eos());
        let (size, item) = track!(self.decode(&buf.inner.as_ref()[buf.head..buf.tail], eos))?;
        buf.head += size;
        if buf.head == buf.tail {
            buf.head = 0;
            buf.tail = 0;
        }
        Ok(item)
    }

    /// Decodes an item from the given reader.
    ///
    /// This method reads only minimal bytes required to decode an item.
    ///
    /// Note that this is a blocking method.
    fn decode_exact<R: Read>(&mut self, mut reader: R) -> Result<Self::Item> {
        let mut buf = [0; 1024];
        loop {
            let mut size = self.requiring_bytes()
                .min(buf.len() as u64)
                .to_finite()
                .unwrap_or(1) as usize;
            let eos = if size != 0 {
                size = track!(reader.read(&mut buf[..size]).map_err(Error::from))?;
                Eos::new(size == 0)
            } else {
                Eos::new(false)
            };

            let (consumed, item) = track!(self.decode(&buf[..size], eos))?;
            track_assert_eq!(consumed, size, ErrorKind::Other; item.is_some(), eos);
            if let Some(item) = item {
                return Ok(item);
            }
        }
    }
}
impl<T: Decode> IoDecodeExt for T {}

/// An extension of `Encode` trait to aid encodings involving I/O.
pub trait IoEncodeExt: Encode {
    fn encode_to_write_buf<B>(&mut self, buf: &mut WriteBuf<B>) -> Result<()>
    where
        B: AsMut<[u8]>,
    {
        let eos = Eos::new(buf.stream_state.is_eos());
        let size = track!(self.encode(&mut buf.inner.as_mut()[buf.tail..], eos))?;
        buf.tail += size;
        Ok(())
    }

    // TODO:
    fn encode_all<W: Write>(&mut self, mut writer: W) -> Result<()> {
        let mut buf = [0; 1024];
        while !self.is_idle() {
            let size = track!(self.encode(&mut buf[..], Eos::new(false)))?;
            track!(writer.write_all(&buf[..size]).map_err(Error::from))?;
            track_assert_ne!(size, 0, ErrorKind::Other);
        }
        Ok(())
    }
}
impl<T: Encode> IoEncodeExt for T {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum StreamState {
    Normal,
    Eos,
    WouldBlock,
    Error,
}
impl StreamState {
    /// Returns `true` if the state is `Normal`, otherwise `false`.
    pub fn is_normal(&self) -> bool {
        *self == StreamState::Normal
    }

    pub fn is_error(&self) -> bool {
        *self == StreamState::Error
    }

    /// Returns `true` if the state is `Eos`, otherwise `false`.
    pub fn is_eos(&self) -> bool {
        *self == StreamState::Eos
    }

    /// Returns `true` if the state is `WouldBlock`, otherwise `false`.
    pub fn would_block(&self) -> bool {
        *self == StreamState::WouldBlock
    }
}

/// Read buffer.
#[derive(Debug)]
pub struct ReadBuf<B> {
    inner: B,
    head: usize,
    tail: usize,
    stream_state: StreamState,
}
impl<B: AsRef<[u8]> + AsMut<[u8]>> ReadBuf<B> {
    /// Makes a new `ReadBuf` instance.
    pub fn new(inner: B) -> Self {
        ReadBuf {
            inner,
            head: 0,
            tail: 0,
            stream_state: StreamState::Normal,
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

    pub fn stream_state(&self) -> StreamState {
        self.stream_state
    }

    pub fn stream_state_mut(&mut self) -> &mut StreamState {
        &mut self.stream_state
    }

    pub fn fill<R: Read>(&mut self, mut reader: R) -> Result<()> {
        while !self.is_full() {
            match reader.read(&mut self.inner.as_mut()[self.tail..]) {
                Err(e) => {
                    if e.kind() == io::ErrorKind::WouldBlock {
                        self.stream_state = StreamState::WouldBlock;
                        break;
                    } else {
                        self.stream_state = StreamState::Error;
                        return Err(track!(Error::from(e)));
                    }
                }
                Ok(0) => {
                    self.stream_state = StreamState::Eos;
                    break;
                }
                Ok(size) => {
                    self.stream_state = StreamState::Normal;
                    self.tail += size;
                }
            }
        }
        Ok(())
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
pub struct WriteBuf<B> {
    inner: B,
    head: usize,
    tail: usize,
    stream_state: StreamState,
}
impl<B: AsRef<[u8]> + AsMut<[u8]>> WriteBuf<B> {
    pub fn new(inner: B) -> Self {
        WriteBuf {
            inner,
            head: 0,
            tail: 0,
            stream_state: StreamState::Normal,
        }
    }

    pub fn len(&self) -> usize {
        self.tail - self.head
    }

    pub fn is_empty(&self) -> bool {
        self.tail == 0
    }

    pub fn is_full(&self) -> bool {
        self.tail == self.inner.as_ref().len()
    }

    pub fn stream_state(&self) -> StreamState {
        self.stream_state
    }

    pub fn stream_state_mut(&mut self) -> &mut StreamState {
        &mut self.stream_state
    }

    pub fn flush<W: Write>(&mut self, mut writer: W) -> Result<()> {
        while !self.is_empty() {
            match writer.write(&mut self.inner.as_mut()[self.head..self.tail]) {
                Err(e) => {
                    if e.kind() == io::ErrorKind::WouldBlock {
                        self.stream_state = StreamState::WouldBlock;
                        break;
                    } else {
                        self.stream_state = StreamState::Error;
                        return Err(track!(Error::from(e)));
                    }
                }
                Ok(0) => {
                    self.stream_state = StreamState::Eos;
                    break;
                }
                Ok(size) => {
                    self.stream_state = StreamState::Normal;
                    self.head += size;
                    if self.head == self.tail {
                        self.head = 0;
                        self.tail = 0;
                    }
                }
            }
        }
        Ok(())
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
