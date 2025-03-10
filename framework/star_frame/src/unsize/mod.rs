pub mod impls;
pub mod init;
#[cfg(feature = "test_helpers")]
mod test_helpers;
#[cfg(test)]
mod tests;
pub mod wrapper;
#[cfg(feature = "test_helpers")]
pub use test_helpers::*;

use crate::Result;
pub use star_frame_proc::{unsized_impl, unsized_type};

pub trait AsShared<'a> {
    type Shared<'b>
    where
        Self: 'a + 'b;

    fn as_shared(&'a self) -> Self::Shared<'a>;
}

impl<'a, T: ?Sized> AsShared<'a> for &'_ mut T {
    type Shared<'b> = &'b T
    where
        Self: 'a + 'b;

    fn as_shared(&'a self) -> Self::Shared<'a> {
        self
    }
}

/// # Safety
/// TODO
pub unsafe trait UnsizedType: 'static {
    type Ref<'a>;
    type Mut<'a>: AsShared<'a, Shared<'a> = Self::Ref<'a>>;
    type Owned;

    /// This const should be true if there are no Zero-sized types in Self,
    /// false if a single ZST is at the end of Self, and panic if there is a ZST in the middle.
    const ZST_STATUS: bool;

    fn get_ref<'a>(data: &mut &'a [u8]) -> Result<Self::Ref<'a>>;
    fn get_mut<'a>(data: &mut &'a mut [u8]) -> Result<Self::Mut<'a>>;
    fn owned(data: &mut &[u8]) -> Result<Self::Owned> {
        Self::owned_from_ref(Self::get_ref(data)?)
    }
    fn owned_from_ref(r: Self::Ref<'_>) -> Result<Self::Owned>;

    /// # Safety
    /// No resize operations should be performed on the data.
    #[allow(unused_variables)]
    unsafe fn resize_notification(
        self_mut: &mut Self::Mut<'_>,
        source_ptr: *const (),
        change: isize,
    ) -> Result<()>;
}

/// Helper macro to call `resize_notification` on all types in a tuple. This should mainly only
/// be used within the [`unsized_type`] macro.
#[doc(hidden)]
#[macro_export]
macro_rules! __resize_notification_checked {
    ($r:ident, $operation:ident -> $($ty:ty),* $(,)?) => {
        $(if $operation.start() > $r.as_ptr().cast() {
            unsafe { <$ty as $crate::unsize::UnsizedType>::resize_notification($r, $operation) }?;
        } else {
            return $crate::anyhow::Ok(());
        })*
        return $crate::anyhow::Ok(());
    };
}

/// ```compile_fail
/// use star_frame::prelude::*;
/// use star_frame::unsize::TestByteSet;
/// #[unsized_type(skip_idl)]
/// struct SizedZst {
///     field1: (),
///     #[unsized_start]
///     list: List<u8>,
/// }
/// let test_bytes = TestByteSet::<SizedZst>::new_default().unwrap();
/// let data_mut = test_bytes.data_mut().unwrap();
/// ```
#[cfg(doctest)]
struct TestZstCompileFail;
