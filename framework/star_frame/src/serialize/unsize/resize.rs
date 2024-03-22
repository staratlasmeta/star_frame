use crate::serialize::ref_wrapper::AsMutBytes;
use crate::Result;

/// # Safety
/// Implementation must ensure that the underlying data is resized and properly handle metadata.
pub unsafe trait Resize<M>: AsMutBytes {
    /// # Safety
    /// Should only be called by implementations for types and not user code.
    unsafe fn resize(&mut self, new_byte_len: usize, new_meta: M) -> Result<()>;
}
unsafe impl<M> Resize<M> for Vec<u8> {
    unsafe fn resize(&mut self, new_byte_len: usize, _meta: M) -> Result<()> {
        self.resize(new_byte_len, 0);
        Ok(())
    }
}
