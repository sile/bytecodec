use {Error, Result};
use combinators::{AndThen, Map, MapErr};

pub trait Decode {
    type Item;

    fn decode(&mut self, buf: &[u8], eos: bool) -> Result<usize>;
    fn pop_item(&mut self) -> Result<Option<Self::Item>>;
    fn decode_size_hint(&self) -> Option<usize> {
        None
    }
}

pub trait MakeDecoder {
    type Decoder: Decode;

    fn make_decoder(&self) -> Self::Decoder;
}

pub trait Encode {
    type Item;

    fn encode(&mut self, buf: &mut [u8]) -> Result<usize>;
    fn push_item(&mut self, item: Self::Item) -> Result<Option<Self::Item>>;
    fn encode_size_hint(&self) -> Option<usize> {
        None
    }
}

pub trait DecodeExt: Decode + Sized {
    fn map<T, F>(self, f: F) -> Map<Self, T, F>
    where
        F: Fn(Self::Item) -> T,
    {
        Map::new(self, f)
    }

    fn and_then<T, F>(self, f: F) -> AndThen<Self, T, F>
    where
        F: Fn(Self::Item) -> T,
        T: Decode,
    {
        AndThen::new(self, f)
    }

    fn map_err<F>(self, f: F) -> MapErr<Self, F>
    where
        F: Fn(Error) -> Error,
    {
        MapErr::new(self, f)
    }
}

pub trait EncodeExt: Encode + Sized {
    fn map_err<F>(self, f: F) -> MapErr<Self, F>
    where
        F: Fn(Error) -> Error,
    {
        MapErr::new(self, f)
    }
}

pub trait MakeEncoder {
    type Encoder: Encode;

    fn make_encoder(&self) -> Self::Encoder;
}
