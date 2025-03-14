use crate::prelude::*;
use std::cell::{Ref, RefMut};
use std::mem::size_of;

/// Similar to [`Ref::map`], but the closure can return an error.
#[inline]
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

/// Similar to [`RefMut::map`], but the closure can return an error
#[inline]
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

/// Returns a slice of bytes from an array of [`NoUninit`] types.
#[inline]
pub fn uninit_array_bytes<T: NoUninit, const N: usize>(array: &[T; N]) -> &[u8] {
    // Safety: `T` is `NoUninit`, so all underlying reads are valid since there's no padding
    // between array elements. The pointer is valid. The entire memory is valid.
    // The size is correct. Everything is fine.
    unsafe { core::slice::from_raw_parts(array.as_ptr().cast::<u8>(), size_of::<T>() * N) }
}

pub mod borsh_bytemuck {
    use crate::align1::Align1;
    use bytemuck::{CheckedBitPattern, NoUninit};
    use std::io::{Read, Write};
    use std::mem::{size_of, MaybeUninit};
    use std::ptr;

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
    ) -> std::io::Result<()> {
        let bytes = bytemuck::bytes_of(value);
        writer.write_all(bytes)
    }
    // todo: figure out if theres a way to make this more optimized. If we could
    //  create just an array [0u8; std::mem::size_of::<P>()], that would probably match
    //  borsh or exceed borsh. If we really need this, we can use the unstable `generic_const_exprs`
    //  feature.
    #[deprecated = "try using `BorshDeserialize` directly. This is much less efficient."]
    pub fn deserialize<R: Read, P: NoUninit + CheckedBitPattern + Align1>(
        reader: &mut R,
    ) -> std::io::Result<P> {
        let mut buffer = MaybeUninit::<P>::uninit();
        let bytes =
            unsafe { &mut *ptr::from_raw_parts_mut(buffer.as_mut_ptr().cast(), size_of::<P>()) };
        reader.read_exact(bytes)?;
        bytemuck::checked::try_from_bytes::<P>(bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(unsafe { buffer.assume_init() })
        //
        // let mut value_bytes = vec![0; std::mem::size_of::<P>()];
        // reader.read_exact(&mut value_bytes)?;
        // let value: &P = bytemuck::checked::try_from_bytes(&value_bytes)
        //     .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        // Ok(*value)
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
