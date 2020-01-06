//! Encoder and decoder for padding bytes.
use crate::{ByteCount, Decode, Encode, Eos, ErrorKind, Result};

/// Decoder for reading padding bytes from input streams.
///
/// `PaddingDecoder` discards any bytes in a stream until it reaches EOS.
#[derive(Debug, Default)]
pub struct PaddingDecoder {
    expected_byte: Option<u8>,
    eos: bool,
}
impl PaddingDecoder {
    /// Makes a new `PaddingDecoder` instance.
    pub fn new(expected_byte: Option<u8>) -> Self {
        PaddingDecoder {
            expected_byte,
            eos: false,
        }
    }

    /// Returns the expected byte used for padding.
    ///
    /// `None` means that this decoder accepts any bytes.
    pub fn expected_byte(&self) -> Option<u8> {
        self.expected_byte
    }

    /// Sets the expected byte used for padding.
    pub fn set_expected_byte(&mut self, b: Option<u8>) {
        self.expected_byte = b;
    }
}
impl Decode for PaddingDecoder {
    type Item = ();

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        if self.eos {
            Ok(0)
        } else if let Some(expected) = self.expected_byte {
            for &padding_byte in buf {
                track_assert_eq!(padding_byte, expected, ErrorKind::InvalidInput);
            }
            self.eos = eos.is_reached();
            Ok(buf.len())
        } else {
            self.eos = eos.is_reached();
            Ok(buf.len())
        }
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        track_assert!(self.eos, ErrorKind::IncompleteDecoding);
        self.eos = false;
        Ok(())
    }

    fn is_idle(&self) -> bool {
        self.eos
    }

    fn requiring_bytes(&self) -> ByteCount {
        if self.eos {
            ByteCount::Finite(0)
        } else {
            ByteCount::Infinite
        }
    }
}

/// Encoder for writing padding bytes to output streams.
///
/// After `start_encoding` is called, it will write the specified padding byte repeatedly
/// until it the output stream reaches EOS.
#[derive(Debug, Default)]
pub struct PaddingEncoder {
    padding_byte: Option<u8>,
}
impl PaddingEncoder {
    /// Makes a new `PaddingEncoder` instance.
    pub fn new() -> Self {
        Self::default()
    }
}
impl Encode for PaddingEncoder {
    type Item = u8;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        if let Some(padding_byte) = self.padding_byte {
            for b in &mut buf[..] {
                *b = padding_byte
            }
            if eos.is_reached() {
                self.padding_byte = None;
            }
            Ok(buf.len())
        } else {
            Ok(0)
        }
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track_assert!(self.is_idle(), ErrorKind::EncoderFull);
        self.padding_byte = Some(item);
        Ok(())
    }

    fn requiring_bytes(&self) -> ByteCount {
        if self.is_idle() {
            ByteCount::Finite(0)
        } else {
            ByteCount::Infinite
        }
    }

    fn is_idle(&self) -> bool {
        self.padding_byte.is_none()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::io::IoDecodeExt;
    use crate::{Encode, EncodeExt, Eos};

    #[test]
    fn padding_encoder_works() {
        let mut encoder = track_try_unwrap!(PaddingEncoder::with_item(3));
        let mut buf = [0; 8];
        track_try_unwrap!(encoder.encode(&mut buf[..], Eos::new(true)));
        assert_eq!(buf, [3; 8]);
        assert!(encoder.is_idle());
    }

    #[test]
    fn padding_decoder_works() {
        let mut decoder = PaddingDecoder::new(None);
        assert!(decoder.decode_exact(&[0; 8][..]).is_ok());

        let mut decoder = PaddingDecoder::new(Some(1));
        assert!(decoder.decode_exact(&[1; 8][..]).is_ok());
        assert!(decoder.decode_exact(&[0; 8][..]).is_err());
    }
}
