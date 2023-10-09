use crate::{List, ListCounter, RemainingDataWithArg, Result, SafeZeroCopy};
use bytemuck::Pod;
use common_utils::{DataSize, WrappableAccount, ZeroCopyWrapper};
use solana_program::log::{sol_log, sol_log_data};
use solana_program::msg;
use solana_program::program_memory::sol_memmove;
use static_assertions::assert_eq_align;
use std::cell::{Ref, RefMut};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::{Deref, DerefMut};
use std::result::Result as StdResult;

/// A sorted list of zero copy structs.
#[derive(Debug)]
pub struct ExclusiveMap<K, V, C>(PhantomData<(K, V, C)>);
#[repr(C)]
#[derive(Debug, Copy, Clone, common_utils::bytemuck::Zeroable)]
pub struct ExclusiveMapItem<K, V> {
    key: K,
    value: V,
}

// Safety: when both fields are safe zero copy, they are align 1 and each Pod. The struct is repr(C)
// and thus (crossing off the Pod requirements):
//  - not infallible
//  - allows any bit pattern (both fields are Pod with zero padding between them)
//  - no padding (since both are repr packed and the main struct is repr C)
//  - all fields are Pod (and thus no pointer types or inferior mutability)
//  - is repr(C)
unsafe impl<K, V> Pod for ExclusiveMapItem<K, V>
where
    K: SafeZeroCopy,
    V: SafeZeroCopy,
{
}
impl<K, V> common_utils::DataSize for ExclusiveMapItem<K, V>
where
    K: SafeZeroCopy,
    V: SafeZeroCopy,
{
    const MIN_DATA_SIZE: usize = K::MIN_DATA_SIZE + V::MIN_DATA_SIZE;
}

const _: fn() = || {
    #[repr(packed)]
    struct PackedStruct(u128);
    assert_eq_align!(ExclusiveMapItem<u8, PackedStruct>, u8);
};

// Safety: both fields are safe zero copy => they are align 1 and each Pod. The struct is repr(C)
unsafe impl<K, V> SafeZeroCopy for ExclusiveMapItem<K, V>
where
    K: SafeZeroCopy,
    V: SafeZeroCopy,
{
}

impl<'a, K, V, C> RemainingDataWithArg<'a, ()> for ExclusiveMap<K, V, C>
where
    K: SafeZeroCopy,
    V: SafeZeroCopy,
    C: ListCounter,
    // SortedMapItem<K, V>: SafeZeroCopy,
{
    type Data = Ref<'a, ExclusiveMapItems<K, V>>;
    type DataMut = RefMut<'a, ExclusiveMapItems<K, V>>;

    fn remaining_data_with_arg(
        data: Ref<'a, [u8]>,
        arg: (),
    ) -> Result<(Self::Data, Ref<'a, [u8]>)> {
        let (rem, extra) = List::<ExclusiveMapItem<K, V>, C>::remaining_data_with_arg(data, arg)?;
        Ok((
            Ref::map(rem, |rem| {
                // Safety: Safe because `SortedMapItems` is transparent to `[T]`
                unsafe {
                    &*(rem as *const [ExclusiveMapItem<K, V>] as *const ExclusiveMapItems<K, V>)
                }
            }),
            extra,
        ))
    }

    fn remaining_data_mut_with_arg(
        data: RefMut<'a, [u8]>,
        arg: (),
    ) -> Result<(Self::DataMut, RefMut<'a, [u8]>)> {
        let (rem, extra) =
            List::<ExclusiveMapItem<K, V>, C>::remaining_data_mut_with_arg(data, arg)?;
        Ok((
            RefMut::map(rem, |rem| {
                // Safety: Safe because `SortedMapItems` is transparent to `[T]`
                unsafe {
                    &mut *(rem as *mut [ExclusiveMapItem<K, V>] as *mut ExclusiveMapItems<K, V>)
                }
            }),
            extra,
        ))
    }
}

/// A reference to a sorted list of zero copy structs.
#[repr(transparent)]
#[derive(Debug)]
pub struct ExclusiveMapItems<K, V>([ExclusiveMapItem<K, V>])
where
    V: SafeZeroCopy,
    K: SafeZeroCopy;

impl<K, V> Deref for ExclusiveMapItems<K, V>
where
    K: SafeZeroCopy,
    V: SafeZeroCopy,
{
    type Target = [ExclusiveMapItem<K, V>];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<K, V> DerefMut for ExclusiveMapItems<K, V>
where
    K: SafeZeroCopy,
    V: SafeZeroCopy,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl<K, V> ExclusiveMapItems<K, V>
where
    K: SafeZeroCopy + Ord,
    V: SafeZeroCopy,
{
    /// Returns the position of the item with the given key, if any.
    /// Just does a binary search by key under the hood
    pub fn index_of(&self, key: &K) -> StdResult<usize, usize> {
        self.binary_search_by_key(key, |k| k.key)
    }
    /// Returns true if the map contains the item.
    pub fn contains_key(&self, key: &K) -> bool {
        // false
        self.index_of(key).is_ok()
    }

    /// Gets an item from the map.
    pub fn get(&self, key: &K) -> Option<&V> {
        self.index_of(key).ok().map(|index| &self.0[index].value)
    }

    /// Gets an item mutably from the map.
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.index_of(key).ok().map(|index| &mut self[index].value)
    }
}
impl<'a, 'info, A, K, V, C> ZeroCopyWrapper<'a, 'info, A>
where
    A: WrappableAccount<RemainingData = ExclusiveMap<K, V, C>>,
    K: SafeZeroCopy + Ord,
    V: SafeZeroCopy,
    C: ListCounter,
{
    /// Inserts a key-value pair into the map.
    /// If the map did not have this key present, None is returned.
    /// If the map did have this key present, the value is updated, and the old value is returned.
    ///
    /// Behavior should follow that of the `BTreeMap` insert method.
    pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>> {
        let mut items = self.remaining_mut()?;
        let index = match items.index_of(&key) {
            Ok(index) => {
                msg!("Found key at index: {}", index);
                let old_value = items[index].value;
                items[index].value = value;
                return Ok(Some(old_value));
            }
            Err(index) => index,
        };
        // sol_log("Didn't find key, inserting at: ");
        let index_str = index.to_string();
        sol_log(&index_str);
        // item not in list. Inserting at `index`
        let old_len = items.len();
        let new_len = old_len + 1;
        drop(items);

        let entry_size = ExclusiveMapItem::<K, V>::MIN_DATA_SIZE;

        self.0.as_ref().realloc(
            A::MIN_DATA_SIZE + size_of::<C>() + entry_size * new_len,
            false,
        )?;

        let mut data = self.0.as_ref().data.borrow_mut();
        C::convert_to_le_bytes(new_len, &mut data[A::MIN_DATA_SIZE..][..size_of::<C>()])?;

        let byte_move_start = A::MIN_DATA_SIZE + size_of::<C>() + entry_size * index;
        let byte_move_end = byte_move_start + entry_size;
        assert!(byte_move_end <= data.len());

        // Safety: Safe because the data is valid and the pointers are valid.
        unsafe {
            sol_memmove(
                data[byte_move_end..].as_mut_ptr(),
                data[byte_move_start..].as_mut_ptr(),
                entry_size * (old_len - index),
            );
        }
        drop(data);

        let mut items = self.remaining_mut()?;
        items[index].key = key;
        items[index].value = value;

        Ok(None)
    }

    /// Removes a key from the map, returning the value at the key if the key was previously in the map.
    /// Behavior should follow that of the `BTreeMap` remove method.
    pub fn remove(&mut self, key: &K) -> Result<Option<V>> {
        let items = self.remaining_mut()?;
        let index = match items.index_of(&key) {
            Ok(index) => index,
            Err(_) => return Ok(None),
        };
        let old_value = items[index].value.clone();
        drop(items);

        let items = self.remaining()?;

        let old_len = items.len();
        let new_len = old_len - 1;
        drop(items);

        let entry_size = ExclusiveMapItem::<K, V>::MIN_DATA_SIZE;

        let mut data = self.0.as_ref().data.borrow_mut();
        let bytes_dst = A::MIN_DATA_SIZE + size_of::<C>() + entry_size * index;
        let bytes_src = bytes_dst + entry_size;
        let byte_amount = entry_size * (old_len - index);

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
            A::MIN_DATA_SIZE + size_of::<C>() + entry_size * new_len,
            false,
        )?;

        Ok(Some(old_value))
    }
}
