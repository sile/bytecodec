//! I/O (i.e., `Read` and `Write` traits) related module.
use std::io::{self, Read, Write};

use {Decode, DecodeBuf, Encode, EncodeBuf, Error, ErrorKind, Result};

/// Decodes an item from `reader` by using `decoder`.
pub fn decode_from_reader<D, R>(mut decoder: D, mut reader: R) -> Result<D::Item>
where
    D: Decode,
    R: Read,
{
    let mut buf = ReadBuf::new([0; 1024]);
    loop {
        let n = decoder.requiring_bytes_hint().unwrap_or(1);
        if n != 0 {
            if track!(buf.fill(reader.by_ref().take(n)))? {
                let e = io::Error::from(io::ErrorKind::WouldBlock);
                return Err(track!(Error::from(e)));
            }
        }
        if let Some(item) = track!(buf.consume(&mut decoder))? {
            return Ok(item);
        }
        track_assert_ne!(n, 0, ErrorKind::Other);
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

    /// Returns `true` if the reader that used in the last `fill` method call has reached EOS,
    /// otherwise `false`.
    pub fn is_eos(&self) -> bool {
        self.eos
    }

    /// Returns `true` if the buffer is empty, otherwise `false`.
    pub fn is_empty(&self) -> bool {
        self.tail == 0
    }

    /// Returns `true` if the buffer is full, otherwise `false`.
    pub fn is_full(&self) -> bool {
        self.tail == self.buf.len()
    }

    /// Fills the buffer by reading bytes from `reader`.
    ///
    /// This returns `Ok(true)` if the read operation would block.
    pub fn fill<R: Read>(&mut self, mut reader: R) -> Result<bool> {
        while self.is_full() {
            match reader.read(&mut self.buf[self.tail..]) {
                Err(e) => {
                    if e.kind() == io::ErrorKind::WouldBlock {
                        return Ok(true);
                    } else {
                        return Err(track!(Error::from(e)));
                    }
                }
                Ok(0) => {
                    self.eos = true;
                    break;
                }
                Ok(size) => {
                    self.eos = false;
                    self.tail += size;
                }
            }
        }
        Ok(false)
    }

    /// Consumes the buffer by using `decoder`.
    ///
    pub fn consume<D: Decode>(&mut self, mut decoder: D) -> Result<Option<D::Item>> {
        let mut buf = DecodeBuf::with_eos(&self.buf[self.head..self.tail], self.eos);
        let item = track!(decoder.decode(&mut buf))?;
        self.head = self.tail - buf.len();
        if self.head == self.tail {
            self.head = 0;
            self.tail = 0;
        }
        Ok(item)
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
