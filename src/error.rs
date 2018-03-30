use std;
use trackable::error::{Failure, TrackableError};
use trackable::error::{ErrorKind as TrackableErrorKind, ErrorKindExt};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    InvalidInput,
    Full,
    UnexpectedEos,
    DecoderTerminated,
    Other,
}
impl TrackableErrorKind for ErrorKind {}
