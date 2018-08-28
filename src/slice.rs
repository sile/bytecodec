//! Slice like types.

/// Slice for owned types.
///
/// # Examples
///
/// ```
/// use bytecodec::slice::OwnedSlice;
///
/// let slice = OwnedSlice::new([1, 2, 3, 4], 1, 3);
/// assert_eq!(slice.as_ref(), [2, 3]);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OwnedSlice<T> {
    inner: T,
    start: usize,
    end: usize,
}
impl<T> OwnedSlice<T> {
    /// Makes a new `OwnedSlice` instance with the given range.
    pub fn new(inner: T, start: usize, end: usize) -> Self {
        OwnedSlice { inner, start, end }
    }

    /// Returns the start position of the slice.
    pub fn start(&self) -> usize {
        self.start
    }

    /// Returns the end position of the slice.
    pub fn end(&self) -> usize {
        self.end
    }

    /// Returns a reference to the inner value.
    pub fn inner_ref(&self) -> &T {
        &self.inner
    }

    /// Returns a mutable reference to the inner value.
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Takes ownership of the instance and returns the inner value.
    pub fn into_inner(self) -> T {
        self.inner
    }
}
impl<T, U> AsRef<[U]> for OwnedSlice<T>
where
    T: AsRef<[U]>,
{
    fn as_ref(&self) -> &[U] {
        &self.inner.as_ref()[self.start..self.end]
    }
}
impl<T, U> AsMut<[U]> for OwnedSlice<T>
where
    T: AsMut<[U]>,
{
    fn as_mut(&mut self) -> &mut [U] {
        &mut self.inner.as_mut()[self.start..self.end]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owned_slice_works() {
        let mut slice = OwnedSlice::new([1, 2, 3, 4], 1, 3);
        assert_eq!(slice.as_ref(), [2, 3]);

        slice.as_mut()[0] = 9;
        assert_eq!(slice.into_inner(), [1, 9, 3, 4]);
    }
}
