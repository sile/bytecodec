//! Already decoded value.
use {ByteCount, Decode, Eos, ErrorKind, Result};

/// `DecodedValue` represents an already decoded item.
///
/// It does not consume any bytes, and returns the given item when `decode()` was first called.
///
/// # Examples
///
/// ```
/// use bytecodec::{Decode, Eos};
/// use bytecodec::value::DecodedValue;
///
/// let mut decoder = DecodedValue::new(10);
///
/// let (size, item) = decoder.decode(b"foo", Eos::new(false)).unwrap();
/// assert_eq!(item, Some(10));
/// assert_eq!(size, 0);
/// ```
#[derive(Debug)]
pub struct DecodedValue<T>(Option<T>);
impl<T> DecodedValue<T> {
    /// Makes a new `DecodedValue` instance.
    pub fn new(value: T) -> Self {
        DecodedValue(Some(value))
    }
}
impl<T> Decode for DecodedValue<T> {
    type Item = T;

    fn decode(&mut self, _buf: &[u8], _eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        let item = track_assert_some!(self.0.take(), ErrorKind::DecoderTerminated);
        Ok((0, Some(item)))
    }

    fn has_terminated(&self) -> bool {
        self.0.is_none()
    }

    fn requiring_bytes(&self) -> ByteCount {
        ByteCount::Finite(0)
    }
}

/// `NullDecoder` always and immediately returns `()` as the decoded items.
#[derive(Debug, Default)]
pub struct NullDecoder;
impl Decode for NullDecoder {
    type Item = ();

    fn decode(&mut self, _buf: &[u8], _eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        Ok((0, Some(())))
    }

    fn has_terminated(&self) -> bool {
        false
    }

    fn requiring_bytes(&self) -> ByteCount {
        ByteCount::Finite(0)
    }
}

#[cfg(test)]
mod test {
    use {Decode, Eos, ErrorKind};
    use super::*;

    #[test]
    fn decoded_value_works() {
        let mut decoder = DecodedValue::new(3);
        assert_eq!(
            decoder.decode(&[][..], Eos::new(false)).unwrap(),
            (0, Some(3))
        );
        assert_eq!(
            decoder
                .decode(&[][..], Eos::new(false))
                .err()
                .map(|e| *e.kind()),
            Some(ErrorKind::DecoderTerminated)
        );
    }

    #[test]
    fn null_decoder_works() {
        let mut decoder = NullDecoder;
        assert_eq!(
            decoder.decode(&[][..], Eos::new(false)).unwrap(),
            (0, Some(()))
        );
        assert_eq!(
            decoder.decode(&[][..], Eos::new(false)).unwrap(),
            (0, Some(()))
        );
    }
}
