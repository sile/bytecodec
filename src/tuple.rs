//! Encoders and decoders for tuples.
#![cfg_attr(feature = "cargo-clippy", allow(type_complexity, too_many_arguments))]
use {ByteCount, Decode, DecodeExt, Encode, Eos, ExactBytesEncode, Result};
use combinator::Buffered;

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
        offset += track!(self.d0.decode(buf, eos))?.0;

        let (size, item) = track!(self.d1.decode(&buf[offset..], eos))?;
        offset += size;

        let item = item.map(|d1| (self.d0.take_item().expect("Never fails"), d1));
        Ok((offset, item))
    }

    fn has_terminated(&self) -> bool {
        self.d0.has_terminated() || self.d1.has_terminated()
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
        offset += track!(self.d0.decode(buf, eos))?.0;
        offset += track!(self.d1.decode(&buf[offset..], eos))?.0;

        let (size, item) = track!(self.d2.decode(&buf[offset..], eos))?;
        offset += size;

        let item = item.map(|d2| {
            let d0 = self.d0.take_item().expect("Never fails");
            let d1 = self.d1.take_item().expect("Never fails");
            (d0, d1, d2)
        });
        Ok((offset, item))
    }

    fn has_terminated(&self) -> bool {
        self.d0.has_terminated() || self.d1.has_terminated() || self.d2.has_terminated()
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
        offset += track!(self.d0.decode(buf, eos))?.0;
        offset += track!(self.d1.decode(&buf[offset..], eos))?.0;
        offset += track!(self.d2.decode(&buf[offset..], eos))?.0;

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

    fn has_terminated(&self) -> bool {
        self.d0.has_terminated() || self.d1.has_terminated() || self.d2.has_terminated()
            || self.d3.has_terminated()
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
        offset += track!(self.d0.decode(buf, eos))?.0;
        offset += track!(self.d1.decode(&buf[offset..], eos))?.0;
        offset += track!(self.d2.decode(&buf[offset..], eos))?.0;
        offset += track!(self.d3.decode(&buf[offset..], eos))?.0;

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

    fn has_terminated(&self) -> bool {
        self.d0.has_terminated() || self.d1.has_terminated() || self.d2.has_terminated()
            || self.d3.has_terminated() || self.d4.has_terminated()
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
        offset += track!(self.d0.decode(buf, eos))?.0;
        offset += track!(self.d1.decode(&buf[offset..], eos))?.0;
        offset += track!(self.d2.decode(&buf[offset..], eos))?.0;
        offset += track!(self.d3.decode(&buf[offset..], eos))?.0;
        offset += track!(self.d4.decode(&buf[offset..], eos))?.0;

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

    fn has_terminated(&self) -> bool {
        self.d0.has_terminated() || self.d1.has_terminated() || self.d2.has_terminated()
            || self.d3.has_terminated() || self.d4.has_terminated()
            || self.d5.has_terminated()
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
        offset += track!(self.d0.decode(buf, eos))?.0;
        offset += track!(self.d1.decode(&buf[offset..], eos))?.0;
        offset += track!(self.d2.decode(&buf[offset..], eos))?.0;
        offset += track!(self.d3.decode(&buf[offset..], eos))?.0;
        offset += track!(self.d4.decode(&buf[offset..], eos))?.0;
        offset += track!(self.d5.decode(&buf[offset..], eos))?.0;

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

    fn has_terminated(&self) -> bool {
        self.d0.has_terminated() || self.d1.has_terminated() || self.d2.has_terminated()
            || self.d3.has_terminated() || self.d4.has_terminated()
            || self.d5.has_terminated() || self.d6.has_terminated()
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
        offset += track!(self.d0.decode(buf, eos))?.0;
        offset += track!(self.d1.decode(&buf[offset..], eos))?.0;
        offset += track!(self.d2.decode(&buf[offset..], eos))?.0;
        offset += track!(self.d3.decode(&buf[offset..], eos))?.0;
        offset += track!(self.d4.decode(&buf[offset..], eos))?.0;
        offset += track!(self.d5.decode(&buf[offset..], eos))?.0;
        offset += track!(self.d6.decode(&buf[offset..], eos))?.0;

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

    fn has_terminated(&self) -> bool {
        self.d0.has_terminated() || self.d1.has_terminated() || self.d2.has_terminated()
            || self.d3.has_terminated() || self.d4.has_terminated()
            || self.d5.has_terminated() || self.d6.has_terminated()
            || self.d7.has_terminated()
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

/// Encoder for 2-elements tuples.
#[derive(Debug, Default)]
pub struct Tuple2Encoder<E0, E1> {
    e0: E0,
    e1: E1,
}
impl<E0, E1> Tuple2Encoder<E0, E1>
where
    E0: Encode,
    E1: Encode,
{
    /// Makes a new `Tuple2Encoder` instance.
    pub fn new(e0: E0, e1: E1) -> Self {
        Tuple2Encoder { e0, e1 }
    }

    /// Returns references to the inner decoders.
    pub fn inner_ref(&self) -> (&E0, &E1) {
        (&self.e0, &self.e1)
    }

    /// Returns mutable references to the inner decoders.
    pub fn inner_mut(&mut self) -> (&mut E0, &mut E1) {
        (&mut self.e0, &mut self.e1)
    }
}
impl<E0, E1> Encode for Tuple2Encoder<E0, E1>
where
    E0: Encode,
    E1: Encode,
{
    type Item = (E0::Item, E1::Item);

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        if !self.e0.is_idle() {
            offset += track!(self.e0.encode(buf, eos))?;
            if !self.e0.is_idle() {
                return Ok(offset);
            }
        }
        offset += track!(self.e1.encode(&mut buf[offset..], eos))?;
        Ok(offset)
    }

    fn start_encoding(&mut self, t: Self::Item) -> Result<()> {
        track!(self.e0.start_encoding(t.0))?;
        track!(self.e1.start_encoding(t.1))?;
        Ok(())
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.e0
            .requiring_bytes()
            .add_for_encoding(self.e1.requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.e0.is_idle() && self.e1.is_idle()
    }
}
impl<E0, E1> ExactBytesEncode for Tuple2Encoder<E0, E1>
where
    E0: ExactBytesEncode,
    E1: ExactBytesEncode,
{
    fn exact_requiring_bytes(&self) -> u64 {
        self.e0.exact_requiring_bytes() + self.e1.exact_requiring_bytes()
    }
}

/// Encoder for 3-elements tuples.
#[derive(Debug, Default)]
pub struct Tuple3Encoder<E0, E1, E2> {
    e0: E0,
    e1: E1,
    e2: E2,
}
impl<E0, E1, E2> Tuple3Encoder<E0, E1, E2>
where
    E0: Encode,
    E1: Encode,
    E2: Encode,
{
    /// Makes a new `Tuple3Encoder` instance.
    pub fn new(e0: E0, e1: E1, e2: E2) -> Self {
        Tuple3Encoder { e0, e1, e2 }
    }

    /// Returns references to the inner decoders.
    pub fn inner_ref(&self) -> (&E0, &E1, &E2) {
        (&self.e0, &self.e1, &self.e2)
    }

    /// Returns mutable references to the inner decoders.
    pub fn inner_mut(&mut self) -> (&mut E0, &mut E1, &mut E2) {
        (&mut self.e0, &mut self.e1, &mut self.e2)
    }
}
impl<E0, E1, E2> Encode for Tuple3Encoder<E0, E1, E2>
where
    E0: Encode,
    E1: Encode,
    E2: Encode,
{
    type Item = (E0::Item, E1::Item, E2::Item);

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        if !self.e0.is_idle() {
            offset += track!(self.e0.encode(buf, eos))?;
            if !self.e0.is_idle() {
                return Ok(offset);
            }
        }
        if !self.e1.is_idle() {
            offset += track!(self.e1.encode(&mut buf[offset..], eos))?;
            if !self.e1.is_idle() {
                return Ok(offset);
            }
        }
        offset += track!(self.e2.encode(&mut buf[offset..], eos))?;
        Ok(offset)
    }

    fn start_encoding(&mut self, t: Self::Item) -> Result<()> {
        track!(self.e0.start_encoding(t.0))?;
        track!(self.e1.start_encoding(t.1))?;
        track!(self.e2.start_encoding(t.2))?;
        Ok(())
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.e0
            .requiring_bytes()
            .add_for_encoding(self.e1.requiring_bytes())
            .add_for_encoding(self.e2.requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.e0.is_idle() && self.e1.is_idle() && self.e2.is_idle()
    }
}
impl<E0, E1, E2> ExactBytesEncode for Tuple3Encoder<E0, E1, E2>
where
    E0: ExactBytesEncode,
    E1: ExactBytesEncode,
    E2: ExactBytesEncode,
{
    fn exact_requiring_bytes(&self) -> u64 {
        self.e0.exact_requiring_bytes() + self.e1.exact_requiring_bytes()
            + self.e2.exact_requiring_bytes()
    }
}

/// Encoder for 4-elements tuples.
#[derive(Debug, Default)]
pub struct Tuple4Encoder<E0, E1, E2, E3> {
    e0: E0,
    e1: E1,
    e2: E2,
    e3: E3,
}
impl<E0, E1, E2, E3> Tuple4Encoder<E0, E1, E2, E3>
where
    E0: Encode,
    E1: Encode,
    E2: Encode,
    E3: Encode,
{
    /// Makes a new `Tuple4Encoder` instance.
    pub fn new(e0: E0, e1: E1, e2: E2, e3: E3) -> Self {
        Tuple4Encoder { e0, e1, e2, e3 }
    }

    /// Returns references to the inner decoders.
    pub fn inner_ref(&self) -> (&E0, &E1, &E2, &E3) {
        (&self.e0, &self.e1, &self.e2, &self.e3)
    }

    /// Returns mutable references to the inner decoders.
    pub fn inner_mut(&mut self) -> (&mut E0, &mut E1, &mut E2, &mut E3) {
        (&mut self.e0, &mut self.e1, &mut self.e2, &mut self.e3)
    }
}
impl<E0, E1, E2, E3> Encode for Tuple4Encoder<E0, E1, E2, E3>
where
    E0: Encode,
    E1: Encode,
    E2: Encode,
    E3: Encode,
{
    type Item = (E0::Item, E1::Item, E2::Item, E3::Item);

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        if !self.e0.is_idle() {
            offset += track!(self.e0.encode(buf, eos))?;
            if !self.e0.is_idle() {
                return Ok(offset);
            }
        }
        if !self.e1.is_idle() {
            offset += track!(self.e1.encode(&mut buf[offset..], eos))?;
            if !self.e1.is_idle() {
                return Ok(offset);
            }
        }
        if !self.e2.is_idle() {
            offset += track!(self.e2.encode(&mut buf[offset..], eos))?;
            if !self.e2.is_idle() {
                return Ok(offset);
            }
        }
        offset += track!(self.e3.encode(&mut buf[offset..], eos))?;
        Ok(offset)
    }

    fn start_encoding(&mut self, t: Self::Item) -> Result<()> {
        track!(self.e0.start_encoding(t.0))?;
        track!(self.e1.start_encoding(t.1))?;
        track!(self.e2.start_encoding(t.2))?;
        track!(self.e3.start_encoding(t.3))?;
        Ok(())
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.e0
            .requiring_bytes()
            .add_for_encoding(self.e1.requiring_bytes())
            .add_for_encoding(self.e2.requiring_bytes())
            .add_for_encoding(self.e3.requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.e0.is_idle() && self.e1.is_idle() && self.e2.is_idle() && self.e3.is_idle()
    }
}
impl<E0, E1, E2, E3> ExactBytesEncode for Tuple4Encoder<E0, E1, E2, E3>
where
    E0: ExactBytesEncode,
    E1: ExactBytesEncode,
    E2: ExactBytesEncode,
    E3: ExactBytesEncode,
{
    fn exact_requiring_bytes(&self) -> u64 {
        self.e0.exact_requiring_bytes() + self.e1.exact_requiring_bytes()
            + self.e2.exact_requiring_bytes() + self.e3.exact_requiring_bytes()
    }
}

/// Encoder for 5-elements tuples.
#[derive(Debug, Default)]
pub struct Tuple5Encoder<E0, E1, E2, E3, E4> {
    e0: E0,
    e1: E1,
    e2: E2,
    e3: E3,
    e4: E4,
}
impl<E0, E1, E2, E3, E4> Tuple5Encoder<E0, E1, E2, E3, E4>
where
    E0: Encode,
    E1: Encode,
    E2: Encode,
    E3: Encode,
    E4: Encode,
{
    /// Makes a new `Tuple5Encoder` instance.
    pub fn new(e0: E0, e1: E1, e2: E2, e3: E3, e4: E4) -> Self {
        Tuple5Encoder { e0, e1, e2, e3, e4 }
    }

    /// Returns references to the inner decoders.
    pub fn inner_ref(&self) -> (&E0, &E1, &E2, &E3, &E4) {
        (&self.e0, &self.e1, &self.e2, &self.e3, &self.e4)
    }

    /// Returns mutable references to the inner decoders.
    pub fn inner_mut(&mut self) -> (&mut E0, &mut E1, &mut E2, &mut E3, &mut E4) {
        (
            &mut self.e0,
            &mut self.e1,
            &mut self.e2,
            &mut self.e3,
            &mut self.e4,
        )
    }
}
impl<E0, E1, E2, E3, E4> Encode for Tuple5Encoder<E0, E1, E2, E3, E4>
where
    E0: Encode,
    E1: Encode,
    E2: Encode,
    E3: Encode,
    E4: Encode,
{
    type Item = (E0::Item, E1::Item, E2::Item, E3::Item, E4::Item);

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        if !self.e0.is_idle() {
            offset += track!(self.e0.encode(buf, eos))?;
            if !self.e0.is_idle() {
                return Ok(offset);
            }
        }
        if !self.e1.is_idle() {
            offset += track!(self.e1.encode(&mut buf[offset..], eos))?;
            if !self.e1.is_idle() {
                return Ok(offset);
            }
        }
        if !self.e2.is_idle() {
            offset += track!(self.e2.encode(&mut buf[offset..], eos))?;
            if !self.e2.is_idle() {
                return Ok(offset);
            }
        }
        if !self.e3.is_idle() {
            offset += track!(self.e3.encode(&mut buf[offset..], eos))?;
            if !self.e3.is_idle() {
                return Ok(offset);
            }
        }
        offset += track!(self.e4.encode(&mut buf[offset..], eos))?;
        Ok(offset)
    }

    fn start_encoding(&mut self, t: Self::Item) -> Result<()> {
        track!(self.e0.start_encoding(t.0))?;
        track!(self.e1.start_encoding(t.1))?;
        track!(self.e2.start_encoding(t.2))?;
        track!(self.e3.start_encoding(t.3))?;
        track!(self.e4.start_encoding(t.4))?;
        Ok(())
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.e0
            .requiring_bytes()
            .add_for_encoding(self.e1.requiring_bytes())
            .add_for_encoding(self.e2.requiring_bytes())
            .add_for_encoding(self.e3.requiring_bytes())
            .add_for_encoding(self.e4.requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.e0.is_idle() && self.e1.is_idle() && self.e2.is_idle() && self.e3.is_idle()
            && self.e4.is_idle()
    }
}
impl<E0, E1, E2, E3, E4> ExactBytesEncode for Tuple5Encoder<E0, E1, E2, E3, E4>
where
    E0: ExactBytesEncode,
    E1: ExactBytesEncode,
    E2: ExactBytesEncode,
    E3: ExactBytesEncode,
    E4: ExactBytesEncode,
{
    fn exact_requiring_bytes(&self) -> u64 {
        self.e0.exact_requiring_bytes() + self.e1.exact_requiring_bytes()
            + self.e2.exact_requiring_bytes() + self.e3.exact_requiring_bytes()
            + self.e4.exact_requiring_bytes()
    }
}

/// Encoder for 6-elements tuples.
#[derive(Debug, Default)]
pub struct Tuple6Encoder<E0, E1, E2, E3, E4, E5> {
    e0: E0,
    e1: E1,
    e2: E2,
    e3: E3,
    e4: E4,
    e5: E5,
}
impl<E0, E1, E2, E3, E4, E5> Tuple6Encoder<E0, E1, E2, E3, E4, E5>
where
    E0: Encode,
    E1: Encode,
    E2: Encode,
    E3: Encode,
    E4: Encode,
    E5: Encode,
{
    /// Makes a new `Tuple6Encoder` instance.
    pub fn new(e0: E0, e1: E1, e2: E2, e3: E3, e4: E4, e5: E5) -> Self {
        Tuple6Encoder {
            e0,
            e1,
            e2,
            e3,
            e4,
            e5,
        }
    }

    /// Returns references to the inner decoders.
    pub fn inner_ref(&self) -> (&E0, &E1, &E2, &E3, &E4, &E5) {
        (&self.e0, &self.e1, &self.e2, &self.e3, &self.e4, &self.e5)
    }

    /// Returns mutable references to the inner decoders.
    pub fn inner_mut(&mut self) -> (&mut E0, &mut E1, &mut E2, &mut E3, &mut E4, &mut E5) {
        (
            &mut self.e0,
            &mut self.e1,
            &mut self.e2,
            &mut self.e3,
            &mut self.e4,
            &mut self.e5,
        )
    }
}
impl<E0, E1, E2, E3, E4, E5> Encode for Tuple6Encoder<E0, E1, E2, E3, E4, E5>
where
    E0: Encode,
    E1: Encode,
    E2: Encode,
    E3: Encode,
    E4: Encode,
    E5: Encode,
{
    type Item = (E0::Item, E1::Item, E2::Item, E3::Item, E4::Item, E5::Item);

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        if !self.e0.is_idle() {
            offset += track!(self.e0.encode(buf, eos))?;
            if !self.e0.is_idle() {
                return Ok(offset);
            }
        }
        if !self.e1.is_idle() {
            offset += track!(self.e1.encode(&mut buf[offset..], eos))?;
            if !self.e1.is_idle() {
                return Ok(offset);
            }
        }
        if !self.e2.is_idle() {
            offset += track!(self.e2.encode(&mut buf[offset..], eos))?;
            if !self.e2.is_idle() {
                return Ok(offset);
            }
        }
        if !self.e3.is_idle() {
            offset += track!(self.e3.encode(&mut buf[offset..], eos))?;
            if !self.e3.is_idle() {
                return Ok(offset);
            }
        }
        if !self.e4.is_idle() {
            offset += track!(self.e4.encode(&mut buf[offset..], eos))?;
            if !self.e4.is_idle() {
                return Ok(offset);
            }
        }
        offset += track!(self.e5.encode(&mut buf[offset..], eos))?;
        Ok(offset)
    }

    fn start_encoding(&mut self, t: Self::Item) -> Result<()> {
        track!(self.e0.start_encoding(t.0))?;
        track!(self.e1.start_encoding(t.1))?;
        track!(self.e2.start_encoding(t.2))?;
        track!(self.e3.start_encoding(t.3))?;
        track!(self.e4.start_encoding(t.4))?;
        track!(self.e5.start_encoding(t.5))?;
        Ok(())
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.e0
            .requiring_bytes()
            .add_for_encoding(self.e1.requiring_bytes())
            .add_for_encoding(self.e2.requiring_bytes())
            .add_for_encoding(self.e3.requiring_bytes())
            .add_for_encoding(self.e4.requiring_bytes())
            .add_for_encoding(self.e5.requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.e0.is_idle() && self.e1.is_idle() && self.e2.is_idle() && self.e3.is_idle()
            && self.e4.is_idle() && self.e5.is_idle()
    }
}
impl<E0, E1, E2, E3, E4, E5> ExactBytesEncode for Tuple6Encoder<E0, E1, E2, E3, E4, E5>
where
    E0: ExactBytesEncode,
    E1: ExactBytesEncode,
    E2: ExactBytesEncode,
    E3: ExactBytesEncode,
    E4: ExactBytesEncode,
    E5: ExactBytesEncode,
{
    fn exact_requiring_bytes(&self) -> u64 {
        self.e0.exact_requiring_bytes() + self.e1.exact_requiring_bytes()
            + self.e2.exact_requiring_bytes() + self.e3.exact_requiring_bytes()
            + self.e4.exact_requiring_bytes() + self.e5.exact_requiring_bytes()
    }
}

/// Encoder for 7-elements tuples.
#[derive(Debug, Default)]
pub struct Tuple7Encoder<E0, E1, E2, E3, E4, E5, E6> {
    e0: E0,
    e1: E1,
    e2: E2,
    e3: E3,
    e4: E4,
    e5: E5,
    e6: E6,
}
impl<E0, E1, E2, E3, E4, E5, E6> Tuple7Encoder<E0, E1, E2, E3, E4, E5, E6>
where
    E0: Encode,
    E1: Encode,
    E2: Encode,
    E3: Encode,
    E4: Encode,
    E5: Encode,
    E6: Encode,
{
    /// Makes a new `Tuple7Encoder` instance.
    pub fn new(e0: E0, e1: E1, e2: E2, e3: E3, e4: E4, e5: E5, e6: E6) -> Self {
        Tuple7Encoder {
            e0,
            e1,
            e2,
            e3,
            e4,
            e5,
            e6,
        }
    }

    /// Returns references to the inner decoders.
    pub fn inner_ref(&self) -> (&E0, &E1, &E2, &E3, &E4, &E5, &E6) {
        (
            &self.e0,
            &self.e1,
            &self.e2,
            &self.e3,
            &self.e4,
            &self.e5,
            &self.e6,
        )
    }

    /// Returns mutable references to the inner decoders.
    pub fn inner_mut(
        &mut self,
    ) -> (
        &mut E0,
        &mut E1,
        &mut E2,
        &mut E3,
        &mut E4,
        &mut E5,
        &mut E6,
    ) {
        (
            &mut self.e0,
            &mut self.e1,
            &mut self.e2,
            &mut self.e3,
            &mut self.e4,
            &mut self.e5,
            &mut self.e6,
        )
    }
}
impl<E0, E1, E2, E3, E4, E5, E6> Encode for Tuple7Encoder<E0, E1, E2, E3, E4, E5, E6>
where
    E0: Encode,
    E1: Encode,
    E2: Encode,
    E3: Encode,
    E4: Encode,
    E5: Encode,
    E6: Encode,
{
    type Item = (
        E0::Item,
        E1::Item,
        E2::Item,
        E3::Item,
        E4::Item,
        E5::Item,
        E6::Item,
    );

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        if !self.e0.is_idle() {
            offset += track!(self.e0.encode(buf, eos))?;
            if !self.e0.is_idle() {
                return Ok(offset);
            }
        }
        if !self.e1.is_idle() {
            offset += track!(self.e1.encode(&mut buf[offset..], eos))?;
            if !self.e1.is_idle() {
                return Ok(offset);
            }
        }
        if !self.e2.is_idle() {
            offset += track!(self.e2.encode(&mut buf[offset..], eos))?;
            if !self.e2.is_idle() {
                return Ok(offset);
            }
        }
        if !self.e3.is_idle() {
            offset += track!(self.e3.encode(&mut buf[offset..], eos))?;
            if !self.e3.is_idle() {
                return Ok(offset);
            }
        }
        if !self.e4.is_idle() {
            offset += track!(self.e4.encode(&mut buf[offset..], eos))?;
            if !self.e4.is_idle() {
                return Ok(offset);
            }
        }
        if !self.e5.is_idle() {
            offset += track!(self.e5.encode(&mut buf[offset..], eos))?;
            if !self.e5.is_idle() {
                return Ok(offset);
            }
        }
        offset += track!(self.e6.encode(&mut buf[offset..], eos))?;
        Ok(offset)
    }

    fn start_encoding(&mut self, t: Self::Item) -> Result<()> {
        track!(self.e0.start_encoding(t.0))?;
        track!(self.e1.start_encoding(t.1))?;
        track!(self.e2.start_encoding(t.2))?;
        track!(self.e3.start_encoding(t.3))?;
        track!(self.e4.start_encoding(t.4))?;
        track!(self.e5.start_encoding(t.5))?;
        track!(self.e6.start_encoding(t.6))?;
        Ok(())
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.e0
            .requiring_bytes()
            .add_for_encoding(self.e1.requiring_bytes())
            .add_for_encoding(self.e2.requiring_bytes())
            .add_for_encoding(self.e3.requiring_bytes())
            .add_for_encoding(self.e4.requiring_bytes())
            .add_for_encoding(self.e5.requiring_bytes())
            .add_for_encoding(self.e6.requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.e0.is_idle() && self.e1.is_idle() && self.e2.is_idle() && self.e3.is_idle()
            && self.e4.is_idle() && self.e5.is_idle() && self.e6.is_idle()
    }
}
impl<E0, E1, E2, E3, E4, E5, E6> ExactBytesEncode for Tuple7Encoder<E0, E1, E2, E3, E4, E5, E6>
where
    E0: ExactBytesEncode,
    E1: ExactBytesEncode,
    E2: ExactBytesEncode,
    E3: ExactBytesEncode,
    E4: ExactBytesEncode,
    E5: ExactBytesEncode,
    E6: ExactBytesEncode,
{
    fn exact_requiring_bytes(&self) -> u64 {
        self.e0.exact_requiring_bytes() + self.e1.exact_requiring_bytes()
            + self.e2.exact_requiring_bytes() + self.e3.exact_requiring_bytes()
            + self.e4.exact_requiring_bytes() + self.e5.exact_requiring_bytes()
            + self.e6.exact_requiring_bytes()
    }
}

/// Encoder for 8-elements tuples.
#[derive(Debug, Default)]
pub struct Tuple8Encoder<E0, E1, E2, E3, E4, E5, E6, E7> {
    e0: E0,
    e1: E1,
    e2: E2,
    e3: E3,
    e4: E4,
    e5: E5,
    e6: E6,
    e7: E7,
}
impl<E0, E1, E2, E3, E4, E5, E6, E7> Tuple8Encoder<E0, E1, E2, E3, E4, E5, E6, E7>
where
    E0: Encode,
    E1: Encode,
    E2: Encode,
    E3: Encode,
    E4: Encode,
    E5: Encode,
    E6: Encode,
    E7: Encode,
{
    /// Makes a new `Tuple8Encoder` instance.
    pub fn new(e0: E0, e1: E1, e2: E2, e3: E3, e4: E4, e5: E5, e6: E6, e7: E7) -> Self {
        Tuple8Encoder {
            e0,
            e1,
            e2,
            e3,
            e4,
            e5,
            e6,
            e7,
        }
    }

    /// Returns references to the inner decoders.
    pub fn inner_ref(&self) -> (&E0, &E1, &E2, &E3, &E4, &E5, &E6, &E7) {
        (
            &self.e0,
            &self.e1,
            &self.e2,
            &self.e3,
            &self.e4,
            &self.e5,
            &self.e6,
            &self.e7,
        )
    }

    /// Returns mutable references to the inner decoders.
    pub fn inner_mut(
        &mut self,
    ) -> (
        &mut E0,
        &mut E1,
        &mut E2,
        &mut E3,
        &mut E4,
        &mut E5,
        &mut E6,
        &mut E7,
    ) {
        (
            &mut self.e0,
            &mut self.e1,
            &mut self.e2,
            &mut self.e3,
            &mut self.e4,
            &mut self.e5,
            &mut self.e6,
            &mut self.e7,
        )
    }
}
impl<E0, E1, E2, E3, E4, E5, E6, E7> Encode for Tuple8Encoder<E0, E1, E2, E3, E4, E5, E6, E7>
where
    E0: Encode,
    E1: Encode,
    E2: Encode,
    E3: Encode,
    E4: Encode,
    E5: Encode,
    E6: Encode,
    E7: Encode,
{
    type Item = (
        E0::Item,
        E1::Item,
        E2::Item,
        E3::Item,
        E4::Item,
        E5::Item,
        E6::Item,
        E7::Item,
    );

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        if !self.e0.is_idle() {
            offset += track!(self.e0.encode(buf, eos))?;
            if !self.e0.is_idle() {
                return Ok(offset);
            }
        }
        if !self.e1.is_idle() {
            offset += track!(self.e1.encode(&mut buf[offset..], eos))?;
            if !self.e1.is_idle() {
                return Ok(offset);
            }
        }
        if !self.e2.is_idle() {
            offset += track!(self.e2.encode(&mut buf[offset..], eos))?;
            if !self.e2.is_idle() {
                return Ok(offset);
            }
        }
        if !self.e3.is_idle() {
            offset += track!(self.e3.encode(&mut buf[offset..], eos))?;
            if !self.e3.is_idle() {
                return Ok(offset);
            }
        }
        if !self.e4.is_idle() {
            offset += track!(self.e4.encode(&mut buf[offset..], eos))?;
            if !self.e4.is_idle() {
                return Ok(offset);
            }
        }
        if !self.e5.is_idle() {
            offset += track!(self.e5.encode(&mut buf[offset..], eos))?;
            if !self.e5.is_idle() {
                return Ok(offset);
            }
        }
        if !self.e6.is_idle() {
            offset += track!(self.e6.encode(&mut buf[offset..], eos))?;
            if !self.e6.is_idle() {
                return Ok(offset);
            }
        }
        offset += track!(self.e7.encode(&mut buf[offset..], eos))?;
        Ok(offset)
    }

    fn start_encoding(&mut self, t: Self::Item) -> Result<()> {
        track!(self.e0.start_encoding(t.0))?;
        track!(self.e1.start_encoding(t.1))?;
        track!(self.e2.start_encoding(t.2))?;
        track!(self.e3.start_encoding(t.3))?;
        track!(self.e4.start_encoding(t.4))?;
        track!(self.e5.start_encoding(t.5))?;
        track!(self.e6.start_encoding(t.6))?;
        track!(self.e7.start_encoding(t.7))?;
        Ok(())
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.e0
            .requiring_bytes()
            .add_for_encoding(self.e1.requiring_bytes())
            .add_for_encoding(self.e2.requiring_bytes())
            .add_for_encoding(self.e3.requiring_bytes())
            .add_for_encoding(self.e4.requiring_bytes())
            .add_for_encoding(self.e5.requiring_bytes())
            .add_for_encoding(self.e6.requiring_bytes())
            .add_for_encoding(self.e7.requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.e0.is_idle() && self.e1.is_idle() && self.e2.is_idle() && self.e3.is_idle()
            && self.e4.is_idle() && self.e5.is_idle() && self.e6.is_idle()
            && self.e7.is_idle()
    }
}
impl<E0, E1, E2, E3, E4, E5, E6, E7> ExactBytesEncode
    for Tuple8Encoder<E0, E1, E2, E3, E4, E5, E6, E7>
where
    E0: ExactBytesEncode,
    E1: ExactBytesEncode,
    E2: ExactBytesEncode,
    E3: ExactBytesEncode,
    E4: ExactBytesEncode,
    E5: ExactBytesEncode,
    E6: ExactBytesEncode,
    E7: ExactBytesEncode,
{
    fn exact_requiring_bytes(&self) -> u64 {
        self.e0.exact_requiring_bytes() + self.e1.exact_requiring_bytes()
            + self.e2.exact_requiring_bytes() + self.e3.exact_requiring_bytes()
            + self.e4.exact_requiring_bytes() + self.e5.exact_requiring_bytes()
            + self.e6.exact_requiring_bytes() + self.e7.exact_requiring_bytes()
    }
}

#[cfg(test)]
mod test {
    use {Encode, EncodeExt};
    use fixnum::{U8Decoder, U8Encoder};
    use io::{IoDecodeExt, IoEncodeExt};
    use super::*;

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
        let mut encoder = Tuple2Encoder::<U8Encoder, U8Encoder>::with_item((0, 1)).unwrap();
        let mut buf = Vec::new();
        encoder.encode_all(&mut buf).unwrap();
        assert_eq!(buf, [0, 1]);
    }

    #[test]
    fn tuple3_encoder_works() {
        let mut encoder =
            Tuple3Encoder::<U8Encoder, U8Encoder, U8Encoder>::with_item((0, 1, 2)).unwrap();
        let mut buf = Vec::new();
        encoder.encode_all(&mut buf).unwrap();
        assert_eq!(buf, [0, 1, 2]);
    }

    #[test]
    fn tuple4_encoder_works() {
        let mut encoder = Tuple4Encoder::<U8Encoder, U8Encoder, U8Encoder, U8Encoder>::with_item(
            (0, 1, 2, 3),
        ).unwrap();
        let mut buf = Vec::new();
        encoder.encode_all(&mut buf).unwrap();
        assert_eq!(buf, [0, 1, 2, 3]);
    }

    #[test]
    fn tuple5_encoder_works() {
        let mut encoder =
            Tuple5Encoder::<U8Encoder, U8Encoder, U8Encoder, U8Encoder, U8Encoder>::default();
        encoder.start_encoding((0, 1, 2, 3, 4)).unwrap();
        let mut buf = Vec::new();
        encoder.encode_all(&mut buf).unwrap();
        assert_eq!(buf, [0, 1, 2, 3, 4]);
    }

    #[test]
    fn tuple6_encoder_works() {
        let mut encoder = Tuple6Encoder::<
            U8Encoder,
            U8Encoder,
            U8Encoder,
            U8Encoder,
            U8Encoder,
            U8Encoder,
        >::default();
        encoder.start_encoding((0, 1, 2, 3, 4, 5)).unwrap();
        let mut buf = Vec::new();
        encoder.encode_all(&mut buf).unwrap();
        assert_eq!(buf, [0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn tuple7_encoder_works() {
        let mut encoder = Tuple7Encoder::<
            U8Encoder,
            U8Encoder,
            U8Encoder,
            U8Encoder,
            U8Encoder,
            U8Encoder,
            U8Encoder,
        >::default();
        encoder.start_encoding((0, 1, 2, 3, 4, 5, 6)).unwrap();
        let mut buf = Vec::new();
        encoder.encode_all(&mut buf).unwrap();
        assert_eq!(buf, [0, 1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn tuple8_encoder_works() {
        let mut encoder = Tuple8Encoder::<
            U8Encoder,
            U8Encoder,
            U8Encoder,
            U8Encoder,
            U8Encoder,
            U8Encoder,
            U8Encoder,
            U8Encoder,
        >::default();
        encoder.start_encoding((0, 1, 2, 3, 4, 5, 6, 7)).unwrap();
        let mut buf = Vec::new();
        encoder.encode_all(&mut buf).unwrap();
        assert_eq!(buf, [0, 1, 2, 3, 4, 5, 6, 7]);
    }
}
