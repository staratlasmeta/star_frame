use std::iter::Fuse;

/// Chains with exact size. Panics on overflow.
pub trait ChainExact: Sized + ExactSizeIterator {
    /// Chains with exact size. Panics on overflow.
    fn chain_exact<I>(self, other: I) -> ChainExactImpl<Self, I::IntoIter>
    where
        I: IntoIterator<Item = Self::Item>,
        I::IntoIter: ExactSizeIterator;
}
impl<I1> ChainExact for I1
where
    I1: ExactSizeIterator,
{
    fn chain_exact<I2>(self, other: I2) -> ChainExactImpl<Self, I2::IntoIter>
    where
        I2: IntoIterator<Item = Self::Item>,
        I2::IntoIter: ExactSizeIterator,
    {
        ChainExactImpl(self.fuse(), other.into_iter().fuse())
    }
}

/// Impl for [`ChainExact::chain_exact`].
#[derive(Debug, Clone)]
pub struct ChainExactImpl<I1, I2>(Fuse<I1>, Fuse<I2>);
impl<I1, I2> Iterator for ChainExactImpl<I1, I2>
where
    I1: ExactSizeIterator,
    I2: ExactSizeIterator<Item = I1::Item>,
{
    type Item = I1::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.next() {
            None => self.1.next(),
            Some(val) => Some(val),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self
            .0
            .len()
            .checked_add(self.1.len())
            .expect("Size overflow");
        (len, Some(len))
    }
}
impl<I1, I2> ExactSizeIterator for ChainExactImpl<I1, I2>
where
    I1: ExactSizeIterator,
    I2: ExactSizeIterator<Item = I1::Item>,
{
}
