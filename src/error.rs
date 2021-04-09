use trackable::error::{ErrorKind as TrackableErrorKind, ErrorKindExt};
use trackable::error::{Failure, TrackableError};

/// This crate specific `Error` type.
#[derive(Debug, Clone, TrackableError)]
pub struct Error(TrackableError<ErrorKind>);
impl From<Failure> for Error {
    fn from(f: Failure) -> Self {
        ErrorKind::Other.takes_over(f).into()
    }
}
impl From<std::io::Error> for Error {
    fn from(f: std::io::Error) -> Self {
        let kind = if f.kind() == std::io::ErrorKind::UnexpectedEof {
            ErrorKind::UnexpectedEos
        } else {
            ErrorKind::Other
        };
        kind.cause(f).into()
    }
}

/// Possible error kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// Input is invalid.
    ///
    /// Usually it indicates there is a problem outside of the encoder/decoder.
    InvalidInput,

    /// Inconsistent state of the encoder/decoder.
    ///
    /// Usually it indicates there is a bug in the encoder/decoder.
    InconsistentState,

    /// Unexpected EOS.
    ///
    /// A target stream has reached EOS despite there are some items to be encoded/decoded.
    UnexpectedEos,

    /// Encoder is full.
    ///
    /// The encoder cannot accept more items because it has some items to be encoded currently.
    EncoderFull,

    /// Decoder has terminated.
    ///
    /// The decoder cannot decode any more items.
    DecoderTerminated,

    /// A decoding process terminated incompletely.
    IncompleteDecoding,

    /// Other errors.
    Other,
}
impl TrackableErrorKind for ErrorKind {}
