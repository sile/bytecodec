use std::collections::VecDeque;
use std::io::{self, Read, Write};

use {Decode, DecodeBuf, Encode, EncodeBuf, Error, ErrorKind, Result};

// TODO: rename
#[derive(Debug)]
pub struct BufEncoder<E: Encode> {
    encoder: E,
    buf: Vec<u8>,
    head: usize,
    tail: usize,
    queue: VecDeque<E::Item>,
}
impl<E: Encode> BufEncoder<E> {
    pub fn new(encoder: E) -> Self {
        BufEncoder {
            encoder,
            buf: vec![0; 4096],
            head: 0,
            tail: 0,
            queue: VecDeque::new(),
        }
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
                if let Some(item) = self.queue.pop_front() {
                    track!(self.encoder.start_encoding(item))?;
                } else {
                    return Ok(false);
                }
            } else {
                let mut buf = EncodeBuf::new(&mut self.buf[self.tail..]);
                while !buf.is_empty() {
                    track!(self.encoder.encode(&mut buf))?;
                }
                self.head = self.tail - buf.len();
            }
        }
    }

    pub fn enqueue_item(&mut self, item: E::Item) {
        self.queue.push_back(item);
    }
}

// TODO: rename
#[derive(Debug)]
pub struct BufDecoder<D> {
    decoder: D,
    buf: Vec<u8>,
    head: usize,
    tail: usize,
}
impl<D: Decode> BufDecoder<D> {
    pub fn new(decoder: D) -> Self {
        BufDecoder {
            decoder,
            buf: vec![0; 4096],
            head: 0,
            tail: 0,
        }
    }

    pub fn decode<R: Read>(&mut self, mut reader: R) -> Result<Decoded<D::Item>> {
        loop {
            if self.tail != 0 {
                let mut buf = DecodeBuf::new(&self.buf[self.head..self.tail]);
                let item = track!(self.decoder.decode(&mut buf))?;
                if buf.is_empty() {
                    self.head = 0;
                    self.tail = 0;
                } else {
                    self.head = self.tail - buf.len();
                }
                if let Some(item) = item {
                    return Ok(Decoded {
                        eos: false,
                        item: Some(item),
                    });
                }
            } else {
                match reader.read(&mut self.buf) {
                    Err(e) => {
                        if e.kind() == io::ErrorKind::WouldBlock {
                            return Ok(Decoded {
                                eos: false,
                                item: None,
                            });
                        }
                        return Err(track!(Error::from(e)));
                    }
                    Ok(0) => {
                        let mut buf = DecodeBuf::eos();
                        let item = track!(self.decoder.decode(&mut buf))?;
                        return Ok(Decoded { eos: true, item });
                    }
                    Ok(size) => {
                        self.tail = size;
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Decoded<T> {
    pub eos: bool,
    pub item: Option<T>,
}
