pub mod impls;
pub mod init;
mod owned_ref;
#[cfg(all(feature = "test_helpers", not(target_os = "solana")))]
mod test_helpers;
#[cfg(all(test, feature = "test_helpers"))]
mod tests;
pub mod wrapper;

use anyhow::ensure;
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

    fn get_ref<'a>(data: &mut &'a [u8]) -> Result<Self::Ref<'a>>;

    /// # Safety
    /// The pointer must be valid for `'a`, and the length metadata must not be more than the slice's actual
    /// length. Implementations are allowed to modify the pointer and read from the data it points to, but should not write to it.
    ///
    /// We use raw pointers here to avoid invalidating the main data pointer through reborrowing, allowing us to pass Miri with the
    /// Tree Borrows aliasing model.
    ///
    /// This implementation should probably be correct as well. TODO: check if unsafe code relies on this being correct.
    unsafe fn get_mut<'a>(data: &mut *mut [u8]) -> Result<Self::Mut<'a>>;

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

#[doc(hidden)]
mod sealed {
    pub trait Sealed {}
    impl Sealed for *mut [u8] {}
}

/// Convenience trait for advancing a raw pointer. See the sole implementation for `*mut [u8]` for more details.
pub trait RawSliceAdvance: Sized + sealed::Sealed {
    fn try_advance(&mut self, advance: usize) -> Result<Self>;
}

impl RawSliceAdvance for *mut [u8] {
    /// Advances the pointer by `advance` bytes, returning a pointer to the advanced over section.
    /// This uses [`Self::wrapping_add`] to advance the pointer, so while this method is safe to call,
    /// the resulting pointer may be unsafe to dereference if Self and its metadata (slice length) are not valid.
    ///
    /// # Examples
    /// ```
    /// # use star_frame::unsize::RawSliceAdvance;
    ///
    /// let mut array = [1, 2, 3, 4, 5];
    /// let mut ptr = core::ptr::from_mut(&mut array[..]);
    /// let first_part = ptr.try_advance(3).unwrap();
    ///
    /// assert_eq!(first_part.len(), 3);
    /// assert_eq!(ptr.len(), 2);
    ///
    /// let first_part = unsafe { &*first_part };
    /// let remaining = unsafe { &*ptr };
    ///
    /// assert_eq!(first_part, &[1, 2, 3]);
    /// assert_eq!(remaining, &[4, 5]);
    /// ```
    #[inline]
    fn try_advance(&mut self, advance: usize) -> Result<Self> {
        let len = self.len();
        ensure!(advance <= len, "advance is out of bounds");
        let to_return = core::ptr::slice_from_raw_parts_mut(self.cast::<u8>(), advance);
        *self = core::ptr::slice_from_raw_parts_mut(
            self.cast::<u8>().wrapping_add(advance),
            len - advance,
        );
        Ok(to_return)
    }
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
