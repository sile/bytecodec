use std::cmp;

/// Number of bytes of interest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum ByteCount {
    Finite(u64),
    Infinite,
    Unknown,
}
impl ByteCount {
    /// Returns `true` if this is `ByteCount::Finite(_)`, otherwise `false`.
    pub fn is_finite(&self) -> bool {
        if let ByteCount::Finite(_) = *self {
            true
        } else {
            false
        }
    }

    /// Returns `true` if this is `ByteCount::Infinite`, otherwise `false`.
    pub fn is_infinite(&self) -> bool {
        *self == ByteCount::Infinite
    }

    /// Returns `true` if this is `ByteCount::Unknown`, otherwise `false`.
    pub fn is_unknow(&self) -> bool {
        *self == ByteCount::Unknown
    }

    /// Tries to convert this `ByteCount` to an `u64` value.
    ///
    /// If it is not a `ByteCount::Finite(_)`,`None` will be returned.
    pub fn to_u64(&self) -> Option<u64> {
        if let ByteCount::Finite(n) = *self {
            Some(n)
        } else {
            None
        }
    }

    /// Adds two `ByteCount` instances for decoding (i.e., `Decode::requiring_bytes` method).
    ///
    /// # Rule
    ///
    /// ```text
    /// Finite(a) + Finite(b) = Finite(a + b)
    /// Infinite  + _         = Infinite
    /// _         + Infinite  = Infinite
    /// Unknown   + Unknown   = Unknown
    /// Finite(0) + Unknown   = Unknown
    /// Unknown   + Finite(0) = Unknown
    /// Finite(a) + Unknown   = Finite(a)
    /// Unknown   + Finite(b) = Finite(b)
    /// ```
    pub fn add_for_decoding(self, other: Self) -> Self {
        match (self, other) {
            (ByteCount::Finite(a), ByteCount::Finite(b)) => ByteCount::Finite(a + b),
            (ByteCount::Infinite, _) => ByteCount::Infinite,
            (_, ByteCount::Infinite) => ByteCount::Infinite,
            (ByteCount::Unknown, ByteCount::Unknown) => ByteCount::Unknown,
            (ByteCount::Finite(0), ByteCount::Unknown) => ByteCount::Unknown,
            (ByteCount::Unknown, ByteCount::Finite(0)) => ByteCount::Unknown,
            (ByteCount::Finite(a), ByteCount::Unknown) => ByteCount::Finite(a),
            (ByteCount::Unknown, ByteCount::Finite(b)) => ByteCount::Finite(b),
        }
    }

    /// Adds two `ByteCount` instances for encoding (i.e., `Encode::requiring_bytes` method).
    ///
    /// # Rule
    ///
    /// ```text
    /// Finite(a) + Finite(b) = Finite(a + b)
    /// Infinite  + _         = Infinite
    /// _         + Infinite  = Infinite
    /// Unknown   + _         = Unknown
    /// _         + Unknown   = Unknown
    /// ```
    pub fn add_for_encoding(self, other: Self) -> Self {
        match (self, other) {
            (ByteCount::Finite(a), ByteCount::Finite(b)) => ByteCount::Finite(a + b),
            (ByteCount::Infinite, _) => ByteCount::Infinite,
            (_, ByteCount::Infinite) => ByteCount::Infinite,
            (_, ByteCount::Unknown) => ByteCount::Unknown,
            (ByteCount::Unknown, _) => ByteCount::Unknown,
        }
    }
}
impl PartialOrd for ByteCount {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match (*self, *other) {
            (ByteCount::Finite(l), ByteCount::Finite(r)) => Some(l.cmp(&r)),
            (ByteCount::Unknown, _) | (_, ByteCount::Unknown) => None,
            (ByteCount::Infinite, ByteCount::Infinite) => Some(cmp::Ordering::Equal),
            (ByteCount::Infinite, _) => Some(cmp::Ordering::Greater),
            (_, ByteCount::Infinite) => Some(cmp::Ordering::Less),
        }
    }
}
impl Default for ByteCount {
    /// Returns `ByteCount::Unknown` as the default value.
    fn default() -> Self {
        ByteCount::Unknown
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(ByteCount::default(), ByteCount::Unknown);

        assert!(ByteCount::Finite(0).is_finite());
        assert!(ByteCount::Infinite.is_infinite());
        assert!(ByteCount::Unknown.is_unknow());

        assert_eq!(ByteCount::Finite(3).to_u64(), Some(3));
        assert_eq!(ByteCount::Infinite.to_u64(), None);
        assert_eq!(ByteCount::Unknown.to_u64(), None);

        assert!(ByteCount::Finite(1) < ByteCount::Finite(2));
        assert!(ByteCount::Finite(9) < ByteCount::Infinite);
        assert!(!(ByteCount::Infinite < ByteCount::Unknown));
        assert!(!(ByteCount::Unknown < ByteCount::Infinite));
        assert!(!(ByteCount::Unknown < ByteCount::Unknown));
        assert!(!(ByteCount::Unknown < ByteCount::Unknown));
    }
}
