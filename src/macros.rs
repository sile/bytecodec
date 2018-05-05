/// Tries to encode item to the given buffer.
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
    }
}
