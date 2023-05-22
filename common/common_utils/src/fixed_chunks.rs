use array_init::try_array_init;
use std::iter::FusedIterator;

/// An iterator extension that allows an iterator to be chunked into fixed sized arrays.
pub trait FixedChunks: Iterator {
    /// Chunks the iterator into fixed sized arrays.
    /// Ignores excess items (`self.len() % N`).
    fn fixed_chunks<const N: usize>(self) -> FixedChunksImpl<Self, N>
    where
        Self: Sized;
}
impl<T> FixedChunks for T
where
    T: Iterator,
{
    fn fixed_chunks<const N: usize>(self) -> FixedChunksImpl<Self, N>
    where
        Self: Sized,
    {
        FixedChunksImpl(self)
    }
}

/// The iterator for [`FixedChunks::fixed_chunks`].
#[derive(Debug, Clone)]
pub struct FixedChunksImpl<I, const N: usize>(I);
impl<I, const N: usize> Iterator for FixedChunksImpl<I, N>
where
    I: Iterator,
{
    type Item = [I::Item; N];

    fn next(&mut self) -> Option<Self::Item> {
        try_array_init(|_| match self.0.next() {
            Some(item) => Ok(item),
            None => Err(()),
        })
        .ok()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.0.size_hint();
        (lower / N, upper.map(|upper| upper / N))
    }
}
impl<I, const N: usize> ExactSizeIterator for FixedChunksImpl<I, N>
where
    I: ExactSizeIterator,
{
    fn len(&self) -> usize {
        self.0.len() / N
    }
}
impl<I, const N: usize> FusedIterator for FixedChunksImpl<I, N> where I: FusedIterator {}
