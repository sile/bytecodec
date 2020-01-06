//! Encoders and decoders for tuples.
use crate::{ByteCount, Decode, Encode, Eos, Result, SizedEncode};

/// Decoder for tuples.
#[derive(Debug, Default)]
pub struct TupleDecoder<D> {
    inner: D,
}
impl<D> TupleDecoder<D> {
    /// Makes a new `TupleDecoder`.
    pub fn new(inner: D) -> Self {
        TupleDecoder { inner }
    }

    /// Returns a reference to the inner decoders.
    pub fn inner_ref(&self) -> &D {
        &self.inner
    }

    /// Returns a mutable reference to the inner decoders.
    pub fn inner_mut(&mut self) -> &mut D {
        &mut self.inner
    }

    /// Takes ownership of this instance and returns the inner decoders.
    pub fn into_inner(self) -> D {
        self.inner
    }
}

macro_rules! impl_decode {
    ([$($t:ident),*],[$($i:tt),*]) => {
        impl<$($t),*> Decode for TupleDecoder<($($t),*,)>
        where
            $($t: Decode),*
        {
            type Item = ($($t::Item),*,);

            fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
                let mut offset = 0;
                $(bytecodec_try_decode!(self.inner.$i, offset, buf, eos, "i={}", $i);)*
                Ok(offset)
            }

            fn finish_decoding(&mut self) -> Result<Self::Item> {
                Ok((
                    $(track!(self.inner.$i.finish_decoding(), "i={}", $i)?),*,
                ))
            }

            fn requiring_bytes(&self) -> ByteCount {
                ByteCount::Finite(0)$(.add_for_decoding(self.inner.$i.requiring_bytes()))*
            }

            fn is_idle(&self) -> bool {
                $(self.inner.$i.is_idle())&&*
            }
        }
    }
}
impl_decode!([D0, D1], [0, 1]);
impl_decode!([D0, D1, D2], [0, 1, 2]);
impl_decode!([D0, D1, D2, D3], [0, 1, 2, 3]);
impl_decode!([D0, D1, D2, D3, D4], [0, 1, 2, 3, 4]);
impl_decode!([D0, D1, D2, D3, D4, D5], [0, 1, 2, 3, 4, 5]);
impl_decode!([D0, D1, D2, D3, D4, D5, D6], [0, 1, 2, 3, 4, 5, 6]);
impl_decode!([D0, D1, D2, D3, D4, D5, D6, D7], [0, 1, 2, 3, 4, 5, 6, 7]);

/// Encoder for tuples.
#[derive(Debug, Default)]
pub struct TupleEncoder<E> {
    inner: E,
}
impl<E> TupleEncoder<E> {
    /// Makes a new `TupleEncoder`.
    pub fn new(inner: E) -> Self {
        TupleEncoder { inner }
    }

    /// Returns a reference to the inner encoders.
    pub fn inner_ref(&self) -> &E {
        &self.inner
    }

    /// Returns a mutable reference to the inner encoders.
    pub fn inner_mut(&mut self) -> &mut E {
        &mut self.inner
    }

    /// Takes ownership of this instance and returns the inner encoders.
    pub fn into_inner(self) -> E {
        self.inner
    }
}

macro_rules! impl_encode {
    ([$($t:ident),*],[$($i:tt),*]) => {
        impl<$($t),*> Encode for TupleEncoder<($($t),*,)>
        where
            $($t: Encode),*
        {
            type Item = ($($t::Item),*,);

            fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
                let mut offset = 0;
                $(bytecodec_try_encode!(self.inner.$i, offset, buf, eos, "i={}", $i);)*
                Ok(offset)
            }

            fn start_encoding(&mut self, t: Self::Item) -> Result<()> {
                $(track!(self.inner.$i.start_encoding(t.$i), "i={}", $i)?;)*
                Ok(())
            }

            fn requiring_bytes(&self) -> ByteCount {
                ByteCount::Finite(0)$(.add_for_encoding(self.inner.$i.requiring_bytes()))*
            }

            fn is_idle(&self) -> bool {
                $(self.inner.$i.is_idle())&&*
            }
        }
        impl<$($t),*> SizedEncode for TupleEncoder<($($t),*,)>
        where
            $($t: SizedEncode),*
        {
            fn exact_requiring_bytes(&self) -> u64 {
                0 $(+ self.inner.$i.exact_requiring_bytes())*
            }
        }
    };
}
impl_encode!([E0, E1], [0, 1]);
impl_encode!([E0, E1, E2], [0, 1, 2]);
impl_encode!([E0, E1, E2, E3], [0, 1, 2, 3]);
impl_encode!([E0, E1, E2, E3, E4], [0, 1, 2, 3, 4]);
impl_encode!([E0, E1, E2, E3, E4, E5], [0, 1, 2, 3, 4, 5]);
impl_encode!([E0, E1, E2, E3, E4, E5, E6], [0, 1, 2, 3, 4, 5, 6]);
impl_encode!([E0, E1, E2, E3, E4, E5, E6, E7], [0, 1, 2, 3, 4, 5, 6, 7]);

#[cfg(test)]
mod test {
    use super::*;
    use crate::fixnum::{U8Decoder, U8Encoder};
    use crate::io::{IoDecodeExt, IoEncodeExt};
    use crate::EncodeExt;

    #[test]
    fn tuple_decoder_works() {
        let mut decoder = TupleDecoder::new((U8Decoder::new(), U8Decoder::new()));
        assert_eq!(
            track_try_unwrap!(decoder.decode_exact(b"foo".as_ref())),
            (b'f', b'o')
        );
    }

    #[test]
    fn tuple_encoder_works() {
        let mut encoder = TupleEncoder::<(U8Encoder, U8Encoder)>::with_item((0, 1)).unwrap();
        let mut buf = Vec::new();
        encoder.encode_all(&mut buf).unwrap();
        assert_eq!(buf, [0, 1]);
    }
}
