use crate::align1::Align1;
use crate::util::MaybeRef;
use crate::versioned_account::context::{
    AccountDataContext, AccountDataMutContext, AccountDataRefContext,
};
use crate::versioned_account::to_from_usize::ToFromUsize;
use crate::versioned_account::unsized_data::UnsizedData;
use crate::{error, Advance, AdvanceArray, PackedValue, Result, UtilError};
use bytemuck::checked::from_bytes;
use bytemuck::{cast_slice, cast_slice_mut, Pod};
use common_utils::util::MaybeMutRef;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::RangeBounds;
use std::ptr;

#[repr(C)]
#[derive(Align1)]
pub struct UnsizedList<T, L, ML = L> {
    data: PhantomData<(T, ML)>,
    length: PackedValue<L>,
    byte_length: PackedValue<ML>,
    bytes: [u8],
}

unsafe impl<T, L, ML> UnsizedData for UnsizedList<T, L, ML>
where
    T: UnsizedData,
    L: ToFromUsize + Pod,
    ML: ToFromUsize + Pod,
{
    type Metadata = ();

    fn init_data_size() -> usize {
        size_of::<L>()
            + size_of::<ML>()
            + L::zeroed().to_usize().unwrap() * (size_of::<ML>() + T::init_data_size())
    }

    unsafe fn init(bytes: &mut [u8]) -> Result<(&mut Self, Self::Metadata)> {
        assert_eq!(bytes.len(), Self::init_data_size());
        Ok((
            &mut *ptr::from_raw_parts_mut(
                bytes.as_mut_ptr().cast(),
                bytes.len() - size_of::<L>() - size_of::<ML>(),
            ),
            (),
        ))
    }

    fn from_bytes<'a>(bytes: &mut &'a [u8]) -> Result<(&'a Self, Self::Metadata)> {
        let bytes_advance = &mut &**bytes;
        let _length: &PackedValue<L> = from_bytes(bytes_advance.try_advance(size_of::<L>())?);
        let byte_length: &PackedValue<ML> = from_bytes(bytes_advance.try_advance(size_of::<ML>())?);

        Ok((
            unsafe {
                &*ptr::from_raw_parts(bytes.as_ptr().cast(), byte_length.0.to_usize().unwrap())
            },
            (),
        ))
    }

    fn from_mut_bytes<'a>(bytes: &mut &'a mut [u8]) -> Result<(&'a mut Self, Self::Metadata)> {
        let bytes_advance = &mut &**bytes;
        let _length: &PackedValue<L> = from_bytes(bytes_advance.try_advance(size_of::<L>())?);
        let byte_length_bytes: &PackedValue<ML> =
            from_bytes(bytes_advance.try_advance(size_of::<ML>())?);
        let byte_length = byte_length_bytes.0.to_usize()?;

        Ok((
            unsafe { &mut *ptr::from_raw_parts_mut(bytes.as_mut_ptr().cast(), byte_length) },
            (),
        ))
    }
}

pub trait UnsizedListContext<T, L, ML>: AccountDataContext<UnsizedList<T, L, ML>>
where
    T: UnsizedData,
    L: ToFromUsize + Pod,
    ML: ToFromUsize + Pod,
{
    type Iter<'a>: ExactSizeIterator<Item = AccountDataRefContext<'a, T>>
    where
        Self: 'a;

    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn get(&self, index: usize) -> Result<Option<AccountDataRefContext<T>>> {
        Ok(self.get_range(index..=index)?.next())
    }
    fn get_range(&self, range: impl RangeBounds<usize>) -> Result<Self::Iter<'_>>;
    fn iter(&self) -> Self::Iter<'_> {
        self.get_range(..).unwrap()
    }
}
pub trait UnsizedListMutContext<T, L, ML>: UnsizedListContext<T, L, ML>
where
    T: UnsizedData,
    L: ToFromUsize + Pod,
    ML: ToFromUsize + Pod,
{
    type IterMut<'a>: ExactSizeIterator<Item = &'a mut T>
    where
        Self: 'a;

    fn get_mut(&mut self, index: usize) -> Result<Option<AccountDataMutContext<T>>>;
    fn get_range_mut(&mut self, range: impl RangeBounds<usize>) -> Result<Self::IterMut<'_>>;
    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        self.get_range_mut(..).unwrap()
    }
}
impl<T, L, ML, U> UnsizedListContext<T, L, ML> for U
where
    T: UnsizedData,
    L: ToFromUsize + Pod,
    ML: ToFromUsize + Pod,
    U: AccountDataContext<UnsizedList<T, L, ML>>,
{
    type Iter<'a> = UnsizedListIter<'a, T, ML> where Self: 'a,;

    fn len(&self) -> usize {
        self.length.0.to_usize().unwrap()
    }

    fn get_range(&self, range: impl RangeBounds<usize>) -> Result<Self::Iter<'_>> {
        let (mut meta_list, mut bytes) = self.meta_list_and_data()?;
        let start = match range.start_bound() {
            std::ops::Bound::Included(&start) => start,
            std::ops::Bound::Excluded(&start) => start + 1,
            std::ops::Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            std::ops::Bound::Included(&end) => end + 1,
            std::ops::Bound::Excluded(&end) => end,
            std::ops::Bound::Unbounded => meta_list.len(),
        };
        if start > end {
            return Err(error!(UtilError::IndexOutOfBounds));
        }
        let dropped_list = meta_list.try_advance(start)?;
        let bytes_start = dropped_list.iter().map(|v| v.0.to_usize().unwrap()).sum();
        bytes.try_advance(bytes_start)?;
        let meta = meta_list.try_advance(end - start)?;
        Ok(UnsizedListIter {
            item: PhantomData,
            meta,
            bytes,
        })
    }
}
impl<'a, T, L, ML> UnsizedListMutContext<T, L, ML>
    for AccountDataMutContext<'a, UnsizedList<T, L, ML>>
where
    T: UnsizedData,
    L: ToFromUsize + Pod,
    ML: ToFromUsize + Pod,
{
    type IterMut<'b> = UnsizedListIterMut<'b, T, ML> where Self: 'b;

    fn get_mut(&mut self, index: usize) -> Result<Option<AccountDataMutContext<T>>> {
        if self.length.0.to_usize()? <= index {
            return Ok(None);
        }
        unsafe {
            self.try_sub_context_mut(move |args| {
                let (meta_list, mut bytes) = args.data.as_mut().meta_list_and_data_mut()?;
                let index_val = meta_list[index].0.to_usize()?;
                let mut bytes_start = 0;
                for val in meta_list.iter().take(index) {
                    bytes_start += val.0.to_usize()?;
                }
                bytes.try_advance(bytes_start)?;

                let (data, meta) = T::from_mut_bytes(&mut bytes.try_advance(index_val)?)?;

                Ok((
                    data,
                    MaybeMutRef::Owned(meta),
                    Box::new(move |new_t_length, new_t_ptr_meta| {
                        let bytes = args
                            .data
                            .as_ptr()
                            .cast::<u8>()
                            .offset(size_of::<L>() + size_of::<ML>());
                        let length = args.data.as_mut().bytes[index];
                    }),
                ))
            })
            .map(Some)
        }
    }

    fn get_range_mut(&mut self, range: impl RangeBounds<usize>) -> Result<Self::IterMut<'_>> {
        todo!()
    }
}

pub struct UnsizedListIter<'a, T, ML> {
    item: PhantomData<T>,
    meta: &'a [PackedValue<ML>],
    bytes: &'a [u8],
}
impl<'a, T, ML> Iterator for UnsizedListIter<'a, T, ML>
where
    T: UnsizedData,
    ML: ToFromUsize + Pod,
{
    type Item = AccountDataRefContext<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let meta: &[_; 1] = self.meta.try_advance_array().ok()?;
        let bytes = self.bytes.advance(meta[0].0.to_usize().unwrap());
        let (data, meta) = T::from_bytes(&mut &*bytes).unwrap();
        Some(AccountDataRefContext {
            meta: MaybeRef::Owned(meta),
            data,
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.meta.len(), Some(self.meta.len()))
    }
}
impl<'a, T, ML> ExactSizeIterator for UnsizedListIter<'a, T, ML>
where
    T: UnsizedData,
    ML: ToFromUsize + Pod,
{
    fn len(&self) -> usize {
        self.meta.len()
    }
}
impl<T, L, ML> UnsizedList<T, L, ML>
where
    L: ToFromUsize + Copy,
    ML: Pod,
{
    fn meta_list_and_data(&self) -> Result<(&[PackedValue<ML>], &[u8])> {
        let length = self.length.0.to_usize()?;
        let mut bytes = &self.bytes;
        let meta_bytes = bytes.try_advance(length * size_of::<PackedValue<ML>>())?;
        Ok((cast_slice(meta_bytes), bytes))
    }

    fn meta_list_and_data_mut(&mut self) -> Result<(&mut [PackedValue<ML>], &mut [u8])> {
        let length = self.length.0.to_usize()?;
        let mut bytes = &mut self.bytes;
        let meta_bytes = bytes.try_advance(length * size_of::<PackedValue<ML>>())?;
        Ok((cast_slice_mut(meta_bytes), bytes))
    }
}

pub struct UnsizedListIterMut<'a, T, ML> {
    item: PhantomData<T>,
    meta: &'a mut [PackedValue<ML>],
    bytes: &'a mut [u8],
}
impl<'a, T, ML> Iterator for UnsizedListIterMut<'a, T, ML>
where
    T: UnsizedData,
    ML: ToFromUsize + Pod,
{
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        let meta: &mut [_; 1] = self.meta.try_advance_array().ok()?;
        let bytes = self.bytes.advance(meta[0].0.to_usize().unwrap());
        let (data, _) = T::from_mut_bytes(&mut &mut *bytes).unwrap();
        Some(data)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.meta.len(), Some(self.meta.len()))
    }
}
impl<'a, T, ML> ExactSizeIterator for UnsizedListIterMut<'a, T, ML>
where
    T: UnsizedData,
    ML: ToFromUsize + Pod,
{
    fn len(&self) -> usize {
        self.meta.len()
    }
}
