//! Null decoder and encoder.
use crate::{ByteCount, Decode, Encode, Eos, Result, SizedEncode};

/// Null decoder.
///
/// `NullDecoder` consumes no bytes and returns `Ok(())` when `finish_decoding` method is called.
#[derive(Debug, Default)]
pub struct NullDecoder;
impl Decode for NullDecoder {
    type Item = ();

    fn decode(&mut self, _buf: &[u8], _eos: Eos) -> Result<usize> {
        Ok(0)
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        Ok(())
    }

    fn is_idle(&self) -> bool {
        true
    }

    fn requiring_bytes(&self) -> ByteCount {
        ByteCount::Finite(0)
    }
}

/// Null encoder.
///
/// `NullEncoder` produces no bytes.
#[derive(Debug, Default)]
pub struct NullEncoder;
impl Encode for NullEncoder {
    type Item = ();

    fn encode(&mut self, _buf: &mut [u8], _eos: Eos) -> Result<usize> {
        Ok(0)
    }

    fn start_encoding(&mut self, _item: Self::Item) -> Result<()> {
        Ok(())
    }

    fn requiring_bytes(&self) -> ByteCount {
        ByteCount::Finite(0)
    }

    fn is_idle(&self) -> bool {
        true
    }
}
impl SizedEncode for NullEncoder {
    fn exact_requiring_bytes(&self) -> u64 {
        0
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn null_decoder_works() {
        let mut decoder = NullDecoder;
        assert_eq!(decoder.decode(&[1][..], Eos::new(true)).ok(), Some(0));
        assert_eq!(decoder.finish_decoding().ok(), Some(()));
        assert_eq!(decoder.finish_decoding().ok(), Some(()));
    }

    #[test]
    fn null_encoder_works() {
        let mut encoder = NullEncoder;
        encoder.start_encoding(()).unwrap();
        assert_eq!(encoder.is_idle(), true);

        let mut buf = [0; 10];
        assert_eq!(encoder.encode(&mut buf[..], Eos::new(true)).ok(), Some(0));
        assert_eq!(encoder.is_idle(), true);
    }
}
