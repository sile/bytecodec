use std::cmp;
use std::io::{self, Read, Write};
use std::ops::{Deref, DerefMut};

use {ErrorKind, Result};

/// Decoding buffer.
///
/// A `DecodeBuf` represents a slice of a byte sequence.
/// Decoders consume consecutive buffers and decode items.
///
/// In addition, `DecodeBuf` optionally provides the number of bytes remaining in the sequence to decoders.
#[derive(Debug, Clone)]
pub struct DecodeBuf<'a> {
    buf: &'a [u8],
    offset: usize,
    remaining_bytes: Option<u64>,
}
impl<'a> DecodeBuf<'a> {
    /// Makes a new `DecodeBuf` instance without remaining bytes information.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::DecodeBuf;
    ///
    /// let buf = DecodeBuf::new(b"foo");
    /// assert_eq!(buf.as_ref(), b"foo");
    /// assert_eq!(buf.remaining_bytes(), None);
    /// ```
    pub fn new(buf: &'a [u8]) -> Self {
        DecodeBuf {
            buf,
            offset: 0,
            remaining_bytes: None,
        }
    }

    /// Makes a new `DecodeBuf` instance with the given EOS indication.
    ///
    /// ```
    /// use bytecodec::DecodeBuf;
    ///
    /// let buf = DecodeBuf::with_eos(b"foo", true); // EOS
    /// assert_eq!(buf.remaining_bytes(), Some(0));
    /// assert!(buf.is_eos());
    ///
    /// let buf = DecodeBuf::with_eos(b"foo", false); // Not EOS
    /// assert_eq!(buf.remaining_bytes(), None);
    /// assert!(!buf.is_eos());
    /// ```
    pub fn with_eos(buf: &'a [u8], eos: bool) -> Self {
        DecodeBuf {
            buf,
            offset: 0,
            remaining_bytes: if eos { Some(0) } else { None },
        }
    }

    /// Makes a new `DecodeBuf` instance with the given number of remaining bytes.
    ///
    /// ```
    /// use bytecodec::DecodeBuf;
    ///
    /// let buf = DecodeBuf::with_remaining_bytes(b"foo", 10);
    /// assert_eq!(buf.as_ref(), b"foo");
    /// assert_eq!(buf.remaining_bytes(), Some(10));
    /// ```
    pub fn with_remaining_bytes(buf: &'a [u8], remaining_bytes: u64) -> Self {
        DecodeBuf {
            buf,
            offset: 0,
            remaining_bytes: Some(remaining_bytes),
        }
    }

    /// Returns the number of bytes remaining in the sequence.
    ///
    /// Note that it does not contain the number of the bytes in this buffer.
    ///
    /// `None` means there is no knowledge about the length of the byte sequence.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::DecodeBuf;
    ///
    /// let mut buf = DecodeBuf::with_remaining_bytes(b"foo", 10);
    /// assert_eq!(buf.len(), 3);
    /// assert_eq!(buf.remaining_bytes(), Some(10));
    ///
    /// buf.consume(2).unwrap();
    /// assert_eq!(buf.len(), 1);
    /// assert_eq!(buf.remaining_bytes(), Some(10));
    /// ```
    pub fn remaining_bytes(&self) -> Option<u64> {
        self.remaining_bytes
    }

    /// Returns `true` if it reaches the end of the sequence(EOS), otherwise `false`.
    ///
    /// "EOS" means that once the current buffer is consumed,
    /// no more bytes is available for decoding items.
    /// From an operational point of view, it means that the number of remaining bytes is `0`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::DecodeBuf;
    ///
    /// // The buffer is not empty but the remaining bytes is zero (EOS)
    /// let mut buf = DecodeBuf::with_remaining_bytes(b"foo", 0);
    /// assert!(buf.is_eos());
    ///
    /// // The buffer and remaining bytes are empty (EOS)
    /// buf.consume(3).unwrap();
    /// assert!(buf.is_eos());
    ///
    /// // There are some remaining bytes (not EOS)
    /// let buf = DecodeBuf::with_remaining_bytes(b"", 10);
    /// assert!(!buf.is_eos());
    ///
    /// // The number of remaining bytes is unknown (can not judge it is EOS)
    /// let buf = DecodeBuf::new(b"");
    /// assert!(!buf.is_eos());
    /// ```
    pub fn is_eos(&self) -> bool {
        self.remaining_bytes().map_or(false, |n| n == 0)
    }

    /// Consumes the specified number of the bytes from the beginning of this buffer.
    ///
    /// Note the invocation of the `Read::read()` method automatically consumes the read bytes.
    ///
    /// # Errors
    ///
    /// If `size` exceeds the length of the buffer,
    /// it will return an `ErrorKind::InvalidInput` error.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Read;
    /// use bytecodec::DecodeBuf;
    ///
    /// let mut buf = DecodeBuf::new(b"foo");
    /// assert_eq!(buf.as_ref(), b"foo");
    ///
    /// buf.consume(1).unwrap();
    /// assert_eq!(buf.as_ref(), b"oo");
    ///
    /// buf.read_to_end(&mut Vec::new()).unwrap();
    /// assert_eq!(buf.as_ref(), b"");
    ///
    /// assert!(buf.consume(1).is_err());
    /// ```
    pub fn consume(&mut self, size: usize) -> Result<()> {
        track_assert!(self.offset + size <= self.buf.len(), ErrorKind::InvalidInput;
                      self.offset, size, self.buf.len());
        self.offset += size;
        Ok(())
    }

    /// Consumes all bytes in the buffer.
    pub fn consume_all(&mut self) {
        self.offset = self.buf.len();
    }

    /// Executes the given function with the limited length decoding buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Read;
    /// use bytecodec::DecodeBuf;
    ///
    /// let mut buf = DecodeBuf::new(b"foobar");
    /// let s = buf.with_limit(3, |buf| {
    ///     let mut s = String::new();
    ///     buf.read_to_string(&mut s).unwrap();
    ///     s
    ///  });
    ///
    /// assert_eq!(s, "foo");
    /// assert_eq!(buf.as_ref(), b"bar");
    /// ```
    ///
    /// # Panics
    ///
    /// if `limit` exceeds the length of the buffer, the calling thread will panic.
    pub fn with_limit<F, T>(&mut self, limit: usize, f: F) -> T
    where
        F: FnOnce(&mut Self) -> T,
    {
        let (result, consumed_len) = {
            let mut buf = if let Some(remaining_bytes) = self.remaining_bytes() {
                let remaining_bytes = remaining_bytes + (self.len() - limit) as u64;
                DecodeBuf::with_remaining_bytes(&self.buf[self.offset..][..limit], remaining_bytes)
            } else {
                DecodeBuf::new(&self.buf[self.offset..][..limit])
            };
            let result = f(&mut buf);
            (result, limit - buf.len())
        };
        self.offset += consumed_len;
        result
    }

    /// Executes the given function with the limited length decoding buffer
    /// that have the specified remaining bytes information.
    ///
    /// # Panics
    ///
    /// if `limit` exceeds the length of the buffer, the calling thread will panic.
    pub fn with_limit_and_remaining_bytes<F, T>(
        &mut self,
        limit: usize,
        remaining_bytes: u64,
        f: F,
    ) -> T
    where
        F: FnOnce(&mut Self) -> T,
    {
        let (result, consumed_len) = {
            let mut buf =
                DecodeBuf::with_remaining_bytes(&self.buf[self.offset..][..limit], remaining_bytes);
            let result = f(&mut buf);
            (result, limit - buf.len())
        };
        self.offset += consumed_len;
        result
    }
}
impl<'a> AsRef<[u8]> for DecodeBuf<'a> {
    fn as_ref(&self) -> &[u8] {
        &self.buf[self.offset..]
    }
}
impl<'a> Deref for DecodeBuf<'a> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
impl<'a> Read for DecodeBuf<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let size = cmp::min(self.len(), buf.len());
        (&mut buf[..size]).copy_from_slice(&self[..size]);
        self.offset += size;
        Ok(size)
    }
}

/// Encoding buffer.
///
/// A `EncodeBuf` represents a slice of a byte sequence to which an encoder write encoded items.
#[derive(Debug)]
pub struct EncodeBuf<'a> {
    buf: &'a mut [u8],
    offset: usize,
    eos: bool,
}
impl<'a> EncodeBuf<'a> {
    /// Makes a new `EncodeBuf` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::EncodeBuf;
    ///
    /// let mut inner = [0; 3];
    /// let buf = EncodeBuf::new(&mut inner[..]);
    /// assert_eq!(buf.len(), 3);
    /// assert!(!buf.is_eos());
    /// ```
    pub fn new(buf: &'a mut [u8]) -> Self {
        EncodeBuf {
            buf,
            offset: 0,
            eos: false,
        }
    }

    /// Makes a new `EncodeBuf` instance with the given EOS indication.
    ///
    /// ```
    /// use bytecodec::EncodeBuf;
    ///
    /// let mut inner = [0; 3];
    /// let buf = EncodeBuf::with_eos(&mut inner[..], true);
    /// assert!(buf.is_eos());
    ///
    /// let mut inner = [0; 3];
    /// let buf = EncodeBuf::with_eos(&mut inner[..], false);
    /// assert!(!buf.is_eos());
    /// ```
    pub fn with_eos(buf: &'a mut [u8], eos: bool) -> Self {
        EncodeBuf {
            buf,
            offset: 0,
            eos,
        }
    }

    /// Returns `true` if it reaches the end of the sequence(EOS), otherwise `false`.
    ///
    /// "EOS" means that once the current buffer is consumed,
    /// no more space is available for writing encoded items.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::EncodeBuf;
    ///
    /// // Not EOS
    /// let mut inner = [0; 3];
    /// let buf = EncodeBuf::new(&mut inner[..]);
    /// assert!(!buf.is_eos());
    ///
    /// // EOS
    /// let mut inner = [0; 3];
    /// let buf = EncodeBuf::with_eos(&mut inner[..], true);
    /// assert!(buf.is_eos());
    /// ```
    pub fn is_eos(&self) -> bool {
        self.eos
    }

    /// Consumes the specified number of the bytes from the beginning of this buffer.
    ///
    /// Note the invocation of the `Write::write()` method automatically consumes the written bytes.
    ///
    /// # Errors
    ///
    /// If `size` exceeds the length of the buffer,
    /// it will return an `ErrorKind::InvalidInput` error.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Write;
    /// use bytecodec::EncodeBuf;
    ///
    /// let mut inner = [0; 3];
    /// let mut buf = EncodeBuf::new(&mut inner[..]);
    /// assert_eq!(buf.len(), 3);
    ///
    /// buf.consume(1).unwrap();
    /// assert_eq!(buf.len(), 2);
    ///
    /// buf.write_all(&[0; 2][..]).unwrap();
    /// assert_eq!(buf.len(), 0);
    ///
    /// assert!(buf.consume(1).is_err());
    /// ```
    pub fn consume(&mut self, size: usize) -> Result<()> {
        track_assert!(self.offset + size <= self.len(), ErrorKind::InvalidInput;
                      self.offset, size, self.len());
        self.offset += size;
        Ok(())
    }

    /// Consumes all bytes in the buffer.
    pub fn consume_all(&mut self) {
        self.offset = self.buf.len();
    }

    /// Executes the given function with the limited length encoding buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Write;
    /// use bytecodec::EncodeBuf;
    ///
    /// let mut inner = [0; 10];
    /// let mut buf = EncodeBuf::new(&mut inner);
    /// let size = buf.with_limit(3, |buf| buf.write(b"foobar").unwrap());
    ///
    /// assert_eq!(size, 3);
    /// assert_eq!(buf.len(), 7);
    /// ```
    ///
    /// # Panics
    ///
    /// if `limit` exceeds the length of the buffer, the calling thread will panic.
    pub fn with_limit<F, T>(&mut self, limit: usize, f: F) -> T
    where
        F: FnOnce(&mut EncodeBuf) -> T,
    {
        if self.len() == limit {
            f(self)
        } else {
            let (result, consumed_len) = {
                let mut buf = EncodeBuf::new(&mut self.buf[self.offset..][..limit]);
                let result = f(&mut buf);
                (result, limit - buf.len())
            };
            self.offset += consumed_len;
            result
        }
    }

    /// Executes the given function with the limited length encoding buffer and the EOS indication.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Write;
    /// use bytecodec::EncodeBuf;
    ///
    /// let mut inner = [0; 10];
    /// let mut buf = EncodeBuf::new(&mut inner);
    /// let size = buf.with_limit_and_eos(3, true, |buf| {
    ///     assert!(buf.is_eos());
    ///     buf.write(b"foobar").unwrap()
    /// });
    ///
    /// assert_eq!(size, 3);
    /// assert_eq!(buf.len(), 7);
    /// ```
    ///
    /// # Panics
    ///
    /// if `limit` exceeds the length of the buffer, the calling thread will panic.
    pub fn with_limit_and_eos<F, T>(&mut self, limit: usize, eos: bool, f: F) -> T
    where
        F: FnOnce(&mut EncodeBuf) -> T,
    {
        let (result, consumed_len) = {
            let mut buf = EncodeBuf::with_eos(&mut self.buf[self.offset..][..limit], eos);
            let result = f(&mut buf);
            (result, limit - buf.len())
        };
        self.offset += consumed_len;
        result
    }
}
impl<'a> AsRef<[u8]> for EncodeBuf<'a> {
    fn as_ref(&self) -> &[u8] {
        &self.buf[self.offset..]
    }
}
impl<'a> AsMut<[u8]> for EncodeBuf<'a> {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.buf[self.offset..]
    }
}
impl<'a> Deref for EncodeBuf<'a> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
impl<'a> DerefMut for EncodeBuf<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}
impl<'a> Write for EncodeBuf<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let size = cmp::min(self.len(), buf.len());
        (&mut self.as_mut()[..size]).copy_from_slice(&buf[..size]);
        self.offset += size;
        Ok(size)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
