/// Tries to encode items to the given buffer.
///
/// This macro is expanded to the following expression.
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
/// Note that this macro assumes `$decoder` is an instance of `Buffered<_>`.
///
/// This macro is expanded to the following expression.
///
/// ```ignore
/// if !$decoder.has_item() {
///     $offset += track!($decoder.decode(&$buf[$offset..], $eos))?.0;
///     if let Some(item) = $decoder.get_item() {
///         Some(item)
///     } else {
///         return Ok(($offset, None));
///     }
/// } else {
///     None
/// }
/// ```
#[macro_export]
macro_rules! bytecodec_try_decode {
    ($decoder:expr, $offset:expr, $buf:expr, $eos:expr) => {
        if !$decoder.has_item() {
            $offset += track!($decoder.decode(&$buf[$offset..], $eos))?.0;
            if let Some(item) = $decoder.get_item() {
                Some(item)
            } else {
                return Ok(($offset, None));
            }
        } else {
            None
        }
    };
    ($decoder:expr, $offset:expr, $buf:expr, $eos:expr, $($track_arg:tt)*) => {
        if !$decoder.has_item() {
            $offset += track!($decoder.decode(&$buf[$offset..], $eos), $($track_arg)*)?.0;
            if let Some(item) = $decoder.get_item() {
                Some(item)
            } else {
                return Ok(($offset, None));
            }
        } else {
            None
        }
    };
    ($decoder:expr, $offset:expr, $buf:expr, $eos:expr; $($track_arg:tt)*) => {
        if !$decoder.has_item() {
            $offset += track!($decoder.decode(&$buf[$offset..], $eos); $($track_arg)*)?.0;
            if let Some(item) = $decoder.get_item() {
                Some(item)
            } else {
                return Ok(($offset, None));
            }
        } else {
            None
        }
    };
}
