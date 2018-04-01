use std::io::{self, Read, Write};

use {Decode, DecodeBuf, Encode, EncodeBuf, Error, ErrorKind, Result};

// TODO: rename
#[derive(Debug)]
pub struct BufEncoder<E: Encode> {
    encoder: E,
    buf: Vec<u8>,
    head: usize,
    tail: usize,
}
impl<E: Encode> BufEncoder<E> {
    pub fn new(encoder: E) -> Self {
        BufEncoder {
            encoder,
            buf: vec![0; 4096],
            head: 0,
            tail: 0,
        }
    }

    pub fn inner(&self) -> &E {
        &self.encoder
    }

    pub fn inner_mut(&mut self) -> &mut E {
        &mut self.encoder
    }

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
            } else if self.encoder.is_completed() {
                return Ok(false);
            } else {
                let mut buf = EncodeBuf::new(&mut self.buf[self.tail..]);
                while !buf.is_empty() {
                    track!(self.encoder.encode(&mut buf))?;
                }
                self.head = self.tail - buf.len();
            }
        }
    }

    pub fn start_encoding(&mut self, item: E::Item) -> Result<()> {
        track!(self.encoder.start_encoding(item))
    }
}

// TODO: rename
#[derive(Debug)]
pub struct BufDecoder<D: Decode> {
    decoder: D,
    buf: Vec<u8>,
    head: usize,
    tail: usize,
    item: Option<D::Item>,
}
impl<D: Decode> BufDecoder<D> {
    pub fn new(decoder: D) -> Self {
        BufDecoder {
            decoder,
            buf: vec![0; 4096],
            head: 0,
            tail: 0,
            item: None,
        }
    }

    pub fn decode<R: Read>(&mut self, mut reader: R) -> Result<bool> {
        while self.item.is_none() {
            if self.tail != 0 {
                let mut buf = DecodeBuf::new(&self.buf[self.head..self.tail]);
                self.item = track!(self.decoder.decode(&mut buf))?;
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
                            return Ok(false);
                        }
                        return Err(track!(Error::from(e)));
                    }
                    Ok(0) => {
                        let mut buf = DecodeBuf::eos();
                        self.item = track!(self.decoder.decode(&mut buf))?;
                        return Ok(true);
                    }
                    Ok(size) => {
                        self.tail = size;
                    }
                }
            }
        }
        Ok(false)
    }

    pub fn pop_item(&mut self) -> Option<D::Item> {
        self.item.take()
    }
}
