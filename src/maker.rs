use std::marker::PhantomData;

use {Decode, Encode};

pub trait MakeDecoder {
    type Decoder: Decode;

    fn make_decoder(&self) -> Self::Decoder;
}

pub trait MakeEncoder {
    type Encoder: Encode;

    fn make_encoder(&self) -> Self::Encoder;
}

#[derive(Debug)]
pub struct DefaultDecoderMaker<D>(PhantomData<D>);
impl<D> DefaultDecoderMaker<D> {
    pub fn new() -> Self {
        DefaultDecoderMaker(PhantomData)
    }
}
impl<D: Decode + Default> MakeDecoder for DefaultDecoderMaker<D> {
    type Decoder = D;

    fn make_decoder(&self) -> Self::Decoder {
        Default::default()
    }
}
impl<D> Default for DefaultDecoderMaker<D> {
    fn default() -> Self {
        Self::new()
    }
}
unsafe impl<D> Send for DefaultDecoderMaker<D> {}
unsafe impl<D> Sync for DefaultDecoderMaker<D> {}
impl<D> Clone for DefaultDecoderMaker<D> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct DefaultEncoderMaker<E>(PhantomData<E>);
impl<E> DefaultEncoderMaker<E> {
    pub fn new() -> Self {
        DefaultEncoderMaker(PhantomData)
    }
}
impl<E: Encode + Default> MakeEncoder for DefaultEncoderMaker<E> {
    type Encoder = E;

    fn make_encoder(&self) -> Self::Encoder {
        Default::default()
    }
}
impl<E> Default for DefaultEncoderMaker<E> {
    fn default() -> Self {
        Self::new()
    }
}
unsafe impl<E> Send for DefaultEncoderMaker<E> {}
unsafe impl<E> Sync for DefaultEncoderMaker<E> {}
impl<E> Clone for DefaultEncoderMaker<E> {
    fn clone(&self) -> Self {
        Self::new()
    }
}
