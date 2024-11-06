use crate::prelude::*;
use crate::unsize::{
    AsBytes, AsMutBytes, RefBytes, RefBytesMut, RefWrapper, RefWrapperMutExt, RefWrapperTypes,
};
use advance::Advance;
use std::cell::{Ref, RefMut};
use std::fmt::Debug;
use std::mem::size_of;

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

/// Constant string comparison. Replaced when const traits enabled.
#[must_use]
pub const fn compare_strings(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    let mut index = 0;
    loop {
        if index >= a_bytes.len() {
            break true;
        }
        if a_bytes[index] != b_bytes[index] {
            break false;
        }
        index += 1;
    }
}

/// A ref that offsets bytes by a given amount.
#[derive(Debug, Copy, Clone)]
pub struct OffsetRef(pub usize);
unsafe impl<S> RefBytes<S> for OffsetRef
where
    S: AsBytes,
{
    fn bytes(wrapper: &RefWrapper<S, Self>) -> Result<&[u8]> {
        let mut bytes = wrapper.sup().as_bytes()?;
        bytes.try_advance(wrapper.r().0)?;
        Ok(bytes)
    }
}
unsafe impl<S> RefBytesMut<S> for OffsetRef
where
    S: AsMutBytes,
{
    fn bytes_mut(wrapper: &mut RefWrapper<S, Self>) -> Result<&mut [u8]> {
        let (sup, r) = unsafe { wrapper.s_r_mut() };
        let mut bytes = sup.as_mut_bytes()?;
        bytes.try_advance(r.0)?;
        Ok(bytes)
    }
}

/// Returns a slice of bytes from an array of [`NoUninit`] types.
pub fn uninit_array_bytes<T: NoUninit, const N: usize>(array: &[T; N]) -> &[u8] {
    // Safety: `T` is `NoUninit`, so all underlying reads are valid since there's no padding
    // between array elements. The pointer is valid. The entire memory is valid.
    // The size is correct. Everything is fine.
    unsafe { core::slice::from_raw_parts(array.as_ptr().cast::<u8>(), size_of::<T>() * N) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_compare_strings() {
        assert!(compare_strings("hello", "hello"));
        assert!(!compare_strings("hello", "world"));
        assert!(!compare_strings("hello", "hell"));
        assert!(!compare_strings("hello", "hellp"));
    }
}
