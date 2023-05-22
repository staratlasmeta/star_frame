use crate::prelude::*;
use bytemuck::{cast_slice, cast_slice_mut};
use common_utils::{ListLength, WrappableAccount};
use std::cell::{Ref, RefMut};
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::{Deref, DerefMut, Index, IndexMut};

/// A sorted list of items. Is a min-heap.
#[derive(Debug)]
pub struct BinaryHeap<T>(PhantomData<T>)
where
    T: SafeZeroCopy;
impl<T> Default for BinaryHeap<T>
where
    T: SafeZeroCopy,
{
    fn default() -> Self {
        BinaryHeap(PhantomData)
    }
}

/// The data of a binary heap
#[derive(Debug)]
pub struct BinaryHeapData<R>
where
    R: Deref,
{
    data: R,
}
impl<R> BinaryHeapData<R>
where
    R: Deref,
{
    /// Gets the length of the binary heap.
    pub fn len<T>(&self) -> usize
    where
        R: Deref<Target = [T]>,
    {
        self.data.len()
    }

    /// Tells if the binary heap is empty.
    pub fn is_empty<T>(&self) -> bool
    where
        R: Deref<Target = [T]>,
    {
        self.data.is_empty()
    }

    /// Gets minimum value in the heap.
    pub fn min_val<T>(&self) -> Option<&T>
    where
        R: Deref<Target = [T]>,
    {
        if self.is_empty() {
            None
        } else {
            Some(&self.data[0])
        }
    }

    /// Gets minimum value in the heap mutably.
    pub fn min_val_mut<T>(&mut self) -> Option<&mut T>
    where
        R: DerefMut<Target = [T]>,
    {
        if self.is_empty() {
            None
        } else {
            Some(&mut self.data[0])
        }
    }

    fn fix_heap<T>(&mut self, index: usize)
    where
        R: DerefMut<Target = [T]>,
        T: SafeZeroCopy + Ord,
    {
        let l = left_child(index);
        let r = right_child(index);
        let mut smallest = index;
        if l < self.len() && self.data[l] < self.data[smallest] {
            smallest = l;
        }
        if r < self.len() && self.data[r] < self.data[smallest] {
            smallest = r;
        }
        if smallest != index {
            self.data.swap(index, smallest);
            self.fix_heap(smallest);
        }
    }
}
impl<R, I> Index<I> for BinaryHeapData<R>
where
    R: Deref,
    R::Target: Index<I>,
{
    type Output = <R::Target as Index<I>>::Output;
    fn index(&self, index: I) -> &Self::Output {
        &self.data[index]
    }
}
impl<R, I> IndexMut<I> for BinaryHeapData<R>
where
    R: DerefMut,
    R::Target: IndexMut<I>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.data[index]
    }
}
impl<'a, T> RemainingDataWithArg<'a, usize> for BinaryHeap<T>
where
    T: SafeZeroCopy,
{
    type Data = BinaryHeapData<Ref<'a, [T]>>;
    type DataMut = BinaryHeapData<RefMut<'a, [T]>>;

    fn remaining_data_with_arg(
        data: Ref<'a, [u8]>,
        arg: usize,
    ) -> Result<(Self::Data, Ref<'a, [u8]>)> {
        if data.len() < arg * size_of::<T>() {
            Err(error!(UtilError::NotEnoughData))
        } else {
            let (data, remaining) = Ref::map_split(data, |mut data| {
                (cast_slice(data.advance(arg * size_of::<T>())), data)
            });
            Ok((BinaryHeapData { data }, remaining))
        }
    }

    fn remaining_data_mut_with_arg(
        data: RefMut<'a, [u8]>,
        arg: usize,
    ) -> Result<(Self::DataMut, RefMut<'a, [u8]>)> {
        if data.len() < arg * size_of::<T>() {
            Err(error!(UtilError::NotEnoughData))
        } else {
            let (data, remaining) = RefMut::map_split(data, |mut data| {
                (cast_slice_mut(data.advance(arg * size_of::<T>())), data)
            });
            Ok((BinaryHeapData { data }, remaining))
        }
    }
}

const fn parent(index: usize) -> usize {
    (index - 1) / 2
}
const fn left_child(index: usize) -> usize {
    2 * index + 1
}
const fn right_child(index: usize) -> usize {
    2 * index + 2
}

impl<'a, 'info, A, R> ZeroCopyWrapper<'a, 'info, A>
where
    A: WrappableAccount<usize, RemainingData = BinaryHeap<R>> + ListLength,
    R: SafeZeroCopy + Ord,
{
    /// The length of the heap
    pub fn heap_len(&self) -> Result<usize> {
        Ok(self.data()?.list_length())
    }

    /// The heap itself
    pub fn heap(&self) -> Result<BinaryHeapData<Ref<[R]>>> {
        self.remaining_with_arg(self.heap_len()?)
    }

    /// The heap itself mutably
    pub fn heap_mut(&mut self) -> Result<BinaryHeapData<RefMut<[R]>>> {
        self.remaining_mut_with_arg(self.heap_len()?)
    }

    /// Inserts a new value into the heap
    /// [`normalize_rent`](crate::normalize_rent()) must be called after.
    pub fn insert_val(&mut self, val: R) -> Result<()> {
        let mut data = self.data_mut()?;
        let length = data.list_length();
        let new_length = length + 1;
        data.set_list_length(new_length)?;
        let new_data_len = A::MIN_DATA_SIZE + new_length * size_of::<R>();
        drop(data);
        self.as_ref().realloc(new_data_len, false)?;
        let mut remaining_data: BinaryHeapData<RefMut<[R]>> =
            self.remaining_mut_with_arg(new_length)?;
        let heap = &mut remaining_data.data;
        let mut i = length;
        heap[i] = val;
        while i > 0 && heap[parent(i)] > heap[i] {
            heap.swap(i, parent(i));
            i = parent(i);
        }
        Ok(())
    }

    /// Removes the smallest value from the heap
    /// [`normalize_rent`](crate::normalize_rent()) must be called after.
    pub fn remove_smallest(&mut self) -> Result<Option<R>> {
        self.remove_heap_index(0)
    }

    /// Removes the value at the given index
    /// [`normalize_rent`](crate::normalize_rent()) must be called after.
    pub fn remove_heap_index(&mut self, index: usize) -> Result<Option<R>> {
        let mut data = self.data_mut()?;
        let length = data.list_length();
        if length <= index {
            return Ok(None);
        }
        let new_length = length - 1;
        data.set_list_length(new_length)?;
        drop(data);
        let mut remaining_data: BinaryHeapData<RefMut<[R]>> =
            self.remaining_mut_with_arg(length)?;
        let heap = &mut remaining_data.data;
        let out = heap[index];
        heap[index] = heap[new_length];
        drop(remaining_data);
        let new_data_len = A::MIN_DATA_SIZE + new_length * size_of::<R>();
        self.as_ref().realloc(new_data_len, false)?;
        let mut remaining_data: BinaryHeapData<RefMut<[R]>> =
            self.remaining_mut_with_arg(new_length)?;
        remaining_data.fix_heap(index);

        Ok(Some(out))
    }
}

#[cfg(test)]
mod test {
    use crate::{PackedValue, ZeroCopyWrapper};
    use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;
    use anchor_lang::solana_program::pubkey;
    use anchor_lang::Discriminator;
    use common_proc::safe_zero_copy_account;
    use common_utils::binary_heap::BinaryHeap;
    use common_utils::prelude::*;
    use common_utils::{ListLength, WrappableAccount};
    use num_traits::ToPrimitive;
    use std::cell::RefCell;
    use std::error::Error;
    use std::mem::size_of;
    use std::rc::Rc;

    #[safe_zero_copy_account]
    #[account(zero_copy)]
    struct Header {
        value: u8,
        length: u64,
    }
    impl WrappableAccount<usize> for Header {
        type RemainingData = BinaryHeap<PackedValue<u64>>;
    }
    impl ListLength for Header {
        fn list_length(&self) -> usize {
            { self.length }.to_usize().unwrap()
        }

        fn set_list_length(&mut self, len: usize) -> Result<()> {
            self.length = len as u64;
            Ok(())
        }
    }

    #[test]
    fn heap_test() -> std::result::Result<(), Box<dyn Error>> {
        let mut lamports = LAMPORTS_PER_SOL;
        let mut data = vec![0; 8 + 8 + 1 + 10_000];
        data[..8].copy_from_slice(&9u64.to_le_bytes());
        data[8..][..8].copy_from_slice(&Header::discriminator());
        let account_info = AccountInfo {
            key: &pubkey!("Fnfn1pmcwViyjzUQ3G6J6qyHxgQ37ndevjPSnJHzQUqp"),
            is_signer: false,
            is_writable: true,
            lamports: Rc::new(RefCell::new(&mut lamports)),
            data: Rc::new(RefCell::new(&mut data[8..][..8 + 1 + 8])),
            owner: &crate::ID,
            executable: false,
            rent_epoch: 0,
        };
        let start_size = account_info.data.borrow().len();
        let loader = AccountLoader::<Header>::try_from(&account_info)?;
        let mut wrapper = ZeroCopyWrapper::<Header>::try_from(&loader)?;
        wrapper.insert_val(PackedValue(100))?;
        wrapper.insert_val(PackedValue(24))?;
        wrapper.insert_val(PackedValue(12))?;
        wrapper.insert_val(PackedValue(10000))?;
        wrapper.insert_val(PackedValue(1))?;
        wrapper.insert_val(PackedValue(1))?;
        assert_eq!(
            account_info.data.borrow().len(),
            start_size + 6 * size_of::<u64>()
        );
        let mut heap = wrapper.heap_mut()?;
        println!("{heap:?}");
        assert_eq!(heap.len(), 6);
        assert!(!heap.is_empty());
        assert_eq!(heap.min_val(), Some(&PackedValue(1)));
        assert_eq!(heap.min_val_mut(), Some(&mut PackedValue(1)));
        drop(heap);

        let account_info2 = account_info.clone();
        let loader2 = AccountLoader::<Header>::try_from(&account_info2)?;
        let mut wrapper2 = ZeroCopyWrapper::<Header>::try_from(&loader2)?;
        assert_eq!(wrapper2.remove_heap_index(1), Ok(Some(PackedValue(12))));
        println!("1 {:?}", wrapper2.heap()?.data.iter().map(|val| val.0));
        assert_eq!(wrapper2.remove_smallest(), Ok(Some(PackedValue(1))));
        println!("2 {:?}", wrapper2.heap()?.data.iter().map(|val| val.0));
        assert_eq!(wrapper2.remove_smallest(), Ok(Some(PackedValue(1))));
        println!("3 {:?}", wrapper2.heap()?.data.iter().map(|val| val.0));
        assert_eq!(wrapper2.remove_smallest(), Ok(Some(PackedValue(24))));
        println!("4 {:?}", wrapper2.heap()?.data.iter().map(|val| val.0));
        assert_eq!(wrapper2.remove_smallest(), Ok(Some(PackedValue(100))));
        println!("5 {:?}", wrapper2.heap()?.data.iter().map(|val| val.0));
        assert_eq!(wrapper2.remove_smallest(), Ok(Some(PackedValue(10000))));
        println!("6 {:?}", wrapper2.heap()?.data.iter().map(|val| val.0));
        assert_eq!(wrapper2.remove_smallest(), Ok(None));

        assert_eq!(account_info.data.borrow().len(), start_size);
        let mut heap = wrapper.heap_mut()?;
        println!("{heap:?}");
        assert_eq!(heap.len(), 0);
        assert!(heap.is_empty());
        assert_eq!(heap.min_val(), None);
        assert_eq!(heap.min_val_mut(), None);

        Ok(())
    }
}
