use std::cmp;
use std::ops::Add;

/// `Eos` contains information abount the position of the end of a stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

    /// TODO: doc
    pub fn with_remaining_bytes(n: ByteCount) -> Self {
        Eos(n)
    }

    /// Returns `true` if the target stream has reached to the end, otherwise `false`.
    pub fn is_reached(&self) -> bool {
        self.0 == ByteCount::Finite(0)
    }

    /// Returns the number of bytes remaining in the target stream.
    ///
    /// If it is unknown, `None` will be returned.
    /// TODO: update doc
    pub fn remaining_bytes(&self) -> ByteCount {
        self.0
    }

    /// Returns a new `Eos` instance that has moved backward from
    /// the end of the target stream by the specified number of bytes.
    pub fn back(&self, bytes: u64) -> Self {
        if let ByteCount::Finite(n) = self.0 {
            Eos(ByteCount::Finite(n + bytes))
        } else {
            *self
        }
    }
}

// TODO: move and doc
// TODO: PartialOrd
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum ByteCount {
    Finite(u64),
    Infinite,
    Unknown,
}
#[allow(missing_docs)]
impl ByteCount {
    pub fn is_finite(&self) -> bool {
        if let ByteCount::Finite(_) = *self {
            true
        } else {
            false
        }
    }

    pub fn is_infinite(&self) -> bool {
        *self == ByteCount::Infinite
    }

    pub fn is_unknow(&self) -> bool {
        *self == ByteCount::Unknown
    }

    /// TODO: doc
    pub fn min(&self, n: u64) -> Self {
        match *self {
            ByteCount::Finite(m) => ByteCount::Finite(cmp::min(n, m)),
            ByteCount::Infinite => ByteCount::Finite(n),
            ByteCount::Unknown => ByteCount::Unknown,
        }
    }

    /// TODO: doc
    pub fn to_finite(&self) -> Option<u64> {
        if let ByteCount::Finite(n) = *self {
            Some(n)
        } else {
            None
        }
    }
}
impl Add<Self> for ByteCount {
    type Output = Self;

    // TODO: remove
    fn add(self, rhs: Self) -> Self {
        match (self, rhs) {
            (ByteCount::Finite(l), ByteCount::Finite(r)) => ByteCount::Finite(l + r),
            (ByteCount::Infinite, _) | (_, ByteCount::Infinite) => ByteCount::Infinite,
            (ByteCount::Unknown, _) | (_, ByteCount::Unknown) => ByteCount::Unknown,
        }
    }
}
