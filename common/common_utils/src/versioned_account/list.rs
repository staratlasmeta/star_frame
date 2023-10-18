//! A list of items that can be stored in an account.

use crate::versioned_account::context::AccountDataMutContext;
use crate::versioned_account::to_from_usize::ToFromUsize;
use crate::versioned_account::unsized_data::UnsizedData;
use crate::{Result, UtilError};
use anchor_lang::error;
use bytemuck::{from_bytes, Pod};
use common_utils::align1::Align1;
use common_utils::{Advance, PackedValue};
use derivative::Derivative;
use solana_program::program_memory::sol_memmove;
use std::collections::Bound;
use std::fmt::Debug;
use std::mem::size_of;
use std::ops::{Deref, DerefMut, RangeBounds};
use std::ptr;
use std::ptr::NonNull;

/// A list of items that can be stored in an account.
#[repr(C)]
#[derive(Align1, Derivative)]
#[derivative(Debug(bound = "T: Debug, L: Copy + Debug"))]
pub struct List<T, L> {
    length: PackedValue<L>,
    list: [T],
}
impl<T, L> Deref for List<T, L> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.list
    }
}
impl<T, L> DerefMut for List<T, L> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.list
    }
}

// Safety: Pointers are the same as input.
unsafe impl<T: Align1 + Pod, L: ToFromUsize + Pod> UnsizedData for List<T, L> {
    type Metadata = ();

    #[inline]
    fn init_data_size() -> usize {
        size_of::<L>() + size_of::<T>() * L::zeroed().to_usize().unwrap()
    }

    unsafe fn init(bytes: &mut [u8]) -> Result<(&mut Self, Self::Metadata)> {
        assert_eq!(bytes.len(), Self::init_data_size());
        Ok((
            &mut *ptr::from_raw_parts_mut(
                bytes.as_mut_ptr().cast(),
                L::zeroed().to_usize().unwrap(),
            ),
            (),
        ))
    }

    fn from_bytes<'a>(bytes: &mut &'a [u8]) -> Result<(&'a Self, Self::Metadata)> {
        let bytes_advance = &mut &**bytes;
        let length: &PackedValue<L> = from_bytes(bytes_advance.try_advance(size_of::<L>())?);
        let length = length.0.to_usize()?;

        Ok((
            // Safety: We don't change lifetimes and limit the size of the slice.
            unsafe {
                &*ptr::from_raw_parts(
                    bytes
                        .try_advance(size_of::<L>() + size_of::<T>() * length)?
                        .as_ptr()
                        .cast::<()>(),
                    length,
                )
            },
            (),
        ))
    }

    fn from_mut_bytes<'a>(bytes: &mut &'a mut [u8]) -> Result<(&'a mut Self, Self::Metadata)> {
        let bytes_advance = &mut &**bytes;
        let length: &PackedValue<L> = from_bytes(bytes_advance.try_advance(size_of::<L>())?);
        let length = length.0.to_usize()?;

        Ok((
            // Safety: We don't change lifetimes and limit the size of the slice.
            unsafe {
                &mut *ptr::from_raw_parts_mut(
                    bytes
                        .try_advance(size_of::<L>() + size_of::<T>() * length)?
                        .as_mut_ptr()
                        .cast::<()>(),
                    length,
                )
            },
            (),
        ))
    }
}

/// Extension trait for contexts with lists.
pub trait ListContext<T: Align1 + Pod, L: ToFromUsize + Pod> {
    /// The length of the list.
    fn len(&self) -> usize;
    /// Whether the list is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Pushes a value to the end of the list.
    fn push(&mut self, value: T) -> Result<()> {
        self.push_all([value])
    }
    /// Pushes values to the end of the list.
    fn push_all<I>(&mut self, values: I) -> Result<()>
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        self.insert_all(self.len(), values)
    }

    /// Inserts a value at the given index.
    fn insert(&mut self, index: usize, value: T) -> Result<()> {
        self.insert_all(index, [value])
    }
    /// Inserts values at the given index.
    fn insert_all<I>(&mut self, index: usize, values: I) -> Result<()>
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator;

    /// Removes the value at the given index.
    fn remove(&mut self, index: usize) -> Result<()> {
        self.remove_range(index..=index)
    }
    /// Removes the values in the given range.
    fn remove_range(&mut self, range: impl RangeBounds<usize>) -> Result<()>;
}
impl<'a, T: Align1 + Pod + Debug, L: ToFromUsize + Pod> ListContext<T, L>
    for AccountDataMutContext<'a, List<T, L>>
{
    fn len(&self) -> usize {
        self.list.len()
    }

    fn insert_all<I>(&mut self, index: usize, values: I) -> Result<()>
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        let values = values.into_iter();
        let old_length = self.list.len();
        let new_length = old_length + values.len();
        if index > old_length {
            return Err(error!(UtilError::IndexOutOfBounds).into());
        }
        let byte_length = size_of::<L>() + size_of::<T>() * new_length;
        (self.set_length)(byte_length, new_length)?;
        self.data = NonNull::from_raw_parts(self.data.cast(), new_length);
        self.length = PackedValue(L::from_usize(new_length)?);
        if old_length > index {
            // Safety: We don't change lifetimes and limit the size of the slice.
            unsafe {
                sol_memmove(
                    self.list[index + values.len()..].as_mut_ptr().cast::<u8>(),
                    self.list[index..].as_mut_ptr().cast::<u8>(),
                    size_of::<T>() * (self.list.len() - index - 1),
                );
            }
        }
        for (value_index, value) in values.enumerate() {
            self.list[index + value_index] = value;
        }

        Ok(())
    }

    fn remove_range(&mut self, range: impl RangeBounds<usize>) -> Result<()> {
        let start = match range.start_bound() {
            Bound::Included(bound) => *bound,
            Bound::Excluded(bound) => *bound + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(bound) => *bound + 1,
            Bound::Excluded(bound) => *bound,
            Bound::Unbounded => self.list.len(),
        };
        if start > end {
            return Err(error!(UtilError::IndexOutOfBounds).into());
        }
        if end > self.list.len() {
            return Err(error!(UtilError::IndexOutOfBounds).into());
        }

        let old_length = self.list.len();
        let new_length = old_length - (end - start);
        if self.list.len() > end {
            // Safety: We don't change lifetimes and limit the size of the slice.
            unsafe {
                sol_memmove(
                    self.list[start..].as_mut_ptr().cast(),
                    self.list[end..].as_mut_ptr().cast(),
                    size_of::<T>() * (self.list.len() - end),
                );
            }
        }
        let byte_length = size_of::<L>() + size_of::<T>() * new_length;
        (self.set_length)(byte_length, new_length)?;
        self.data = NonNull::from_raw_parts(self.data.cast(), new_length);
        self.length = PackedValue(L::from_usize(new_length)?);

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::testing::TestAccountInfo;
    use crate::versioned_account::account_info::AccountInfoData;
    use crate::versioned_account::list::{List, ListContext};
    use crate::{PackedValue, Result};
    use std::mem::size_of;

    type ListType = List<PackedValue<u64>, u32>;

    #[test]
    fn list_test() -> Result<()> {
        let mut account_data = TestAccountInfo::new(4);
        {
            let account = account_data.account_info();

            {
                // Safety: We are using a valid account.
                let mut account_data_access = unsafe { account.data_access_mut() }?;
                let mut context = account_data_access.context_mut::<ListType>();

                context.push_all([PackedValue(1), PackedValue(2), PackedValue(3)])?;
                context.insert(1, PackedValue(4))?; // [1, 4, 2, 3]
            }

            {
                // Safety: We are using a valid account.
                let account_data_access = unsafe { account.data_access() }?;
                let context = account_data_access.context::<ListType>();

                assert_eq!(
                    **context,
                    [
                        PackedValue(1),
                        PackedValue(4),
                        PackedValue(2),
                        PackedValue(3)
                    ]
                );
            }
        }
        assert_eq!(
            account_data.data_bytes().len(),
            size_of::<u32>() + size_of::<u64>() * 4
        );
        {
            let account = account_data.account_info();
            // Safety: We are using a valid account.
            let mut account_data_access = unsafe { account.data_access_mut() }?;
            let mut context = account_data_access.context_mut::<ListType>();
            context.remove_range(1..=2)?;

            assert_eq!(**context, [PackedValue(1), PackedValue(3)]);
        }

        Ok(())
    }
}
