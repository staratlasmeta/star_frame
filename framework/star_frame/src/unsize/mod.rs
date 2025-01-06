mod impls;
mod init;
mod ref_wrapper;
mod resize;
#[cfg(feature = "test_helpers")]
mod test_helpers;
#[cfg(test)]
mod tests;
mod unsized_enum;

// todo: all glob imports? :/
pub use impls::*;
pub use init::*;
pub use ref_wrapper::*;
pub use resize::*;
pub use star_frame_proc::unsized_type;
#[cfg(feature = "test_helpers")]
pub use test_helpers::*;
pub use unsized_enum::*;

use crate::align1::Align1;
use anyhow::Result;
use bytemuck::{CheckedBitPattern, NoUninit, Zeroable};
use std::fmt::Debug;
use std::mem::size_of;
use typenum::{Bit, False, True};

/// Allows for zero copy deserialization of sized and unsized types over a set of bytes with [`AsBytes`].
///
/// The [`unsized_type`] attribute macro allows for the creation of structs with multiple unsized fields
/// under the hood using normal Rust syntax.
///
/// # Blanket Implementations
/// This trait is implemented for all types that meet [`UnsizedGenerics`]. This includes all [`Pod`](bytemuck::Pod) types that also implement [`Align1`].
///
/// # Macro Example
/// ```
/// use star_frame::prelude::*;
///
/// # fn main() -> Result<()> {
/// #[unsized_type(skip_idl)]
/// pub struct UnsizedTest<T0, T1>
/// where T0: UnsizedGenerics, T1: UnsizedGenerics
/// {
///     pub sized1: T0,
///     pub sized2: u8,
///     #[unsized_start]
///     pub unsized1: List<u8>,
///     pub unsized2: List<T1>,
/// }
///
/// // TestByteSet allows us to emulate how DataAccount works with on-chain AccountInfo data
/// let mut bytes = TestByteSet::<UnsizedTest<u8, [u8; 1]>>::new(DefaultInit)?;
/// let mut r = &mut bytes.mutable()?;
/// r.sized1 = 1;
/// r.unsized2()?.push([1])?;
/// r.unsized1()?.push(2)?;
///
/// assert_eq!(r.sized1, 1);
/// assert_eq!(r.unsized1()?.first().unwrap(), &2);
/// assert_eq!(&**(r.unsized2()?), &[[1]]);
///
/// # Ok(())
/// # }
/// ```
///
/// # Safety
/// [`UnsizedType::from_bytes`] must return correct values.
/// TODO: make this more descriptive
pub unsafe trait UnsizedType: 'static {
    /// Any extra metadata needed to rebuild the type. Usually an optimization to allow for rebuilding
    /// without reading all the inner data, such as with [`CombinedUnsized`].
    type RefMeta: 'static + Copy;
    /// What's stored in a [`RefWrapper`] return from [`UnsizedType::from_bytes`] and [`UnsizedType::from_bytes_and_meta`].
    /// Usually a unique type that stores an [`UnsizedType::RefMeta`] and adds relevant functions to the [`RefWrapper`].
    type RefData;
    /// The owned version of the underlying type. For example, a [`Vec<T>`] for an unsized [`List<T>`](crate::prelude::List).
    /// If [`Self::IsUnsized`] is [`False`], `Owned` can be [`Self`]. The data should try to convey
    /// the same information, but it doesn't have to be a 1:1 representation.
    type Owned;
    /// [`True`] if this type doesn't have a statically known size, [`False`] otherwise.
    type IsUnsized: Bit + LengthAccess<Self>;

    /// Reads this type from the provider `super_ref` and returns a deeper layer of [`RefWrapper`].
    fn from_bytes<S: AsBytes>(
        super_ref: S,
    ) -> Result<FromBytesReturn<S, Self::RefData, Self::RefMeta>>;

    /// Performs the same operation as [`UnsizedType::from_bytes`] but has access to a previously read [`UnsizedType::RefMeta`].
    /// This should be called where possible for optimization.
    /// # Safety
    /// `meta` must have come from a call to [`UnsizedType::from_bytes`] on the same `super_ref`.
    #[allow(unused_variables)]
    unsafe fn from_bytes_and_meta<S: AsBytes>(
        super_ref: S,
        meta: Self::RefMeta,
    ) -> Result<FromBytesReturn<S, Self::RefData, Self::RefMeta>> {
        Self::from_bytes(super_ref)
    }

    fn owned<S: AsBytes>(r: RefWrapper<S, Self::RefData>) -> Result<Self::Owned>;

    /// A convenience method to convert impl [`AsBytes`] to `Self::Owned`. This is mainly intended to be used on the client side.
    fn deserialize<S: AsBytes>(bytes: S) -> Result<Self::Owned> {
        Self::owned(Self::from_bytes(bytes)?.ref_wrapper)
    }
}
/// This is a helper trait for the [`UnsizedType`] trait. The required supertraits meet the [`CheckedBitPattern`] blanket implementation for [`UnsizedType`].
pub trait UnsizedGenerics: CheckedBitPattern + Align1 + NoUninit + Zeroable {}
impl<T> UnsizedGenerics for T where T: CheckedBitPattern + Align1 + NoUninit + Zeroable {}

/// This is a helper trait for the [`LengthAccess`] trait. It should probably not be implemented manually.
///
/// # Safety
/// [`Self::LENGTH`] must be the correct size of underlying type, which must be Sized
pub unsafe trait StaticLength: UnsizedType<IsUnsized = False> {
    const LENGTH: usize;
}
unsafe impl<T> StaticLength for T
where
    T: CheckedBitPattern + UnsizedType<IsUnsized = False>,
{
    const LENGTH: usize = size_of::<T>();
}
unsafe impl<T, U> StaticLength for CombinedUnsized<T, U>
where
    T: ?Sized + StaticLength,
    U: ?Sized + StaticLength,
{
    const LENGTH: usize = T::LENGTH + U::LENGTH;
}

/// # Safety
/// [`LengthAccess::len`] must return the same value that was passed to [`LengthAccess::from_len`].
pub unsafe trait LengthAccess<T: ?Sized> {
    type LengthData: 'static + Copy + Debug;

    fn from_len(len: usize) -> Self::LengthData;
    fn len(data: Self::LengthData) -> usize;
}
unsafe impl<T: ?Sized> LengthAccess<T> for True {
    type LengthData = usize;

    fn from_len(len: usize) -> Self::LengthData {
        len
    }
    fn len(data: Self::LengthData) -> usize {
        data
    }
}
unsafe impl<T> LengthAccess<T> for False
where
    T: StaticLength,
{
    type LengthData = ();

    fn from_len(_len: usize) -> Self::LengthData {}
    fn len(_data: Self::LengthData) -> usize {
        T::LENGTH
    }
}

/// A return type for [`UnsizedType::from_bytes`] and [`UnsizedType::from_bytes_and_meta`].
///
/// `S` is the `super_ref` passed in to `from_bytes`, `R` is the [`UnsizedType::RefData`], and
/// `M` is the [`UnsizedType::RefMeta`].
#[derive(Debug, Copy, Clone)]
pub struct FromBytesReturn<S, R, M> {
    /// How many bytes the [`UnsizedType::from_bytes`] used on the underlying bytes.
    pub bytes_used: usize,
    /// The resulting [`RefWrapper`] from the operation, where S is the `SuperRef`
    pub ref_wrapper: RefWrapper<S, R>,
    /// The resulting [`UnsizedType::RefData`] from the operation.
    pub meta: M,
}
impl<S, R, M> FromBytesReturn<S, R, M> {
    /// Maps the inner data using a mapper function `f`.
    ///
    /// # Safety
    /// Same requirements as [`RefWrapper::wrap_r`].
    pub unsafe fn map_ref<R2>(self, f: impl FnOnce(&mut S, R) -> R2) -> FromBytesReturn<S, R2, M> {
        FromBytesReturn {
            ref_wrapper: unsafe { self.ref_wrapper.wrap_r(f) },
            meta: self.meta,
            bytes_used: self.bytes_used,
        }
    }

    /// Maps the meta using a mapper function `f`.
    ///
    /// # Safety
    /// Meta must be correct for [`FromBytesReturn::ref_wrapper`].
    pub unsafe fn map_meta<M2>(self, f: impl FnOnce(M) -> M2) -> FromBytesReturn<S, R, M2> {
        FromBytesReturn {
            ref_wrapper: self.ref_wrapper,
            meta: f(self.meta),
            bytes_used: self.bytes_used,
        }
    }
}
