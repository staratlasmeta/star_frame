use crate::unsize::ref_wrapper::AsMutBytes;
use crate::Result;

/// # Safety
/// Implementation must ensure that the underlying data is resized and properly handle metadata.
pub unsafe trait Resize<M>: AsMutBytes {
    /// # Safety
    /// Should only be called by implementations for types and not user code.
    unsafe fn resize(s: &mut Self, new_byte_len: usize, new_meta: M) -> Result<()>;

    /// # Safety
    /// Should only be called by implementations for types and not user code.
    unsafe fn set_meta(s: &mut Self, new_meta: M) -> Result<()>;
}
unsafe impl<'a, T, M> Resize<M> for &'a mut T
where
    T: Resize<M>,
{
    unsafe fn resize(s: &mut Self, new_byte_len: usize, new_meta: M) -> Result<()> {
        unsafe { T::resize(*s, new_byte_len, new_meta) }
    }

    unsafe fn set_meta(s: &mut Self, new_meta: M) -> Result<()> {
        unsafe { T::set_meta(*s, new_meta) }
    }
}
unsafe impl<M> Resize<M> for Vec<u8> {
    unsafe fn resize(s: &mut Self, new_byte_len: usize, _new_meta: M) -> Result<()> {
        s.resize(new_byte_len, 0);
        Ok(())
    }

    unsafe fn set_meta(_s: &mut Self, _new_meta: M) -> Result<()> {
        Ok(())
    }
}
