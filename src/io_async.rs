//! I/O (i.e., `Read` and `Write` traits) related module.
use crate::io::{ReadBuf, StreamState, WriteBuf};
use crate::{Error, Result};
use core::pin::Pin;
use core::task::{Context, Poll as Poll03};
use pin_project::pin_project;
use tokio::io::{AsyncRead, AsyncWrite};

impl<B: AsRef<[u8]> + AsMut<[u8]>> ReadBuf<B> {
    /// Fills the read buffer by reading bytes from the given reader.
    ///
    /// The fill process continues until one of the following condition is satisfied:
    /// - The read buffer became full
    /// - A read operation returned a `WouldBlock` error
    /// - The input stream has reached EOS
    pub fn poll_fill<R: AsyncRead>(
        self: &mut Self,
        mut reader: Pin<&mut R>,
        cx: &mut Context<'_>,
    ) -> Result<()> {
        while !self.is_full() {
            let mut buffer = tokio::io::ReadBuf::new(&mut self.inner.as_mut()[self.tail..]);
            match reader.as_mut().poll_read(cx, &mut buffer) {
                Poll03::Pending => {
                    self.stream_state = StreamState::WouldBlock;
                    break;
                }
                Poll03::Ready(Ok(())) => {
                    let size = buffer.filled().len();
                    if size == 0 {
                        self.stream_state = StreamState::Eos;
                        break;
                    }
                    self.stream_state = StreamState::Normal;
                    self.tail += size;
                }
                Poll03::Ready(Err(e)) => {
                    self.stream_state = StreamState::Error;
                    return Err(track!(Error::from(e)));
                }
            }
        }
        Ok(())
    }
}

impl<B: AsRef<[u8]> + AsMut<[u8]>> WriteBuf<B> {
    /// Writes the encoded bytes contained in this buffer to the given writer.
    ///
    /// The written bytes will be removed from the buffer.
    ///
    /// The flush process continues until one of the following condition is satisfied:
    /// - The write buffer became empty
    /// - A write operation returned a `WouldBlock` error
    /// - The output stream has reached EOS
    pub fn poll_flush<W: AsyncWrite>(
        &mut self,
        mut writer: Pin<&mut W>,
        cx: &mut Context<'_>,
    ) -> Result<()> {
        while !self.is_empty() {
            match writer
                .as_mut()
                .poll_write(cx, &self.inner.as_ref()[self.head..self.tail])
            {
                Poll03::Ready(Err(e)) => {
                    self.stream_state = StreamState::Error;
                    return Err(track!(Error::from(e)));
                }
                Poll03::Ready(Ok(0)) => {
                    self.stream_state = StreamState::Eos;
                    break;
                }
                Poll03::Ready(Ok(size)) => {
                    self.stream_state = StreamState::Normal;
                    self.head += size;
                    if self.head == self.tail {
                        self.head = 0;
                        self.tail = 0;
                    }
                }
                Poll03::Pending => {
                    self.stream_state = StreamState::WouldBlock;
                    break;
                }
            }
        }
        Ok(())
    }
}

/// Buffered I/O stream.
#[pin_project]
#[derive(Debug)]
pub struct BufferedIo<T> {
    #[pin]
    stream: T,
    rbuf: ReadBuf<Vec<u8>>,
    wbuf: WriteBuf<Vec<u8>>,
}
impl<T: AsyncRead + AsyncWrite> BufferedIo<T> {
    /// Makes a new `BufferedIo` instance.
    pub fn new(stream: T, read_buf_size: usize, write_buf_size: usize) -> Self {
        BufferedIo {
            stream,
            rbuf: ReadBuf::new(vec![0; read_buf_size]),
            wbuf: WriteBuf::new(vec![0; write_buf_size]),
        }
    }

    /// Executes an I/O operation on the inner stream.
    ///
    /// "I/O operation" means "filling the read buffer" and "flushing the write buffer".
    pub fn execute_io(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Result<()> {
        let mut this = self.project();
        track!(this.rbuf.poll_fill(this.stream.as_mut(), cx))?;
        track!(this.wbuf.poll_flush(this.stream.as_mut(), cx))?;
        Ok(())
    }

    /// Returns `true` if the inner stream reaches EOS, otherwise `false`.
    pub fn is_eos(&self) -> bool {
        self.rbuf.stream_state().is_eos() || self.wbuf.stream_state().is_eos()
    }

    /// Returns `true` if the previous I/O operation on the inner stream would block, otherwise `false`.
    pub fn would_block(&self) -> bool {
        self.rbuf.stream_state().would_block()
            && (self.wbuf.is_empty() || self.wbuf.stream_state().would_block())
    }

    /// Returns a reference to the read buffer of the instance.
    pub fn read_buf_ref(&self) -> &ReadBuf<Vec<u8>> {
        &self.rbuf
    }

    /// Returns a mutable reference to the read buffer of the instance.
    pub fn read_buf_mut(&mut self) -> &mut ReadBuf<Vec<u8>> {
        &mut self.rbuf
    }

    /// Returns a reference to the write buffer of the instance.
    pub fn write_buf_ref(&self) -> &WriteBuf<Vec<u8>> {
        &self.wbuf
    }

    /// Returns a mutable reference to the write buffer of the instance.
    pub fn write_buf_mut(&mut self) -> &mut WriteBuf<Vec<u8>> {
        &mut self.wbuf
    }

    /// Returns a reference to the inner stream of the instance.
    pub fn stream_ref(&self) -> &T {
        &self.stream
    }

    /// Returns a mutable reference to the inner stream of the instance.
    pub fn stream_mut(&mut self) -> &mut T {
        &mut self.stream
    }

    /// Takes ownership of the instance, and returns the inner stream.
    pub fn into_stream(self) -> T {
        self.stream
    }
}

#[cfg(test)]
mod test {
    use std::io::{Read, Write};

    use super::*;
    use bytes::{Utf8Decoder, Utf8Encoder};
    use EncodeExt;

    #[test]
    fn decode_from_read_buf_works() {
        let mut buf = ReadBuf::new(vec![0; 1024]);
        track_try_unwrap!(buf.fill(b"foo".as_ref()));
        assert_eq!(buf.len(), 3);
        assert_eq!(buf.stream_state(), StreamState::Eos);

        let mut decoder = Utf8Decoder::new();
        track_try_unwrap!(decoder.decode_from_read_buf(&mut buf));
        assert_eq!(track_try_unwrap!(decoder.finish_decoding()), "foo");
    }

    #[test]
    fn read_from_read_buf_works() {
        let mut rbuf = ReadBuf::new(vec![0; 1024]);
        track_try_unwrap!(rbuf.fill(b"foo".as_ref()));
        assert_eq!(rbuf.len(), 3);
        assert_eq!(rbuf.stream_state(), StreamState::Eos);

        let mut buf = Vec::new();
        rbuf.read_to_end(&mut buf).unwrap();
        assert_eq!(buf, b"foo");
        assert_eq!(rbuf.len(), 0);
    }

    #[test]
    fn encode_to_write_buf_works() {
        let mut encoder = track_try_unwrap!(Utf8Encoder::with_item("foo"));

        let mut buf = WriteBuf::new(vec![0; 1024]);
        track_try_unwrap!(encoder.encode_to_write_buf(&mut buf));
        assert_eq!(buf.len(), 3);

        let mut v = Vec::new();
        track_try_unwrap!(buf.flush(&mut v));
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.stream_state(), StreamState::Normal);
        assert_eq!(v, b"foo");
    }

    #[test]
    fn write_to_write_buf_works() {
        let mut buf = WriteBuf::new(vec![0; 1024]);
        buf.write_all(b"foo").unwrap();
        assert_eq!(buf.len(), 3);

        let mut v = Vec::new();
        track_try_unwrap!(buf.flush(&mut v));
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.stream_state(), StreamState::Normal);
        assert_eq!(v, b"foo");
    }
}
