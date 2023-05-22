use crate::prelude::*;
use bytemuck::{cast_slice, cast_slice_mut};
use itertools::Itertools;
use num_traits::Num;
use std::cell::{Ref, RefMut};
use std::cmp::max;
use std::fmt::Debug;
use std::iter::once;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::{Bound, RangeBounds};

/// A type that can count list items.
pub trait ListCounter: Sized + Num + Debug {
    /// Reads this from a set of bytes.
    // TODO: Swap for `AdvanceArray` when const expressions enabled
    fn from_le_bytes<'a>(bytes: &'a mut impl Advance<'a, Element = u8>) -> usize;
    /// Writes a value to a set of bytes.
    fn convert_to_le_bytes(value: usize, target: &mut [u8]) -> Result<()>;
}
macro_rules! impl_list_counter_for_prim {
    ($([$type:ty, $to_type:ident]),* $(,)?) => {
        $(
            impl ListCounter for $type {
                fn from_le_bytes<'a>(bytes: &'a mut impl Advance<'a, Element = u8>) -> usize {
                    let bytes: &[u8] = &*bytes.advance(size_of::<$type>());
                    <$type>::from_le_bytes(
                        bytes
                            .try_into()
                            .unwrap_or_else(|_| panic!("Failed to advance {} bytes", size_of::<$type>())),
                    )
                    .to_usize()
                    .unwrap()
                }

                fn convert_to_le_bytes(value: usize, target: &mut [u8]) -> Result<()> {
                    target.copy_from_slice(&<$type>::to_le_bytes(value
                        .$to_type()
                        .ok_or_else(|| error!(UtilError::NumericOverflow)
                    )?));
                    Ok(())
                }
            }
        )*
    };
}
impl_list_counter_for_prim!([u8, to_u8], [u16, to_u16], [u32, to_u32], [u64, to_u64]);

/// A generic list with count that supports push and pop
pub trait RemainingList:
    for<'a> RemainingDataWithArg<
    'a,
    (),
    Data = Ref<'a, [Self::Item]>,
    DataMut = RefMut<'a, [Self::Item]>,
>
{
    /// The item in the list
    type Item: SafeZeroCopy;
    /// The counter type for the list.
    type ListCounter: ListCounter;
}

/// Account stores the length of a list.
pub trait ListLength {
    /// The length of the contained list.
    fn list_length(&self) -> usize;
    /// Sets the length of the contained list.
    fn set_list_length(&mut self, len: usize) -> Result<()>;
}

/// A generic list that supports push and pop
pub trait RemainingListInputCount:
    for<'a> RemainingDataWithArg<
    'a,
    usize,
    Data = Ref<'a, [Self::Item]>,
    DataMut = RefMut<'a, [Self::Item]>,
>
{
    /// The item in the list
    type Item: SafeZeroCopy;
}

/// An ordered list of [`Pod`](bytemuck::Pod) items.
#[derive(Debug)]
pub struct List<T, C = ()>(PhantomData<fn() -> (T, C)>);
impl<T, C> RemainingList for List<T, C>
where
    T: SafeZeroCopy,
    C: ListCounter,
{
    type Item = T;
    type ListCounter = C;
}
impl<T> RemainingListInputCount for List<T>
where
    T: SafeZeroCopy,
{
    type Item = T;
}
impl<'a, T, C> RemainingDataWithArg<'a, ()> for List<T, C>
where
    T: SafeZeroCopy,
    C: ListCounter,
{
    type Data = Ref<'a, [T]>;
    type DataMut = RefMut<'a, [T]>;

    fn remaining_data_with_arg(
        mut data: Ref<'a, [u8]>,
        _arg: (),
    ) -> Result<(Self::Data, Ref<'a, [u8]>)> {
        let mut length = usize::MAX;
        data = Ref::map(data, |mut data| {
            length = C::from_le_bytes(&mut data);
            data
        });
        if data.len() < length * size_of::<T>() {
            Err(error!(UtilError::NotEnoughData))
        } else {
            Ok(Ref::map_split(data, |mut data| {
                (cast_slice(data.advance(length * size_of::<T>())), data)
            }))
        }
    }

    fn remaining_data_mut_with_arg(
        mut data: RefMut<'a, [u8]>,
        _arg: (),
    ) -> Result<(Self::DataMut, RefMut<'a, [u8]>)> {
        let mut length = usize::MAX;
        data = RefMut::map(data, |mut data| {
            length = C::from_le_bytes(&mut data);
            data
        });
        if data.len() < length * size_of::<T>() {
            Err(error!(UtilError::NotEnoughData))
        } else {
            Ok(RefMut::map_split(data, |mut data| {
                (cast_slice_mut(data.advance(length * size_of::<T>())), data)
            }))
        }
    }
}
impl<'a, T> RemainingDataWithArg<'a, usize> for List<T>
where
    T: SafeZeroCopy,
{
    type Data = Ref<'a, [T]>;
    type DataMut = RefMut<'a, [T]>;

    fn remaining_data_with_arg(
        data: Ref<'a, [u8]>,
        arg: usize,
    ) -> Result<(Self::Data, Ref<'a, [u8]>)> {
        if data.len() < arg * size_of::<T>() {
            Err(error!(UtilError::NotEnoughData))
        } else {
            Ok(Ref::map_split(data, |mut data| {
                (cast_slice(data.advance(arg * size_of::<T>())), data)
            }))
        }
    }

    fn remaining_data_mut_with_arg(
        data: RefMut<'a, [u8]>,
        arg: usize,
    ) -> Result<(Self::DataMut, RefMut<'a, [u8]>)> {
        if data.len() < arg * size_of::<T>() {
            Err(error!(UtilError::NotEnoughData))
        } else {
            Ok(RefMut::map_split(data, |mut data| {
                (cast_slice_mut(data.advance(arg * size_of::<T>())), data)
            }))
        }
    }
}

/// An unordered list of [`Pod`](bytemuck::Pod) items.
#[derive(Debug)]
pub struct UnorderedList<T, C = ()>(PhantomData<fn() -> (T, C)>);
impl<T, C> RemainingList for UnorderedList<T, C>
where
    T: SafeZeroCopy,
    C: ListCounter,
{
    type Item = T;
    type ListCounter = C;
}
impl<T> RemainingListInputCount for UnorderedList<T>
where
    T: SafeZeroCopy,
{
    type Item = T;
}
impl<'a, T, C> RemainingDataWithArg<'a, ()> for UnorderedList<T, C>
where
    T: SafeZeroCopy,
    C: ListCounter,
{
    type Data = Ref<'a, [T]>;
    type DataMut = RefMut<'a, [T]>;

    fn remaining_data_with_arg(
        data: Ref<'a, [u8]>,
        arg: (),
    ) -> Result<(Self::Data, Ref<'a, [u8]>)> {
        List::<T, C>::remaining_data_with_arg(data, arg)
    }

    fn remaining_data_mut_with_arg(
        data: RefMut<'a, [u8]>,
        arg: (),
    ) -> Result<(Self::DataMut, RefMut<'a, [u8]>)> {
        List::<T, C>::remaining_data_mut_with_arg(data, arg)
    }
}
impl<'a, T, A> RemainingDataWithArg<'a, A> for UnorderedList<T>
where
    List<T>: RemainingDataWithArg<'a, A>,
{
    type Data = <List<T> as RemainingDataWithArg<'a, A>>::Data;
    type DataMut = <List<T> as RemainingDataWithArg<'a, A>>::DataMut;

    fn remaining_data_with_arg(data: Ref<'a, [u8]>, arg: A) -> Result<(Self::Data, Ref<'a, [u8]>)> {
        List::<T>::remaining_data_with_arg(data, arg)
    }

    fn remaining_data_mut_with_arg(
        data: RefMut<'a, [u8]>,
        arg: A,
    ) -> Result<(Self::DataMut, RefMut<'a, [u8]>)> {
        List::<T>::remaining_data_mut_with_arg(data, arg)
    }
}

/// Trait for list functions.
/// `SPLITTER` allows implementing this trait for different types that may have some overlap.
pub trait WrapperList<const SPLITTER: bool> {
    /// The item for the list
    type Item;

    /// Pushes an item to the list.
    /// [`normalize_rent`](crate::normalize_rent()) must be called after.
    fn push(&mut self, item: Self::Item) -> Result<()> {
        self.push_all(once(item))
    }
    /// Pushes all items from an iterator to the list.
    /// [`normalize_rent`](crate::normalize_rent()) must be called after.
    fn push_all(&mut self, items: impl ExactSizeIterator<Item = Self::Item>) -> Result<()> {
        self.try_push_all(items.map(Ok))
    }
    /// Tries to push all items from an iterator to the list.
    /// [`normalize_rent`](crate::normalize_rent()) must be called after.
    fn try_push_all(
        &mut self,
        items: impl ExactSizeIterator<Item = Result<Self::Item>>,
    ) -> Result<()>;
    /// Pops an item off the back of the list.
    /// [`normalize_rent`](crate::normalize_rent()) must be called after.
    fn pop(&mut self) -> Result<()> {
        self.pop_count(1)
    }
    /// Pops `count` items off the back of the list.
    /// [`normalize_rent`](crate::normalize_rent()) must be called after.
    fn pop_count(&mut self, count: usize) -> Result<()>;
}

impl<'a, 'info, A> WrapperList<false> for ZeroCopyWrapper<'a, 'info, A>
where
    A: WrappableAccount,
    A::RemainingData: RemainingList,
{
    type Item = <A::RemainingData as RemainingList>::Item;

    fn try_push_all(
        &mut self,
        items: impl ExactSizeIterator<Item = Result<<A::RemainingData as RemainingList>::Item>>,
    ) -> Result<()> {
        let mut data = self.data_and_extra_mut()?.1;
        let old_length = if data.len() == 0 {
            drop(data);
            let new_data_size = A::MIN_DATA_SIZE
                + size_of::<<A::RemainingData as RemainingList>::ListCounter>()
                + items.len() * size_of::<<A::RemainingData as RemainingList>::Item>();

            self.0.as_ref().realloc(new_data_size, false)?;
            <A::RemainingData as RemainingList>::ListCounter::convert_to_le_bytes(
                items.len(),
                (&mut &mut *self.data_and_extra_mut()?.1)
                    .advance(size_of::<<A::RemainingData as RemainingList>::ListCounter>()),
            )?;

            0
        } else if data.len() < size_of::<<A::RemainingData as RemainingList>::ListCounter>() {
            msg!("List did not have enough data to read length");
            return Err(error!(UtilError::NotEnoughData));
        } else {
            let mut data_ref = &mut *data;
            let length_bytes =
                data_ref.advance(size_of::<<A::RemainingData as RemainingList>::ListCounter>());
            let length = <A::RemainingData as RemainingList>::ListCounter::from_le_bytes(
                &mut &*length_bytes,
            );
            let new_length = length + items.len();
            <A::RemainingData as RemainingList>::ListCounter::convert_to_le_bytes(
                new_length,
                length_bytes,
            )?;

            drop(data);
            let new_data_size = A::MIN_DATA_SIZE
                + size_of::<<A::RemainingData as RemainingList>::ListCounter>()
                + new_length * size_of::<<A::RemainingData as RemainingList>::Item>();

            self.0.as_ref().realloc(new_data_size, false)?;

            length
        };

        let mut list: RefMut<[<A::RemainingData as RemainingList>::Item]> = self.remaining_mut()?;
        for (list, item) in list.iter_mut().skip(old_length).zip_eq(items) {
            *list = item?;
        }
        Ok(())
    }

    fn pop_count(&mut self, count: usize) -> Result<()> {
        let mut data = self.data_and_extra_mut()?.1;
        let mut data_ref = &mut *data;
        let length_bytes =
            data_ref.advance(size_of::<<A::RemainingData as RemainingList>::ListCounter>());
        let length =
            <A::RemainingData as RemainingList>::ListCounter::from_le_bytes(&mut &*length_bytes);
        require_gte!(length, count, UtilError::TooManyPopped);
        let new_length = length - count;
        <A::RemainingData as RemainingList>::ListCounter::convert_to_le_bytes(
            new_length,
            length_bytes,
        )?;
        drop(data);

        let new_data_size = A::MIN_DATA_SIZE
            + size_of::<<A::RemainingData as RemainingList>::ListCounter>()
            + new_length * size_of::<<A::RemainingData as RemainingList>::Item>();
        self.0.as_ref().realloc(new_data_size, false)?;
        Ok(())
    }
}
impl<'a, 'info, A> ZeroCopyWrapper<'a, 'info, A>
where
    A: WrappableAccount<usize> + ListLength,
    A::RemainingData: RemainingListInputCount,
{
    /// Gets the list
    pub fn list(&self) -> Result<<A::RemainingData as RemainingDataWithArg<'_, usize>>::Data> {
        Ok(self.data_and_list()?.1)
    }

    /// Gets the account data and the list.
    #[allow(clippy::type_complexity)]
    pub fn data_and_list(
        &self,
    ) -> Result<(
        Ref<'_, A>,
        <A::RemainingData as RemainingDataWithArg<'_, usize>>::Data,
        Ref<'_, [u8]>,
    )> {
        let length = self.data()?.list_length();
        self.data_and_remaining_with_arg(length)
    }

    /// Gets the list mutably.
    pub fn list_mut(
        &mut self,
    ) -> Result<<A::RemainingData as RemainingDataWithArg<'_, usize>>::DataMut> {
        Ok(self.data_and_list_mut()?.1)
    }

    /// Gets the account data and the list mutably.
    #[allow(clippy::type_complexity)]
    pub fn data_and_list_mut(
        &mut self,
    ) -> Result<(
        RefMut<'_, A>,
        <A::RemainingData as RemainingDataWithArg<'_, usize>>::DataMut,
        RefMut<'_, [u8]>,
    )> {
        let length = self.data_mut()?.list_length();
        self.data_and_remaining_mut_with_arg(length)
    }
}
impl<'a, 'info, A> WrapperList<true> for ZeroCopyWrapper<'a, 'info, A>
where
    A: WrappableAccount<usize> + ListLength,
    A::RemainingData: RemainingListInputCount,
{
    type Item = <A::RemainingData as RemainingListInputCount>::Item;

    fn try_push_all(
        &mut self,
        items: impl ExactSizeIterator<
            Item = Result<<A::RemainingData as RemainingListInputCount>::Item>,
        >,
    ) -> Result<()> {
        let mut data = self.data_mut()?;
        let current_length = data.list_length();
        let new_length = current_length.saturating_add(items.len());
        data.set_list_length(new_length)?;
        drop(data);
        let new_data_size = A::MIN_DATA_SIZE
            + size_of::<<A::RemainingData as RemainingListInputCount>::Item>() * new_length;
        self.0.as_ref().realloc(new_data_size, false)?;
        let mut list = self.remaining_mut_with_arg(new_length)?;
        for (list, item) in list.iter_mut().skip(current_length).zip_eq(items) {
            *list = item?;
        }
        Ok(())
    }

    fn pop_count(&mut self, count: usize) -> Result<()> {
        let mut current_data = self.data_mut()?;
        let current_length = current_data.list_length();
        let new_length = current_length
            .checked_sub(count)
            .ok_or_else(|| error!(UtilError::TooManyPopped))?;
        current_data.set_list_length(new_length)?;
        drop(current_data);
        let new_data_size = A::MIN_DATA_SIZE
            + size_of::<<A::RemainingData as RemainingListInputCount>::Item>() * new_length;
        self.0.as_ref().realloc(new_data_size, false)?;
        Ok(())
    }
}

/// Trait for unordered list functions.
/// `SPLITTER` allows implementing this trait for different types that may have some overlap.
pub trait WrapperListUnorderedList<const SPLITTER: bool>: WrapperList<SPLITTER> {
    /// Removes a specific index from the list.
    /// This ensures the list stays packed.
    /// [`normalize_rent`](crate::normalize_rent()) must be called after.
    fn remove_index(&mut self, index: usize) -> Result<()> {
        self.remove_range(index..=index)
    }

    /// Removes a range of items moving items at the end into the empty slot (if able).
    /// This ensures the list stays packed.
    /// [`normalize_rent`](crate::normalize_rent()) must be called after.
    fn remove_range(&mut self, range: impl RangeBounds<usize>) -> Result<()>;
}

impl<'a, 'info, A, T, C> WrapperListUnorderedList<false> for ZeroCopyWrapper<'a, 'info, A>
where
    A: WrappableAccount<(), RemainingData = UnorderedList<T, C>>,
    T: SafeZeroCopy,
    C: ListCounter,
{
    fn remove_range(&mut self, range: impl RangeBounds<usize>) -> Result<()> {
        let lower_index = match range.start_bound() {
            Bound::Included(index) => *index,
            Bound::Excluded(index) => index.saturating_add(1),
            Bound::Unbounded => 0,
        };
        let upper_bound = match range.end_bound() {
            Bound::Included(index) => index.saturating_add(1),
            Bound::Excluded(index) => *index,
            Bound::Unbounded => self.remaining()?.len(),
        };

        let mut list = self.remaining_mut()?;
        let list_length = list.len();
        assert!(upper_bound <= list.len());
        assert!(lower_index < upper_bound);
        let num_removed = upper_bound - lower_index;
        list.copy_within(max(list_length - num_removed, upper_bound).., lower_index);
        drop(list);

        let mut data = self.data_and_extra_mut()?.1;
        let mut data_ref = &mut *data;
        let length_bytes =
            data_ref.advance(size_of::<<A::RemainingData as RemainingList>::ListCounter>());
        let new_length = list_length - num_removed;
        C::convert_to_le_bytes(new_length, length_bytes)?;
        drop(data);

        let new_data_size = A::MIN_DATA_SIZE
            + size_of::<<A::RemainingData as RemainingList>::ListCounter>()
            + new_length * size_of::<<A::RemainingData as RemainingList>::Item>();
        self.0.as_ref().realloc(new_data_size, false)?;
        Ok(())
    }
}

impl<'a, 'info, A, T> WrapperListUnorderedList<true> for ZeroCopyWrapper<'a, 'info, A>
where
    A: WrappableAccount<usize, RemainingData = UnorderedList<T>> + ListLength,
    T: SafeZeroCopy + Debug,
{
    fn remove_range(&mut self, range: impl RangeBounds<usize>) -> Result<()> {
        let (mut current_data, mut list, _) = self.data_and_list_mut()?;
        let current_length = current_data.list_length();
        assert_eq!(current_length, list.len());
        let lower_index = match range.start_bound() {
            Bound::Included(index) => *index,
            Bound::Excluded(index) => index.saturating_add(1),
            Bound::Unbounded => 0,
        };
        let upper_bound = match range.end_bound() {
            Bound::Included(index) => index.saturating_add(1),
            Bound::Excluded(index) => *index,
            Bound::Unbounded => current_length,
        };
        assert!(upper_bound <= list.len());
        assert!(lower_index < upper_bound);
        let num_removed = upper_bound - lower_index;
        list.copy_within(
            max(current_length - num_removed, upper_bound)..,
            lower_index,
        );
        let new_length = current_length.checked_sub(num_removed).unwrap();
        current_data.set_list_length(new_length)?;
        drop(current_data);
        drop(list);
        let new_data_size = A::MIN_DATA_SIZE + size_of::<T>() * new_length;
        self.0.as_ref().realloc(new_data_size, false)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use array_init::array_init;
    use bytemuck::{Pod, Zeroable};
    use common_utils::prelude::*;
    use rand::{thread_rng, Rng};
    use rayon::prelude::*;
    use solana_program::entrypoint::MAX_PERMITTED_DATA_INCREASE;
    use solana_program::system_instruction::MAX_PERMITTED_DATA_LENGTH;
    use std::cell::RefCell;
    use std::cmp::min;
    use std::collections::HashSet;
    use std::mem::size_of;
    use std::rc::Rc;

    #[derive(Zeroable, Pod, Copy, Clone, Debug, Eq, PartialEq)]
    #[repr(C)]
    struct AccountInfoDataHeader {
        /// Should be u8::MAX
        dup_info: u8,
        is_signer: u8,
        is_writable: u8,
        executable: u8,
        original_data_len: u32,
        key: Pubkey,
        owner: Pubkey,
        lamports: u64,
        data_len: u64,
    }

    #[safe_zero_copy_account]
    #[account(zero_copy)]
    #[derive(Eq, PartialEq)]
    struct ListAccount {
        version: u8,
        data1: u64,
        key: Pubkey,
        list_length: u32,
    }
    #[safe_zero_copy]
    #[zero_copy]
    #[derive(Eq, PartialEq, Hash)]
    struct ListItem {
        val1: i64,
        key: Pubkey,
    }
    impl WrappableAccount<usize> for ListAccount {
        type RemainingData = UnorderedList<ListItem>;
    }
    impl ListLength for ListAccount {
        fn list_length(&self) -> usize {
            self.list_length as usize
        }
        fn set_list_length(&mut self, length: usize) -> Result<()> {
            self.list_length = length
                .to_u32()
                .ok_or_else(|| error!(UtilError::NumericOverflow))?;
            Ok(())
        }
    }

    fn setup_account<T: SafeZeroCopyAccount>(
        data_vec: &mut Vec<u8>,
        is_signer: bool,
        is_writable: bool,
        executable: bool,
        key: Pubkey,
        lamports: Option<u64>,
        extra_bytes: usize,
    ) -> AccountLoader<T> {
        data_vec.fill(0);
        let data_len = min(
            size_of::<AccountInfoDataHeader>() + extra_bytes + T::MIN_DATA_SIZE,
            MAX_PERMITTED_DATA_LENGTH.to_usize().unwrap(),
        );
        data_vec.resize(data_len + MAX_PERMITTED_DATA_INCREASE, 0);
        let mut slice = data_vec.as_mut_slice();
        let header: &mut AccountInfoDataHeader =
            from_bytes_mut(slice.advance(size_of::<AccountInfoDataHeader>()));
        *header = AccountInfoDataHeader {
            dup_info: u8::MAX,
            is_signer: u8::from(is_signer),
            is_writable: u8::from(is_writable),
            executable: u8::from(executable),
            original_data_len: data_len.to_u32().unwrap(),
            key,
            owner: T::owner(),
            lamports: lamports.unwrap_or_else(|| Rent::default().minimum_balance(data_len)),
            data_len: data_len as u64,
        };
        let account_info = AccountInfo {
            key: &header.key,
            is_signer,
            is_writable,
            lamports: Rc::new(RefCell::new(&mut header.lamports)),
            data: Rc::new(RefCell::new(slice)),
            owner: &header.owner,
            executable,
            rent_epoch: 0,
        };
        AccountLoader::try_from_unchecked(&T::owner(), &account_info).unwrap()
    }

    #[test]
    fn remove_test() {
        const NUM_ATTEMPTS: u64 = 1 << 10;
        const LIST_SIZE: usize = 1 << 10;
        (0..NUM_ATTEMPTS)
            .into_par_iter()
            .map(|_| {
                let mut data = Vec::new();
                let account = setup_account(
                    &mut data,
                    true,
                    true,
                    false,
                    Pubkey::new_unique(),
                    None,
                    LIST_SIZE * size_of::<ListItem>(),
                );
                let mut wrapper = ZeroCopyWrapper::from(&account);
                *wrapper.init()? = ListAccount {
                    version: 0,
                    data1: 100,
                    key: Pubkey::new_unique(),
                    list_length: 0,
                };
                assert_eq!(&*wrapper.list()?, &[]);
                let new_items: [_; LIST_SIZE] = array_init(|index| ListItem {
                    val1: index as i64,
                    key: Pubkey::new_unique(),
                });
                wrapper.push_all(new_items.into_iter())?;
                assert_eq!(&*wrapper.list()?, &new_items);
                let mut rng = thread_rng();
                let lower = rng.gen_range(0..new_items.len());
                let upper = rng.gen_range(lower + 1..=new_items.len());
                wrapper.remove_range(lower..upper)?;
                assert_eq!(
                    wrapper.list()?.iter().copied().collect::<HashSet<_>>(),
                    new_items[..lower]
                        .iter()
                        .chain(new_items[upper..].iter())
                        .copied()
                        .collect::<HashSet<_>>()
                );
                Ok(())
            })
            .collect::<Result<Vec<_>>>()
            .unwrap();
    }
}
