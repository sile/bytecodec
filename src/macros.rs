/// Tries to encode items to the given buffer.
///
/// Conceptually, this macro is expanded to the following expression.
///
/// ```ignore
/// if !$encoder.is_idle() {
///     $offset += track!($encoder.encode(&mut $buf[$offset..], $eos))?;
///     if !$encoder.is_idle() {
///         return Ok($offset);
///     }
/// }
/// ```
#[macro_export]
macro_rules! bytecodec_try_encode {
    ($encoder:expr, $offset:expr, $buf:expr, $eos:expr) => {
        if !$encoder.is_idle() {
            $offset += track!($encoder.encode(&mut $buf[$offset..], $eos))?;
            if !$encoder.is_idle() {
                return Ok($offset);
            }
        }
    };
    ($encoder:expr, $offset:expr, $buf:expr, $eos:expr, $($track_arg:tt)*) => {
        if !$encoder.is_idle() {
            $offset += track!($encoder.encode(&mut $buf[$offset..], $eos), $($track_arg)*)?;
            if !$encoder.is_idle() {
                return Ok($offset);
            }
        }
    };
    ($encoder:expr, $offset:expr, $buf:expr, $eos:expr; $($track_arg:tt)*) => {
        if !$encoder.is_idle() {
            $offset += track!($encoder.encode(&mut $buf[$offset..], $eos); $($track_arg)*)?;
            if !$encoder.is_idle() {
                return Ok($offset);
            }
        }
    };
}

/// Tries to decode an item from the given buffer.
///
/// Conceptually, this macro is expanded to the following expression.
///
/// ```ignore
/// if !$decoder.is_idle() {
///     $offset += track!($decoder.decode(&$buf[$offset..], $eos))?;
///     if !$decoder.is_idle() {
///         return Ok($offset);
///     }
/// }
/// ```
#[macro_export]
macro_rules! bytecodec_try_decode {
    ($decoder:expr, $offset:expr, $buf:expr, $eos:expr) => {
        if !$decoder.is_idle() {
            $offset += track!($decoder.decode(&$buf[$offset..], $eos))?;
            if !$decoder.is_idle() {
                return Ok($offset);
            }
        }
    };
    ($decoder:expr, $offset:expr, $buf:expr, $eos:expr, $($track_arg:tt)*) => {
        if !$decoder.is_idle() {
            $offset += track!($decoder.decode(&$buf[$offset..], $eos), $($track_arg)*)?;
            if !$decoder.is_idle() {
                return Ok($offset);
            }
        }
    };
    ($decoder:expr, $offset:expr, $buf:expr, $eos:expr; $($track_arg:tt)*) => {
        if !$decoder.is_idle() {
            $offset += track!($decoder.decode(&$buf[$offset..], $eos); $($track_arg)*)?;
            if !$decoder.is_idle() {
                return Ok($offset);
            }
        }
    };
}
