use ByteCount;

/// `Eos` contains information on the distance to the end of a stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd)]
pub struct Eos(ByteCount);
impl Eos {
    /// Makes a new `Eos` instance.
    pub fn new(is_eos_reached: bool) -> Self {
        if is_eos_reached {
            Eos(ByteCount::Finite(0))
        } else {
            Eos(ByteCount::Unknown)
        }
    }

    /// Makes a new `Eos` instance that
    /// has the given information about the number of remaining bytes in a stream.
    pub fn with_remaining_bytes(n: ByteCount) -> Self {
        Eos(n)
    }

    /// Returns `true` if the target stream has reached to the end, otherwise `false`.
    pub fn is_reached(&self) -> bool {
        self.0 == ByteCount::Finite(0)
    }

    /// Returns the information about the number of bytes remaining in the target stream.
    pub fn remaining_bytes(&self) -> ByteCount {
        self.0
    }

    /// Returns a new `Eos` instance that has moved backward from
    /// the end of the target stream by the specified number of bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecodec::{ByteCount, Eos};
    ///
    /// let eos = Eos::new(true);
    /// assert_eq!(eos.is_reached(), true);
    /// assert_eq!(eos.remaining_bytes(), ByteCount::Finite(0));
    ///
    /// let eos = eos.back(5);
    /// assert_eq!(eos.is_reached(), false);
    /// assert_eq!(eos.remaining_bytes(), ByteCount::Finite(5));
    /// ```
    pub fn back(&self, bytes: u64) -> Self {
        if let ByteCount::Finite(n) = self.0 {
            Eos(ByteCount::Finite(n + bytes))
        } else {
            *self
        }
    }
}
