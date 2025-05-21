use crate::Result;
use core::ptr;
use derive_more::{self, Deref, DerefMut};
use std::cell::{Ref, RefMut};
use std::convert::Infallible;

#[derive(Debug, Deref)]
pub(crate) struct OwnedRef<'a, T> {
    #[deref]
    data: T,
    r: Ref<'a, ()>,
}

#[allow(unused)]
impl<'a, T> OwnedRef<'a, T> {
    pub fn new<I: ?Sized>(r: Ref<'a, I>, mapper: impl FnOnce(&'a I) -> T) -> Self
    where
        T: 'a,
    {
        Self::try_new::<_, Infallible>(r, |r| Ok(mapper(r))).unwrap()
    }

    pub fn try_new<I: ?Sized, E>(
        r: Ref<'a, I>,
        mapper: impl FnOnce(&'a I) -> Result<T, E>,
    ) -> Result<Self, E>
    where
        T: 'a,
    {
        // SAFETY: this whole object lasts 'a, so there is no opportunity to leak the reference
        let ref_data: &'a I = unsafe { &*ptr::from_ref(&*r) };
        let data = mapper(ref_data)?;
        Ok(OwnedRef {
            data,
            r: Ref::map(r, |_| &()),
        })
    }

    pub fn map<U>(s: Self, f: impl FnOnce(T) -> U) -> OwnedRef<'a, U> {
        OwnedRef {
            data: f(s.data),
            r: s.r,
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn clone(s: &Self) -> OwnedRef<'a, T>
    where
        T: Clone,
    {
        OwnedRef {
            data: s.data.clone(),
            r: Ref::clone(&s.r),
        }
    }
}

#[derive(Debug, Deref, DerefMut)]
pub(crate) struct OwnedRefMut<'a, T> {
    #[deref]
    #[deref_mut]
    data: T,
    r: RefMut<'a, [u8]>,
}

#[allow(unused)]
impl<'a, T> OwnedRefMut<'a, T> {
    pub fn new<I: ?Sized>(r: RefMut<'a, I>, mapper: impl FnOnce(&'a mut I) -> T) -> Self
    where
        T: 'a,
    {
        Self::try_new::<_, Infallible>(r, |r| Ok(mapper(r))).unwrap()
    }

    pub fn try_new<I: ?Sized, E>(
        mut r: RefMut<'a, I>,
        mapper: impl FnOnce(&'a mut I) -> Result<T, E>,
    ) -> Result<Self, E>
    where
        T: 'a,
    {
        // SAFETY: this whole object lasts 'a, so there is no opportunity to leak the reference
        let ref_data: &'a mut I = unsafe { &mut *ptr::from_mut(&mut *r) };
        let data = mapper(ref_data)?;
        Ok(OwnedRefMut {
            data,
            r: RefMut::map(r, |_| &mut []),
        })
    }

    pub fn map<U>(s: Self, f: impl FnOnce(T) -> U) -> OwnedRefMut<'a, U> {
        OwnedRefMut {
            data: f(s.data),
            r: s.r,
        }
    }
}
