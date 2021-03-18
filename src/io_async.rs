//! I/O (i.e., `Read` and `Write` traits) related module.
use crate::io::{BufferedIo, ReadBuf, StreamState, WriteBuf};
use crate::{Error, Result};
use core::pin::Pin;
use core::task::{Context, Poll as Poll03};
use tokio::io::{AsyncRead, AsyncWrite};

impl<B: AsRef<[u8]> + AsMut<[u8]>> ReadBuf<B> {
    /// Fills the read buffer by reading bytes from the given reader.
    ///
    /// The fill process continues until one of the following condition is satisfied:
    /// - The read buffer became full
    /// - A read operation returned a `WouldBlock` error
    /// - The input stream has reached EOS
    pub fn poll_fill<R: AsyncRead>(
        &mut self,
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

impl<T: AsyncRead + AsyncWrite> BufferedIo<T> {
    /// Executes an I/O operation on the inner stream.
    ///
    /// "I/O operation" means "filling the read buffer" and "flushing the write buffer".
    pub fn execute_io_poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Result<()> {
        let mut this = self.project();
        track!(this.rbuf.poll_fill(this.stream.as_mut(), cx))?;
        track!(this.wbuf.poll_flush(this.stream.as_mut(), cx))?;
        Ok(())
    }
}
