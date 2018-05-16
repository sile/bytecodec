//! Encoders and decoders for tuples.
#![cfg_attr(feature = "cargo-clippy", allow(type_complexity, too_many_arguments))]
use combinator::Buffered;
use {ByteCount, CalculateBytes, Decode, DecodeExt, Encode, Eos, ExactBytesEncode, Result};

/// Decoder for 2-elements tuples.
#[derive(Debug, Default)]
pub struct Tuple2Decoder<D0, D1>
where
    D0: Decode,
    D1: Decode,
{
    d0: Buffered<D0>,
    d1: D1,
}
impl<D0, D1> Tuple2Decoder<D0, D1>
where
    D0: Decode,
    D1: Decode,
{
    /// Makes a new `Tuple2Decoder` instance.
    pub fn new(d0: D0, d1: D1) -> Self {
        Tuple2Decoder {
            d0: d0.buffered(),
            d1,
        }
    }

    /// Returns references to the inner decoders.
    pub fn inner_ref(&self) -> (&D0, &D1) {
        (self.d0.inner_ref(), &self.d1)
    }

    /// Returns mutable references to the inner decoders.
    pub fn inner_mut(&mut self) -> (&mut D0, &mut D1) {
        (self.d0.inner_mut(), &mut self.d1)
    }
}
impl<D0, D1> Decode for Tuple2Decoder<D0, D1>
where
    D0: Decode,
    D1: Decode,
{
    type Item = (D0::Item, D1::Item);

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        let mut offset = 0;
        bytecodec_try_decode!(self.d0, offset, buf, eos);

        let (size, item) = track!(self.d1.decode(&buf[offset..], eos))?;
        offset += size;

        let item = item.map(|d1| (self.d0.take_item().expect("Never fails"), d1));
        Ok((offset, item))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.d0
            .requiring_bytes()
            .add_for_decoding(self.d1.requiring_bytes())
    }
}

/// Decoder for 3-elements tuples.
#[derive(Debug, Default)]
pub struct Tuple3Decoder<D0, D1, D2>
where
    D0: Decode,
    D1: Decode,
    D2: Decode,
{
    d0: Buffered<D0>,
    d1: Buffered<D1>,
    d2: D2,
}
impl<D0, D1, D2> Tuple3Decoder<D0, D1, D2>
where
    D0: Decode,
    D1: Decode,
    D2: Decode,
{
    /// Makes a new `Tuple3Decoder` instance.
    pub fn new(d0: D0, d1: D1, d2: D2) -> Self {
        Tuple3Decoder {
            d0: d0.buffered(),
            d1: d1.buffered(),
            d2,
        }
    }

    /// Returns references to the inner decoders.
    pub fn inner_ref(&self) -> (&D0, &D1, &D2) {
        (self.d0.inner_ref(), self.d1.inner_ref(), &self.d2)
    }

    /// Returns mutable references to the inner decoders.
    pub fn inner_mut(&mut self) -> (&mut D0, &mut D1, &mut D2) {
        (self.d0.inner_mut(), self.d1.inner_mut(), &mut self.d2)
    }
}
impl<D0, D1, D2> Decode for Tuple3Decoder<D0, D1, D2>
where
    D0: Decode,
    D1: Decode,
    D2: Decode,
{
    type Item = (D0::Item, D1::Item, D2::Item);

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        let mut offset = 0;
        bytecodec_try_decode!(self.d0, offset, buf, eos);
        bytecodec_try_decode!(self.d1, offset, buf, eos);

        let (size, item) = track!(self.d2.decode(&buf[offset..], eos))?;
        offset += size;

        let item = item.map(|d2| {
            let d0 = self.d0.take_item().expect("Never fails");
            let d1 = self.d1.take_item().expect("Never fails");
            (d0, d1, d2)
        });
        Ok((offset, item))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.d0
            .requiring_bytes()
            .add_for_decoding(self.d1.requiring_bytes())
            .add_for_decoding(self.d2.requiring_bytes())
    }
}

/// Decoder for 4-elements tuples.
#[derive(Debug, Default)]
pub struct Tuple4Decoder<D0, D1, D2, D3>
where
    D0: Decode,
    D1: Decode,
    D2: Decode,
    D3: Decode,
{
    d0: Buffered<D0>,
    d1: Buffered<D1>,
    d2: Buffered<D2>,
    d3: D3,
}
impl<D0, D1, D2, D3> Tuple4Decoder<D0, D1, D2, D3>
where
    D0: Decode,
    D1: Decode,
    D2: Decode,
    D3: Decode,
{
    /// Makes a new `Tuple4Decoder` instance.
    pub fn new(d0: D0, d1: D1, d2: D2, d3: D3) -> Self {
        Tuple4Decoder {
            d0: d0.buffered(),
            d1: d1.buffered(),
            d2: d2.buffered(),
            d3,
        }
    }

    /// Returns references to the inner decoders.
    pub fn inner_ref(&self) -> (&D0, &D1, &D2, &D3) {
        (
            self.d0.inner_ref(),
            self.d1.inner_ref(),
            self.d2.inner_ref(),
            &self.d3,
        )
    }

    /// Returns mutable references to the inner decoders.
    pub fn inner_mut(&mut self) -> (&mut D0, &mut D1, &mut D2, &mut D3) {
        (
            self.d0.inner_mut(),
            self.d1.inner_mut(),
            self.d2.inner_mut(),
            &mut self.d3,
        )
    }
}
impl<D0, D1, D2, D3> Decode for Tuple4Decoder<D0, D1, D2, D3>
where
    D0: Decode,
    D1: Decode,
    D2: Decode,
    D3: Decode,
{
    type Item = (D0::Item, D1::Item, D2::Item, D3::Item);

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        let mut offset = 0;
        bytecodec_try_decode!(self.d0, offset, buf, eos);
        bytecodec_try_decode!(self.d1, offset, buf, eos);
        bytecodec_try_decode!(self.d2, offset, buf, eos);

        let (size, item) = track!(self.d3.decode(&buf[offset..], eos))?;
        offset += size;

        let item = item.map(|d3| {
            let d0 = self.d0.take_item().expect("Never fails");
            let d1 = self.d1.take_item().expect("Never fails");
            let d2 = self.d2.take_item().expect("Never fails");
            (d0, d1, d2, d3)
        });
        Ok((offset, item))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.d0
            .requiring_bytes()
            .add_for_decoding(self.d1.requiring_bytes())
            .add_for_decoding(self.d2.requiring_bytes())
            .add_for_decoding(self.d3.requiring_bytes())
    }
}

/// Decoder for 5-elements tuples.
#[derive(Debug, Default)]
pub struct Tuple5Decoder<D0, D1, D2, D3, D4>
where
    D0: Decode,
    D1: Decode,
    D2: Decode,
    D3: Decode,
    D4: Decode,
{
    d0: Buffered<D0>,
    d1: Buffered<D1>,
    d2: Buffered<D2>,
    d3: Buffered<D3>,
    d4: D4,
}
impl<D0, D1, D2, D3, D4> Tuple5Decoder<D0, D1, D2, D3, D4>
where
    D0: Decode,
    D1: Decode,
    D2: Decode,
    D3: Decode,
    D4: Decode,
{
    /// Makes a new `Tuple5Decoder` instance.
    pub fn new(d0: D0, d1: D1, d2: D2, d3: D3, d4: D4) -> Self {
        Tuple5Decoder {
            d0: d0.buffered(),
            d1: d1.buffered(),
            d2: d2.buffered(),
            d3: d3.buffered(),
            d4,
        }
    }

    /// Returns references to the inner decoders.
    pub fn inner_ref(&self) -> (&D0, &D1, &D2, &D3, &D4) {
        (
            self.d0.inner_ref(),
            self.d1.inner_ref(),
            self.d2.inner_ref(),
            self.d3.inner_ref(),
            &self.d4,
        )
    }

    /// Returns mutable references to the inner decoders.
    pub fn inner_mut(&mut self) -> (&mut D0, &mut D1, &mut D2, &mut D3, &mut D4) {
        (
            self.d0.inner_mut(),
            self.d1.inner_mut(),
            self.d2.inner_mut(),
            self.d3.inner_mut(),
            &mut self.d4,
        )
    }
}
impl<D0, D1, D2, D3, D4> Decode for Tuple5Decoder<D0, D1, D2, D3, D4>
where
    D0: Decode,
    D1: Decode,
    D2: Decode,
    D3: Decode,
    D4: Decode,
{
    type Item = (D0::Item, D1::Item, D2::Item, D3::Item, D4::Item);

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        let mut offset = 0;
        bytecodec_try_decode!(self.d0, offset, buf, eos);
        bytecodec_try_decode!(self.d1, offset, buf, eos);
        bytecodec_try_decode!(self.d2, offset, buf, eos);
        bytecodec_try_decode!(self.d3, offset, buf, eos);

        let (size, item) = track!(self.d4.decode(&buf[offset..], eos))?;
        offset += size;

        let item = item.map(|d4| {
            let d0 = self.d0.take_item().expect("Never fails");
            let d1 = self.d1.take_item().expect("Never fails");
            let d2 = self.d2.take_item().expect("Never fails");
            let d3 = self.d3.take_item().expect("Never fails");
            (d0, d1, d2, d3, d4)
        });
        Ok((offset, item))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.d0
            .requiring_bytes()
            .add_for_decoding(self.d1.requiring_bytes())
            .add_for_decoding(self.d2.requiring_bytes())
            .add_for_decoding(self.d3.requiring_bytes())
            .add_for_decoding(self.d4.requiring_bytes())
    }
}

/// Decoder for 6-elements tuples.
#[derive(Debug, Default)]
pub struct Tuple6Decoder<D0, D1, D2, D3, D4, D5>
where
    D0: Decode,
    D1: Decode,
    D2: Decode,
    D3: Decode,
    D4: Decode,
    D5: Decode,
{
    d0: Buffered<D0>,
    d1: Buffered<D1>,
    d2: Buffered<D2>,
    d3: Buffered<D3>,
    d4: Buffered<D4>,
    d5: D5,
}
impl<D0, D1, D2, D3, D4, D5> Tuple6Decoder<D0, D1, D2, D3, D4, D5>
where
    D0: Decode,
    D1: Decode,
    D2: Decode,
    D3: Decode,
    D4: Decode,
    D5: Decode,
{
    /// Makes a new `Tuple6Decoder` instance.
    pub fn new(d0: D0, d1: D1, d2: D2, d3: D3, d4: D4, d5: D5) -> Self {
        Tuple6Decoder {
            d0: d0.buffered(),
            d1: d1.buffered(),
            d2: d2.buffered(),
            d3: d3.buffered(),
            d4: d4.buffered(),
            d5,
        }
    }

    /// Returns references to the inner decoders.
    pub fn inner_ref(&self) -> (&D0, &D1, &D2, &D3, &D4, &D5) {
        (
            self.d0.inner_ref(),
            self.d1.inner_ref(),
            self.d2.inner_ref(),
            self.d3.inner_ref(),
            self.d4.inner_ref(),
            &self.d5,
        )
    }

    /// Returns mutable references to the inner decoders.
    pub fn inner_mut(&mut self) -> (&mut D0, &mut D1, &mut D2, &mut D3, &mut D4, &mut D5) {
        (
            self.d0.inner_mut(),
            self.d1.inner_mut(),
            self.d2.inner_mut(),
            self.d3.inner_mut(),
            self.d4.inner_mut(),
            &mut self.d5,
        )
    }
}
impl<D0, D1, D2, D3, D4, D5> Decode for Tuple6Decoder<D0, D1, D2, D3, D4, D5>
where
    D0: Decode,
    D1: Decode,
    D2: Decode,
    D3: Decode,
    D4: Decode,
    D5: Decode,
{
    type Item = (D0::Item, D1::Item, D2::Item, D3::Item, D4::Item, D5::Item);

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        let mut offset = 0;
        bytecodec_try_decode!(self.d0, offset, buf, eos);
        bytecodec_try_decode!(self.d1, offset, buf, eos);
        bytecodec_try_decode!(self.d2, offset, buf, eos);
        bytecodec_try_decode!(self.d3, offset, buf, eos);
        bytecodec_try_decode!(self.d4, offset, buf, eos);

        let (size, item) = track!(self.d5.decode(&buf[offset..], eos))?;
        offset += size;

        let item = item.map(|d5| {
            let d0 = self.d0.take_item().expect("Never fails");
            let d1 = self.d1.take_item().expect("Never fails");
            let d2 = self.d2.take_item().expect("Never fails");
            let d3 = self.d3.take_item().expect("Never fails");
            let d4 = self.d4.take_item().expect("Never fails");
            (d0, d1, d2, d3, d4, d5)
        });
        Ok((offset, item))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.d0
            .requiring_bytes()
            .add_for_decoding(self.d1.requiring_bytes())
            .add_for_decoding(self.d2.requiring_bytes())
            .add_for_decoding(self.d3.requiring_bytes())
            .add_for_decoding(self.d4.requiring_bytes())
            .add_for_decoding(self.d5.requiring_bytes())
    }
}

/// Decoder for 7-elements tuples.
#[derive(Debug, Default)]
pub struct Tuple7Decoder<D0, D1, D2, D3, D4, D5, D6>
where
    D0: Decode,
    D1: Decode,
    D2: Decode,
    D3: Decode,
    D4: Decode,
    D5: Decode,
    D6: Decode,
{
    d0: Buffered<D0>,
    d1: Buffered<D1>,
    d2: Buffered<D2>,
    d3: Buffered<D3>,
    d4: Buffered<D4>,
    d5: Buffered<D5>,
    d6: D6,
}
impl<D0, D1, D2, D3, D4, D5, D6> Tuple7Decoder<D0, D1, D2, D3, D4, D5, D6>
where
    D0: Decode,
    D1: Decode,
    D2: Decode,
    D3: Decode,
    D4: Decode,
    D5: Decode,
    D6: Decode,
{
    /// Makes a new `Tuple7Decoder` instance.
    pub fn new(d0: D0, d1: D1, d2: D2, d3: D3, d4: D4, d5: D5, d6: D6) -> Self {
        Tuple7Decoder {
            d0: d0.buffered(),
            d1: d1.buffered(),
            d2: d2.buffered(),
            d3: d3.buffered(),
            d4: d4.buffered(),
            d5: d5.buffered(),
            d6,
        }
    }

    /// Returns references to the inner decoders.
    pub fn inner_ref(&self) -> (&D0, &D1, &D2, &D3, &D4, &D5, &D6) {
        (
            self.d0.inner_ref(),
            self.d1.inner_ref(),
            self.d2.inner_ref(),
            self.d3.inner_ref(),
            self.d4.inner_ref(),
            self.d5.inner_ref(),
            &self.d6,
        )
    }

    /// Returns mutable references to the inner decoders.
    pub fn inner_mut(
        &mut self,
    ) -> (
        &mut D0,
        &mut D1,
        &mut D2,
        &mut D3,
        &mut D4,
        &mut D5,
        &mut D6,
    ) {
        (
            self.d0.inner_mut(),
            self.d1.inner_mut(),
            self.d2.inner_mut(),
            self.d3.inner_mut(),
            self.d4.inner_mut(),
            self.d5.inner_mut(),
            &mut self.d6,
        )
    }
}
impl<D0, D1, D2, D3, D4, D5, D6> Decode for Tuple7Decoder<D0, D1, D2, D3, D4, D5, D6>
where
    D0: Decode,
    D1: Decode,
    D2: Decode,
    D3: Decode,
    D4: Decode,
    D5: Decode,
    D6: Decode,
{
    type Item = (
        D0::Item,
        D1::Item,
        D2::Item,
        D3::Item,
        D4::Item,
        D5::Item,
        D6::Item,
    );

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        let mut offset = 0;
        bytecodec_try_decode!(self.d0, offset, buf, eos);
        bytecodec_try_decode!(self.d1, offset, buf, eos);
        bytecodec_try_decode!(self.d2, offset, buf, eos);
        bytecodec_try_decode!(self.d3, offset, buf, eos);
        bytecodec_try_decode!(self.d4, offset, buf, eos);
        bytecodec_try_decode!(self.d5, offset, buf, eos);

        let (size, item) = track!(self.d6.decode(&buf[offset..], eos))?;
        offset += size;

        let item = item.map(|d6| {
            let d0 = self.d0.take_item().expect("Never fails");
            let d1 = self.d1.take_item().expect("Never fails");
            let d2 = self.d2.take_item().expect("Never fails");
            let d3 = self.d3.take_item().expect("Never fails");
            let d4 = self.d4.take_item().expect("Never fails");
            let d5 = self.d5.take_item().expect("Never fails");
            (d0, d1, d2, d3, d4, d5, d6)
        });
        Ok((offset, item))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.d0
            .requiring_bytes()
            .add_for_decoding(self.d1.requiring_bytes())
            .add_for_decoding(self.d2.requiring_bytes())
            .add_for_decoding(self.d3.requiring_bytes())
            .add_for_decoding(self.d4.requiring_bytes())
            .add_for_decoding(self.d5.requiring_bytes())
            .add_for_decoding(self.d6.requiring_bytes())
    }
}

/// Decoder for 8-elements tuples.
#[derive(Debug, Default)]
pub struct Tuple8Decoder<D0, D1, D2, D3, D4, D5, D6, D7>
where
    D0: Decode,
    D1: Decode,
    D2: Decode,
    D3: Decode,
    D4: Decode,
    D5: Decode,
    D6: Decode,
    D7: Decode,
{
    d0: Buffered<D0>,
    d1: Buffered<D1>,
    d2: Buffered<D2>,
    d3: Buffered<D3>,
    d4: Buffered<D4>,
    d5: Buffered<D5>,
    d6: Buffered<D6>,
    d7: D7,
}
impl<D0, D1, D2, D3, D4, D5, D6, D7> Tuple8Decoder<D0, D1, D2, D3, D4, D5, D6, D7>
where
    D0: Decode,
    D1: Decode,
    D2: Decode,
    D3: Decode,
    D4: Decode,
    D5: Decode,
    D6: Decode,
    D7: Decode,
{
    /// Makes a new `Tuple8Decoder` instance.
    pub fn new(d0: D0, d1: D1, d2: D2, d3: D3, d4: D4, d5: D5, d6: D6, d7: D7) -> Self {
        Tuple8Decoder {
            d0: d0.buffered(),
            d1: d1.buffered(),
            d2: d2.buffered(),
            d3: d3.buffered(),
            d4: d4.buffered(),
            d5: d5.buffered(),
            d6: d6.buffered(),
            d7,
        }
    }

    /// Returns references to the inner decoders.
    pub fn inner_ref(&self) -> (&D0, &D1, &D2, &D3, &D4, &D5, &D6, &D7) {
        (
            self.d0.inner_ref(),
            self.d1.inner_ref(),
            self.d2.inner_ref(),
            self.d3.inner_ref(),
            self.d4.inner_ref(),
            self.d5.inner_ref(),
            self.d6.inner_ref(),
            &self.d7,
        )
    }

    /// Returns mutable references to the inner decoders.
    pub fn inner_mut(
        &mut self,
    ) -> (
        &mut D0,
        &mut D1,
        &mut D2,
        &mut D3,
        &mut D4,
        &mut D5,
        &mut D6,
        &mut D7,
    ) {
        (
            self.d0.inner_mut(),
            self.d1.inner_mut(),
            self.d2.inner_mut(),
            self.d3.inner_mut(),
            self.d4.inner_mut(),
            self.d5.inner_mut(),
            self.d6.inner_mut(),
            &mut self.d7,
        )
    }
}
impl<D0, D1, D2, D3, D4, D5, D6, D7> Decode for Tuple8Decoder<D0, D1, D2, D3, D4, D5, D6, D7>
where
    D0: Decode,
    D1: Decode,
    D2: Decode,
    D3: Decode,
    D4: Decode,
    D5: Decode,
    D6: Decode,
    D7: Decode,
{
    type Item = (
        D0::Item,
        D1::Item,
        D2::Item,
        D3::Item,
        D4::Item,
        D5::Item,
        D6::Item,
        D7::Item,
    );

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<(usize, Option<Self::Item>)> {
        let mut offset = 0;
        bytecodec_try_decode!(self.d0, offset, buf, eos);
        bytecodec_try_decode!(self.d1, offset, buf, eos);
        bytecodec_try_decode!(self.d2, offset, buf, eos);
        bytecodec_try_decode!(self.d3, offset, buf, eos);
        bytecodec_try_decode!(self.d4, offset, buf, eos);
        bytecodec_try_decode!(self.d5, offset, buf, eos);
        bytecodec_try_decode!(self.d6, offset, buf, eos);

        let (size, item) = track!(self.d7.decode(&buf[offset..], eos))?;
        offset += size;

        let item = item.map(|d7| {
            let d0 = self.d0.take_item().expect("Never fails");
            let d1 = self.d1.take_item().expect("Never fails");
            let d2 = self.d2.take_item().expect("Never fails");
            let d3 = self.d3.take_item().expect("Never fails");
            let d4 = self.d4.take_item().expect("Never fails");
            let d5 = self.d5.take_item().expect("Never fails");
            let d6 = self.d6.take_item().expect("Never fails");
            (d0, d1, d2, d3, d4, d5, d6, d7)
        });
        Ok((offset, item))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.d0
            .requiring_bytes()
            .add_for_decoding(self.d1.requiring_bytes())
            .add_for_decoding(self.d2.requiring_bytes())
            .add_for_decoding(self.d3.requiring_bytes())
            .add_for_decoding(self.d4.requiring_bytes())
            .add_for_decoding(self.d5.requiring_bytes())
            .add_for_decoding(self.d6.requiring_bytes())
            .add_for_decoding(self.d7.requiring_bytes())
    }
}

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
        impl<$($t),*> ExactBytesEncode for TupleEncoder<($($t),*,)>
        where
            $($t: ExactBytesEncode),*
        {
            fn exact_requiring_bytes(&self) -> u64 {
                0 $(+ self.inner.$i.exact_requiring_bytes())*
            }
        }
        impl<$($t),*> CalculateBytes for TupleEncoder<($($t),*,)>
        where
            $($t: CalculateBytes),*
        {
            fn calculate_requiring_bytes(&self, t: &Self::Item) -> u64 {
                0 $(+ self.inner.$i.calculate_requiring_bytes(&t.$i))*
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
    use fixnum::{U8Decoder, U8Encoder};
    use io::{IoDecodeExt, IoEncodeExt};
    use {Encode, EncodeExt};

    #[test]
    fn tuple2_decoder_works() {
        let mut decoder = Tuple2Decoder::new(U8Decoder::new(), U8Decoder::new());
        assert_eq!(
            track_try_unwrap!(decoder.decode_exact(b"foo".as_ref())),
            (b'f', b'o')
        );
    }

    #[test]
    fn tuple3_decoder_works() {
        let mut decoder = Tuple3Decoder::new(U8Decoder::new(), U8Decoder::new(), U8Decoder::new());
        assert_eq!(
            track_try_unwrap!(decoder.decode_exact(b"foo".as_ref())),
            (b'f', b'o', b'o')
        );
    }

    #[test]
    fn tuple4_decoder_works() {
        let mut decoder = Tuple4Decoder::new(
            U8Decoder::new(),
            U8Decoder::new(),
            U8Decoder::new(),
            U8Decoder::new(),
        );
        assert_eq!(
            track_try_unwrap!(decoder.decode_exact(b"foobar".as_ref())),
            (b'f', b'o', b'o', b'b')
        );
    }

    #[test]
    fn tuple5_decoder_works() {
        let mut decoder = Tuple5Decoder::new(
            U8Decoder::new(),
            U8Decoder::new(),
            U8Decoder::new(),
            U8Decoder::new(),
            U8Decoder::new(),
        );
        assert_eq!(
            track_try_unwrap!(decoder.decode_exact(b"foobar".as_ref())),
            (b'f', b'o', b'o', b'b', b'a')
        );
    }

    #[test]
    fn tuple6_decoder_works() {
        let mut decoder = Tuple6Decoder::new(
            U8Decoder::new(),
            U8Decoder::new(),
            U8Decoder::new(),
            U8Decoder::new(),
            U8Decoder::new(),
            U8Decoder::new(),
        );
        assert_eq!(
            track_try_unwrap!(decoder.decode_exact(b"foobar".as_ref())),
            (b'f', b'o', b'o', b'b', b'a', b'r')
        );
    }

    #[test]
    fn tuple7_decoder_works() {
        let mut decoder = Tuple7Decoder::new(
            U8Decoder::new(),
            U8Decoder::new(),
            U8Decoder::new(),
            U8Decoder::new(),
            U8Decoder::new(),
            U8Decoder::new(),
            U8Decoder::new(),
        );
        assert_eq!(
            track_try_unwrap!(decoder.decode_exact(b"foobarbaz".as_ref())),
            (b'f', b'o', b'o', b'b', b'a', b'r', b'b')
        );
    }

    #[test]
    fn tuple8_decoder_works() {
        let mut decoder = Tuple8Decoder::new(
            U8Decoder::new(),
            U8Decoder::new(),
            U8Decoder::new(),
            U8Decoder::new(),
            U8Decoder::new(),
            U8Decoder::new(),
            U8Decoder::new(),
            U8Decoder::new(),
        );
        assert_eq!(
            track_try_unwrap!(decoder.decode_exact(b"foobarbaz".as_ref())),
            (b'f', b'o', b'o', b'b', b'a', b'r', b'b', b'a')
        );
    }

    #[test]
    fn tuple2_encoder_works() {
        let mut encoder = TupleEncoder::<(U8Encoder, U8Encoder)>::with_item((0, 1)).unwrap();
        let mut buf = Vec::new();
        encoder.encode_all(&mut buf).unwrap();
        assert_eq!(buf, [0, 1]);
    }

    #[test]
    fn tuple3_encoder_works() {
        let mut encoder =
            TupleEncoder::<(U8Encoder, U8Encoder, U8Encoder)>::with_item((0, 1, 2)).unwrap();
        let mut buf = Vec::new();
        encoder.encode_all(&mut buf).unwrap();
        assert_eq!(buf, [0, 1, 2]);
    }

    #[test]
    fn tuple4_encoder_works() {
        let mut encoder = TupleEncoder::<(U8Encoder, U8Encoder, U8Encoder, U8Encoder)>::with_item(
            (0, 1, 2, 3),
        ).unwrap();
        let mut buf = Vec::new();
        encoder.encode_all(&mut buf).unwrap();
        assert_eq!(buf, [0, 1, 2, 3]);
    }

    #[test]
    fn tuple5_encoder_works() {
        let mut encoder =
            TupleEncoder::<(U8Encoder, U8Encoder, U8Encoder, U8Encoder, U8Encoder)>::default();
        encoder.start_encoding((0, 1, 2, 3, 4)).unwrap();
        let mut buf = Vec::new();
        encoder.encode_all(&mut buf).unwrap();
        assert_eq!(buf, [0, 1, 2, 3, 4]);
    }

    #[test]
    fn tuple6_encoder_works() {
        let mut encoder = TupleEncoder::<(
            U8Encoder,
            U8Encoder,
            U8Encoder,
            U8Encoder,
            U8Encoder,
            U8Encoder,
        )>::default();
        encoder.start_encoding((0, 1, 2, 3, 4, 5)).unwrap();
        let mut buf = Vec::new();
        encoder.encode_all(&mut buf).unwrap();
        assert_eq!(buf, [0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn tuple7_encoder_works() {
        let mut encoder = TupleEncoder::<(
            U8Encoder,
            U8Encoder,
            U8Encoder,
            U8Encoder,
            U8Encoder,
            U8Encoder,
            U8Encoder,
        )>::default();
        encoder.start_encoding((0, 1, 2, 3, 4, 5, 6)).unwrap();
        let mut buf = Vec::new();
        encoder.encode_all(&mut buf).unwrap();
        assert_eq!(buf, [0, 1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn tuple8_encoder_works() {
        let mut encoder = TupleEncoder::<(
            U8Encoder,
            U8Encoder,
            U8Encoder,
            U8Encoder,
            U8Encoder,
            U8Encoder,
            U8Encoder,
            U8Encoder,
        )>::default();
        encoder.start_encoding((0, 1, 2, 3, 4, 5, 6, 7)).unwrap();
        let mut buf = Vec::new();
        encoder.encode_all(&mut buf).unwrap();
        assert_eq!(buf, [0, 1, 2, 3, 4, 5, 6, 7]);
    }
}
