pub mod impls;
pub mod init;
mod owned_ref;
#[cfg(all(feature = "test_helpers", not(target_os = "solana")))]
mod test_helpers;
#[cfg(all(test, feature = "test_helpers"))]
mod tests;
pub mod wrapper;

pub use owned_ref::*;
pub use star_frame_proc::{unsized_impl, unsized_type};
#[cfg(all(feature = "test_helpers", not(target_os = "solana")))]
pub use test_helpers::*;

use crate::Result;

pub trait AsShared {
    type Ref<'a>
    where
        Self: 'a;
    fn as_shared(&self) -> Self::Ref<'_>;
}

pub type UnsizedTypeMut<'a, T> = <T as UnsizedType>::Mut<'a>;

/// # Safety
/// TODO
pub unsafe trait UnsizedType: 'static {
    type Ref<'a>;
    type Mut<'a>: AsShared<Ref<'a> = Self::Ref<'a>>;

    type Owned;

    /// This const should be true if there are no Zero-sized types in Self,
    /// false if a single ZST is at the end of Self, and panic if there is a ZST in the middle.
    const ZST_STATUS: bool;

    fn mut_as_ref<'a>(m: &'a Self::Mut<'_>) -> Self::Ref<'a>;

    /// # Safety
    /// Variance is complicated. Everything that is assigned to the return value must be at least 'a.
    /// This should really only be called from within the [`unsized_type`] macro. Be careful if you need to call it directly.
    unsafe fn sub_ref_mut<'a: 'b, 'b>(r: &'b mut Self::Mut<'a>) -> Self::Mut<'b> {
        todo!()
    }

    fn get_ref<'a>(data: &mut &'a [u8]) -> Result<Self::Ref<'a>>;
    fn get_mut<'a>(data: &mut &'a mut [u8]) -> Result<Self::Mut<'a>>;
    fn owned(mut data: &[u8]) -> Result<Self::Owned> {
        let data = &mut data;
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

/// # Safety
/// The `from_owned` function must create a valid `Self` from the `owned` value.
pub unsafe trait FromOwned: UnsizedType {
    fn byte_size(owned: &Self::Owned) -> usize;

    /// Writes to and advances the buffer, returning the number of bytes advanced.
    /// Errors if the buffer is too small (< `byte_size`).
    fn from_owned(owned: Self::Owned, bytes: &mut &mut [u8]) -> Result<usize>;
}

// todo: convert these tests to TryBuild
/// # Test ZST on sized
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
///
/// # Test ZST at end
/// ```
/// use star_frame::prelude::*;
/// use star_frame::unsize::TestByteSet;
/// #[unsized_type(skip_idl)]
/// struct ZstAtEnd {
///     field1: u8,
///     #[unsized_start]
///     remaining: RemainingBytes,
/// }
/// let test_bytes = TestByteSet::<ZstAtEnd>::new_default().unwrap();
/// let data_mut = test_bytes.data_mut().unwrap();
/// ```
///
/// # Test nested ZST
/// ```compile_fail
/// use star_frame::prelude::*;
/// use star_frame::unsize::TestByteSet;
/// #[unsized_type(skip_idl)]
/// struct ZstAtEnd {
///     field1: u8,
///     #[unsized_start]
///     remaining: RemainingBytes,
/// }
///
/// #[unsized_type(skip_idl)]
/// struct NestedZst {
///     field1: u8,
///     #[unsized_start]
///     zst_in_middle: ZstAtEnd,
///     list: List<u8>
/// }
/// let test_bytes = TestByteSet::<NestedZst>::new_default().unwrap();
/// let data_mut = test_bytes.data_mut().unwrap();
/// ```
#[cfg(doctest)]
struct TestUnsizedZst;
