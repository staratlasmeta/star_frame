use crate::{List, ListCounter, RemainingDataWithArg, Result, SafeZeroCopy};
use common_utils::{WrappableAccount, ZeroCopyWrapper};
use solana_program::program_memory::sol_memmove;
use std::cell::{Ref, RefMut};
use std::convert::identity;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::{Bound, Deref, DerefMut, RangeBounds};

/// A sorted list of zero copy structs.
#[derive(Debug)]
pub struct SortedList<T, C>(PhantomData<(T, C)>);
impl<'a, T, C> RemainingDataWithArg<'a, ()> for SortedList<T, C>
where
    T: SafeZeroCopy,
    C: ListCounter,
{
    type Data = Ref<'a, SortedListItems<T>>;
    type DataMut = RefMut<'a, SortedListItems<T>>;

    fn remaining_data_with_arg(
        data: Ref<'a, [u8]>,
        arg: (),
    ) -> Result<(Self::Data, Ref<'a, [u8]>)> {
        let (rem, extra) = List::<T, C>::remaining_data_with_arg(data, arg)?;
        Ok((
            Ref::map(rem, |rem| {
                // Safety: Safe because `SortedListItems` is transparent to `[T]`
                unsafe { &*(rem as *const [T] as *const SortedListItems<T>) }
            }),
            extra,
        ))
    }

    fn remaining_data_mut_with_arg(
        data: RefMut<'a, [u8]>,
        arg: (),
    ) -> Result<(Self::DataMut, RefMut<'a, [u8]>)> {
        let (rem, extra) = List::<T, C>::remaining_data_mut_with_arg(data, arg)?;
        Ok((
            RefMut::map(rem, |rem| {
                // Safety: Safe because `SortedListItems` is transparent to `[T]`
                unsafe { &mut *(rem as *mut [T] as *mut SortedListItems<T>) }
            }),
            extra,
        ))
    }
}

/// A reference to a sorted list of zero copy structs.
#[repr(transparent)]
#[derive(Debug)]
pub struct SortedListItems<T>([T]);
impl<T> Deref for SortedListItems<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> DerefMut for SortedListItems<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl<T> SortedListItems<T>
where
    T: Ord,
{
    /// Returns true if the sorted list contains the item.
    pub fn contains(&self, item: &T) -> bool {
        self.0.binary_search(item).is_ok()
    }

    /// Gets an item from the sorted list.
    pub fn get_item(&self, item: &T) -> Option<&T> {
        self.0.binary_search(item).ok().map(|index| &self.0[index])
    }

    /// Gets an item mutably from the sorted list.
    pub fn get_item_mut(&mut self, item: &mut T) -> Option<&mut T> {
        self.0
            .binary_search(item)
            .ok()
            .map(|index| &mut self[index])
    }
}

impl<'a, 'info, A, T, C> ZeroCopyWrapper<'a, 'info, A>
where
    A: WrappableAccount<RemainingData = SortedList<T, C>>,
    T: SafeZeroCopy + Ord,
    C: ListCounter,
{
    /// Inserts an item into the sorted list.
    pub fn insert_into_sorted(&mut self, item: T) -> Result<()> {
        let items = self.remaining()?;
        let index = items.binary_search(&item).unwrap_or_else(identity);
        let old_len = items.len();
        let new_len = old_len + 1;
        drop(items);

        self.0.as_ref().realloc(
            A::MIN_DATA_SIZE + size_of::<C>() + size_of::<T>() * new_len,
            false,
        )?;

        let mut data = self.0.as_ref().data.borrow_mut();
        C::convert_to_le_bytes(new_len, &mut data[A::MIN_DATA_SIZE..][..size_of::<C>()])?;

        let byte_move_start = A::MIN_DATA_SIZE + size_of::<C>() + size_of::<T>() * index;
        let byte_move_end = byte_move_start + size_of::<T>();
        assert!(byte_move_end + size_of::<T>() <= data.len());

        // Safety: Safe because the data is valid and the pointers are valid.
        unsafe {
            sol_memmove(
                data[byte_move_end..].as_mut_ptr(),
                data[byte_move_start..].as_mut_ptr(),
                size_of::<T>(),
            );
        }

        let mut items = self.remaining_mut()?;
        items[index] = item;

        Ok(())
    }

    /// Removes a range of items from the sorted list.
    pub fn remove_range(&mut self, range: impl RangeBounds<usize>) -> Result<()> {
        let items = self.remaining()?;
        let start = match range.start_bound() {
            Bound::Included(start) => *start,
            Bound::Excluded(start) => *start + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(end) => *end + 1,
            Bound::Excluded(end) => *end,
            Bound::Unbounded => items.len(),
        };
        assert!(start <= end);
        assert!(end <= items.len());

        let old_len = items.len();
        let new_len = old_len - (end - start);
        drop(items);

        let mut data = self.0.as_ref().data.borrow_mut();
        let bytes_dst = A::MIN_DATA_SIZE + size_of::<C>() + size_of::<T>() * start;
        let bytes_src = bytes_dst + size_of::<T>() * (end - start);
        let byte_amount = size_of::<T>() * (old_len - end);

        // Safety: Safe because the data is valid and the pointers are valid.
        unsafe {
            sol_memmove(
                data[bytes_dst..].as_mut_ptr(),
                data[bytes_src..].as_mut_ptr(),
                byte_amount,
            );
        }

        C::convert_to_le_bytes(new_len, &mut data[A::MIN_DATA_SIZE..][..size_of::<C>()])?;
        drop(data);
        self.0.as_ref().realloc(
            A::MIN_DATA_SIZE + size_of::<C>() + size_of::<T>() * new_len,
            false,
        )?;

        Ok(())
    }
}
