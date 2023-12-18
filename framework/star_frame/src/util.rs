use std::cell::{Ref, RefMut};
use std::fmt::Debug;
use std::mem::align_of;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    MainNet,
    DevNet,
    TestNet,
    Custom(&'static str),
}
#[cfg(feature = "idl")]
impl From<Network> for star_frame_idl::Network {
    fn from(value: Network) -> Self {
        match value {
            Network::MainNet => Self::MainNet,
            Network::DevNet => Self::DevNet,
            Network::TestNet => Self::TestNet,
            Network::Custom(c) => Self::Custom(c.to_string()),
        }
    }
}

/// Similar to [`Ref::map`], but the closure can return an error.
pub fn try_map_ref<'a, I: 'a + ?Sized, O: 'a + ?Sized, E>(
    r: Ref<'a, I>,
    f: impl FnOnce(&I) -> Result<&O, E>,
) -> Result<Ref<'a, O>, E> {
    // Safety: We don't extend the lifetime of the reference beyond what it is.
    unsafe {
        // let value: &'a I = &*(&*r as *const I); // &*:( => &:) Since :( impl deref => :)
        let result = f(&r)? as *const O;
        Ok(Ref::map(r, |_| &*result))
    }
}

/// Similar to [`RefMut::map`], but the closure can return an error.
pub fn try_map_ref_mut<'a, I: 'a + ?Sized, O: 'a + ?Sized, E>(
    mut r: RefMut<'a, I>,
    f: impl FnOnce(&mut I) -> Result<&mut O, E>,
) -> Result<RefMut<'a, O>, E> {
    // Safety: We don't extend the lifetime of the reference beyond what it is.
    unsafe {
        // let value: &'a mut I = &mut *(&mut *r as *mut I);
        let result = f(&mut r)? as *mut O;
        Ok(RefMut::map(r, |_| &mut *result))
    }
}

/// Manual implementation for checking if a pointer is aligned.
pub trait PtrIsAligned {
    /// Checks if this pointer is aligned.
    fn ptr_is_aligned(&self) -> bool;
}
impl<T> PtrIsAligned for *const T {
    fn ptr_is_aligned(&self) -> bool {
        *self as usize & (align_of::<T>() - 1) == 0
    }
}
impl<T> PtrIsAligned for *mut T {
    fn ptr_is_aligned(&self) -> bool {
        *self as usize & (align_of::<T>() - 1) == 0
    }
}
