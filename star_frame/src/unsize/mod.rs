//! Zero-copy, dynamically sized, CU-efficient types.

pub mod impls;
pub mod init;
#[cfg(all(feature = "test_helpers", not(target_os = "solana")))]
mod test_helpers;
#[cfg(all(test, feature = "test_helpers"))]
mod tests;
pub mod wrapper;

use std::ops::Range;

pub use star_frame_proc::{unsized_impl, unsized_type};
#[cfg(all(feature = "test_helpers", not(target_os = "solana")))]
pub use test_helpers::*;

use crate::{ensure, ErrorCode, Result};

/// A helper trait that connects an [`UnsizedType::Mut`] to its parent [`UnsizedType`].
///
/// # Safety
/// The `UnsizedType` should almost always be the same as the `UnsizedTypePtr`'s parent.
/// The `check_pointers` method should ensure that the pointers inside the `UnsizedTypePtr`
/// are within the given range and are sequential (past the cursor which should be updated on each check).
/// This is to ensure that the pointers have not been swapped out with a `mem::swap` or similar operation.
pub unsafe trait UnsizedTypePtr {
    type UnsizedType: UnsizedType + ?Sized;
    /// Ensures that the pointers haven't been swapped out with another allocation.
    /// Cursor ensures each pointer is sequential, and is moved forward at each check
    fn check_pointers(&self, range: &Range<usize>, cursor: &mut usize) -> bool;
}

/// The core trait of the unsized type system, representing an effective 'zero copy' type that can be operated on a contiguous memory buffer.
///
/// # Safety
/// `Self::Ptr` should not be allowed to Copy itself from behind a reference. It should definitely not implement `Copy` or `Clone`, but any other methods also need to
/// ensure that invariant.
pub unsafe trait UnsizedType: 'static {
    type Ptr: UnsizedTypePtr;

    type Owned;

    /// This const should be true if there are no Zero-sized types in Self,
    /// false if a single ZST is at the end of Self, and panic if there is a ZST in the middle.
    const ZST_STATUS: bool;

    /// # Safety
    /// Implementations are allowed to modify the pointer and read from the data it points to, but should not write to it.
    ///
    /// We use raw pointers here to avoid invalidating the main data pointer through reborrowing, allowing us to pass Miri with the
    /// Tree Borrows aliasing model.
    ///
    /// This implementation should probably be correct as well. TODO: check if unsafe code relies on this being correct.
    unsafe fn get_ptr(data: &mut *mut [u8]) -> Result<Self::Ptr>;

    /// Gets the pointer to the start of the data for Self
    fn start_ptr(m: &Self::Ptr) -> *mut ();

    /// Gets the length of data that Self occupies
    fn data_len(m: &Self::Ptr) -> usize;

    fn owned(data: &[u8]) -> Result<Self::Owned> {
        let data: *const [u8] = data;
        Self::owned_from_ptr(&unsafe { Self::get_ptr(&mut data.cast_mut()) }?)
    }

    fn owned_from_ptr(r: &Self::Ptr) -> Result<Self::Owned>;

    /// # Safety
    /// No resize operations should be performed on the data.
    #[allow(unused_variables)]
    unsafe fn resize_notification(
        self_mut: &mut Self::Ptr,
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
    /// Advances the pointer by `advance` bytes, returning a pointer to the advanced over section.
    /// This uses the `wrapping_add` method on `*mut T` to advance the pointer, so while this method is safe to call,
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
    fn try_advance(&mut self, advance: usize) -> Result<Self>;
}

impl RawSliceAdvance for *mut [u8] {
    #[inline]
    fn try_advance(&mut self, advance: usize) -> Result<Self> {
        let len = self.len();
        ensure!(
            advance <= len,
            ErrorCode::RawSliceAdvance,
            "Tried to advance a raw slice by {advance} bytes, but the slice only has {len} bytes remaining"
        );
        let to_return = core::ptr::slice_from_raw_parts_mut(self.cast::<u8>(), advance);
        *self = core::ptr::slice_from_raw_parts_mut(
            self.cast::<u8>().wrapping_add(advance),
            len - advance,
        );
        Ok(to_return)
    }
}

/// Writing bytes of an [`UnsizedType::Owned`] to a buffer.
///
/// This is used in implementations for setting bytes of the underlying data of an [`UnsizedType`].
pub trait FromOwned: UnsizedType {
    /// The number of bytes that the [`UnsizedType::Owned`] will take up in the buffer.
    fn byte_size(owned: &Self::Owned) -> usize;

    /// Writes to and advances the buffer, returning the number of bytes advanced.
    ///
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

#[allow(unused)]
#[cfg(miri)]
extern "Rust" {
    fn miri_static_root(ptr: *const ());
}

/// Commonly used types and traits for the unsized type system.
pub mod prelude {
    use super::*;
    pub use super::{unsized_impl, unsized_type, UnsizedType};
    pub use impls::prelude::*;
    pub use init::DefaultInit;
    pub use wrapper::{ExclusiveRecurse, ExclusiveWrapper, ExclusiveWrapperTop};
}
