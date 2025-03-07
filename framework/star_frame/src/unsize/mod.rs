pub mod impls;
pub mod init;
#[cfg(feature = "test_helpers")]
mod test_helpers;
#[cfg(test)]
mod tests;
pub mod wrapper;
#[cfg(feature = "test_helpers")]
pub use test_helpers::*;

pub use star_frame_proc::{unsized_impl, unsized_type};

use crate::Result;

pub trait AsShared<'a> {
    type Shared<'b>
    where
        Self: 'a + 'b;

    fn as_shared(&'a self) -> Self::Shared<'a>;
}

impl<'a, T: ?Sized> AsShared<'a> for &'_ mut T {
    type Shared<'b> = &'b T where Self: 'a + 'b;

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

    fn get_ref<'a>(data: &mut &'a [u8]) -> Result<Self::Ref<'a>>;
    fn get_mut<'a>(data: &mut &'a mut [u8]) -> Result<Self::Mut<'a>>;
    fn owned(data: &mut &[u8]) -> Result<Self::Owned> {
        Self::owned_from_ref(Self::get_ref(data)?)
    }
    fn owned_from_ref(r: Self::Ref<'_>) -> Result<Self::Owned>;

    /// # Safety
    /// No resize operations should be performed on the data.
    #[allow(unused_variables)]
    unsafe fn resize_notification(data: &mut &mut [u8], operation: ResizeOperation) -> Result<()>;
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

#[derive(Copy, Clone, Debug)]
pub enum ResizeOperation {
    Add {
        start: *const (),
        amount: usize,
    },
    /// `start` is inclusive, `end` is exclusive.
    Remove {
        start: *const (),
        end: *const (),
    },
}
impl ResizeOperation {
    #[must_use]
    pub fn start(&self) -> *const () {
        match self {
            ResizeOperation::Remove { start, .. } | ResizeOperation::Add { start, .. } => *start,
        }
    }
}
