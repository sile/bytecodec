use std;
use trackable::error::{ErrorKind as TrackableErrorKind, ErrorKindExt};
use trackable::error::{Failure, TrackableError};

/// This crate specific `Error` type.
#[derive(Debug, Clone)]
pub struct Error(TrackableError<ErrorKind>);
derive_traits_for_trackable_error_newtype!(Error, ErrorKind);
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
#[allow(missing_docs)]
pub enum ErrorKind {
    InvalidInput,
    UnexpectedEos,
    EncoderFull,
    DecoderTerminated,
    IncompleteItem,
    Other,
}
impl TrackableErrorKind for ErrorKind {}
