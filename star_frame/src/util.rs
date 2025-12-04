//! Useful miscellaneous functions.
use crate::prelude::*;
use core::{
    cell::{Ref, RefMut},
    mem::size_of,
};

/// Similar to [`Ref::map`], but the closure can return an error.
#[inline]
pub fn try_map_ref<'a, I: 'a + ?Sized, O: 'a + ?Sized, E>(
    r: Ref<'a, I>,
    f: impl FnOnce(&I) -> Result<&O, E>,
) -> Result<Ref<'a, O>, E> {
    // SAFETY:
    // We don't extend the lifetime of the reference beyond what it is.
    unsafe {
        let result = f(&r)? as *const O;
        Ok(Ref::map(r, |_| &*result))
    }
}

/// Similar to [`RefMut::map`], but the closure can return an error
#[inline]
pub fn try_map_ref_mut<'a, I: 'a + ?Sized, O: 'a + ?Sized, E>(
    mut r: RefMut<'a, I>,
    f: impl FnOnce(&mut I) -> Result<&mut O, E>,
) -> Result<RefMut<'a, O>, E> {
    // SAFETY:
    // We don't extend the lifetime of the reference beyond what it is.
    unsafe {
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

/// Returns a slice of bytes from an array of [`NoUninit`] types.
#[inline]
pub fn uninit_array_bytes<T: NoUninit, const N: usize>(array: &[T; N]) -> &[u8] {
    // SAFETY:
    // `T` is `NoUninit`, so all underlying reads are valid since there's no padding
    // between array elements. The pointer is valid. The entire memory is valid.
    // The size is correct. Everything is fine.
    unsafe { core::slice::from_raw_parts(array.as_ptr().cast::<u8>(), size_of::<T>() * N) }
}

/// Quicker way to compare 32 bytes.
///
/// Adapted from [Typhoon](https://github.com/exotic-markets-labs/typhoon/blob/60c5197cc632f1bce07ba27876669e4ca8580421/crates/accounts/src/utils.rs#L2)
#[inline]
#[must_use]
pub fn fast_32_byte_eq(a: &[u8; 32], b: &[u8; 32]) -> bool {
    bytemuck::cast_slice::<_, PackedValue<u64>>(a) == bytemuck::cast_slice::<_, PackedValue<u64>>(b)
}

pub trait FastAddressEq<T> {
    fn fast_eq(&self, other: &T) -> bool;
}

impl FastAddressEq<Address> for Address {
    #[inline]
    fn fast_eq(&self, other: &Address) -> bool {
        fast_32_byte_eq(self.as_array(), other.as_array())
    }
}

impl FastAddressEq<[u8; 32]> for Address {
    #[inline]
    fn fast_eq(&self, other: &[u8; 32]) -> bool {
        fast_32_byte_eq(self.as_array(), other)
    }
}

impl FastAddressEq<[u8; 32]> for [u8; 32] {
    #[inline]
    fn fast_eq(&self, other: &[u8; 32]) -> bool {
        fast_32_byte_eq(self, other)
    }
}

impl FastAddressEq<Address> for [u8; 32] {
    #[inline]
    fn fast_eq(&self, other: &Address) -> bool {
        fast_32_byte_eq(self, other.as_array())
    }
}

/// Custom [`borsh`] derive `serialize_with` and `deserialize_with` overrides for use with [`bytemuck`] types.
pub mod borsh_bytemuck {
    use super::*;
    use borsh::io::{Read, Write};
    use core::mem::MaybeUninit;

    /// Custom `serialize_with` override for [`borsh::BorshSerialize`] that uses [`bytemuck`] to serialize.
    /// This is intended for packed structs that are probably used in account data.
    ///
    /// # Example
    /// ```
    /// use borsh::BorshSerialize;
    /// use star_frame::prelude::*;
    ///
    /// #[derive(Align1, NoUninit, Copy, Clone)]
    /// #[repr(C, packed)]
    /// pub struct SomePackedThing {
    ///     pub a: u32,
    ///     pub b: u64,
    /// }
    ///
    /// #[derive(BorshSerialize)]
    /// pub struct SomeBorshThing {
    ///     #[borsh(serialize_with = "borsh_bytemuck::serialize")]
    ///     pub packed_thing: SomePackedThing,
    /// }
    ///```
    pub fn serialize<W: Write, P: NoUninit + Align1>(
        value: &P,
        writer: &mut W,
    ) -> borsh::io::Result<()> {
        let bytes = bytemuck::bytes_of(value);
        writer.write_all(bytes)
    }

    /// Custom `deserialize_with` override for [`borsh::BorshDeserialize`] that uses [`bytemuck`] to deserialize.
    /// This is intended for packed structs that are probably used in account data.
    ///
    /// # Example
    /// ```
    /// use borsh::BorshDeserialize;
    /// use star_frame::prelude::*;
    ///
    /// #[derive(Align1, NoUninit, Copy, Clone, CheckedBitPattern)]
    /// #[repr(C, packed)]
    /// pub struct SomePackedThing {
    ///     pub a: u32,
    ///     pub b: u64,
    /// }
    ///
    /// #[derive(BorshDeserialize)]
    /// pub struct SomeBorshThing {
    ///     #[borsh(deserialize_with = "borsh_bytemuck::deserialize")]
    ///     pub packed_thing: SomePackedThing,
    /// }
    /// ```
    pub fn deserialize<R: Read, P: NoUninit + CheckedBitPattern + Align1>(
        reader: &mut R,
    ) -> borsh::io::Result<P> {
        let mut buffer = MaybeUninit::<P>::zeroed();
        let bytes = unsafe {
            &mut *ptr_meta::from_raw_parts_mut(buffer.as_mut_ptr().cast::<()>(), size_of::<P>())
        };
        reader.read_exact(bytes)?;
        bytemuck::checked::try_from_bytes::<P>(bytes)
            .map_err(|e| borsh::io::Error::new(borsh::io::ErrorKind::InvalidData, e.to_string()))?;
        Ok(unsafe { buffer.assume_init() })
    }

    /// Derives [`BorshSerialize`](borsh::BorshSerialize) and [`BorshDeserialize`](borsh::BorshDeserialize) for [`bytemuck`] types.
    ///
    /// # Example
    /// ```
    /// use star_frame::prelude::*;
    ///
    /// #[derive(Align1, NoUninit, CheckedBitPattern, Copy, Clone)]
    /// #[repr(C, packed)]
    /// pub struct SomePackedThing {
    ///     pub a: u32,
    ///     pub b: u64,
    /// }
    ///
    /// borsh_with_bytemuck!(SomePackedThing);
    /// ```
    #[macro_export]
    macro_rules! borsh_with_bytemuck {
        ($($ty:ident),*) => {
            $(
                impl $crate::borsh::BorshSerialize for $ty {
                    fn serialize<W: $crate::borsh::io::Write>(&self, writer: &mut W) -> $crate::borsh::io::Result<()> {
                        $crate::util::borsh_bytemuck::serialize(self, writer)
                    }
                }

                impl $crate::borsh::BorshDeserialize for $ty {
                    fn deserialize_reader<R: $crate::borsh::io::Read>(reader: &mut R) -> $crate::borsh::io::Result<Self> {
                        $crate::util::borsh_bytemuck::deserialize(reader)
                    }
                }
            )*
        };
    }
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
