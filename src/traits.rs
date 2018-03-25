use Result;

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

// EncodeExt

pub trait MakeEncoder {
    type Encoder: Encode;

    fn make_encoder(&self) -> Self::Encoder;
}
