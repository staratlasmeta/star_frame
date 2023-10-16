use std::ops::{Deref, DerefMut};

/// A wrapper around a value that may be a reference or owned.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum MaybeRef<'a, T> {
    /// Owned value.
    Owned(T),
    /// Reference.
    Ref(&'a T),
}
impl<'a, T> Deref for MaybeRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Owned(val) => val,
            Self::Ref(val) => val,
        }
    }
}

/// A wrapper around a value that may be a mutable reference or owned.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MaybeMutRef<'a, T> {
    /// Owned value.
    Owned(T),
    /// Mutable reference.
    Mut(&'a mut T),
}
impl<'a, T> Deref for MaybeMutRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Owned(val) => val,
            Self::Mut(val) => val,
        }
    }
}
impl<'a, T> DerefMut for MaybeMutRef<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Owned(val) => val,
            Self::Mut(val) => val,
        }
    }
}
