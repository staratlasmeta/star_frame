use crate::prelude::*;
use bytemuck::{from_bytes, from_bytes_mut};
use std::cell::{Ref, RefMut};
use std::marker::PhantomData;
use std::mem::size_of;

/// A type that can be parsed as remaining data using an arg.
// TODO: remove `'a` when GAT is merged
pub trait RemainingDataWithArg<'a, A> {
    /// The remaining data type
    type Data;
    /// The mutable version of [`RemainingDataWithArg::Data`]
    type DataMut;

    /// Gets the remaining data with an arg.
    fn remaining_data_with_arg(data: Ref<'a, [u8]>, arg: A) -> Result<(Self::Data, Ref<'a, [u8]>)>;
    /// Gets the remaining data with an arg mutably.
    fn remaining_data_mut_with_arg(
        data: RefMut<'a, [u8]>,
        arg: A,
    ) -> Result<(Self::DataMut, RefMut<'a, [u8]>)>;
}
///  A type that can be parsed as remaining data.
pub trait RemainingData<'a>: RemainingDataWithArg<'a, ()> {
    /// Gets the remaining data
    #[inline]
    fn remaining_data(data: Ref<'a, [u8]>) -> Result<(Self::Data, Ref<'a, [u8]>)> {
        Self::remaining_data_with_arg(data, ())
    }
    /// Gets the remaining data mutably
    #[inline]
    fn remaining_data_mut(data: RefMut<'a, [u8]>) -> Result<(Self::DataMut, RefMut<'a, [u8]>)> {
        Self::remaining_data_mut_with_arg(data, ())
    }
}
impl<'a, T> RemainingData<'a> for T where T: RemainingDataWithArg<'a, ()> {}

/// Remaining data has a size that can be calculated with the arg
pub trait RemainingDataSizeWithArg<A>: for<'a> RemainingDataWithArg<'a, A> {
    /// The size of the remaining data based on the arg
    fn data_size_with_arg(arg: A) -> Result<usize>;
}
/// Remaining data has a size that can be calculated
pub trait RemainingDataSize<T>: RemainingDataSizeWithArg<()> {
    /// The size of the remaining data
    #[inline]
    fn data_size() -> Result<usize> {
        Self::data_size_with_arg(())
    }
}
impl<T> RemainingDataSize<T> for T where T: RemainingDataSizeWithArg<()> {}

impl<'a> RemainingDataWithArg<'a, ()> for () {
    type Data = ();
    type DataMut = ();

    fn remaining_data_with_arg(
        data: Ref<'a, [u8]>,
        arg: (),
    ) -> Result<(Self::Data, Ref<'a, [u8]>)> {
        Ok((arg, data))
    }

    fn remaining_data_mut_with_arg(
        data: RefMut<'a, [u8]>,
        arg: (),
    ) -> Result<(Self::DataMut, RefMut<'a, [u8]>)> {
        Ok((arg, data))
    }
}

/// Reads the remaining data as bytes.
#[derive(Debug, Copy, Clone, Default)]
pub struct Bytes;
impl<'a> RemainingDataWithArg<'a, ()> for Bytes {
    type Data = Ref<'a, [u8]>;
    type DataMut = RefMut<'a, [u8]>;

    fn remaining_data_with_arg(
        data: Ref<'a, [u8]>,
        _arg: (),
    ) -> Result<(Self::Data, Ref<'a, [u8]>)> {
        Ok(Ref::map_split(data, |data| (data, &data[data.len()..])))
    }

    fn remaining_data_mut_with_arg(
        data: RefMut<'a, [u8]>,
        _arg: (),
    ) -> Result<(Self::DataMut, RefMut<'a, [u8]>)> {
        Ok(RefMut::map_split(data, |mut data| {
            (data.advance(data.len()), data)
        }))
    }
}

/// Struct is followed by a [`Pod`](bytemuck::Pod) value.
#[derive(Debug)]
pub struct PodValue<R>(PhantomData<fn() -> R>);
impl<R> Clone for PodValue<R> {
    fn clone(&self) -> Self {
        Self(PhantomData)
    }
}
impl<R> Copy for PodValue<R> {}
impl<R> Default for PodValue<R> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
impl<'a, R> RemainingDataWithArg<'a, ()> for PodValue<R>
where
    R: SafeZeroCopy,
{
    type Data = Ref<'a, R>;
    type DataMut = RefMut<'a, R>;

    fn remaining_data_with_arg(
        data: Ref<'a, [u8]>,
        _arg: (),
    ) -> Result<(Self::Data, Ref<'a, [u8]>)> {
        if data.len() < size_of::<R>() {
            Err(error!(UtilError::NotEnoughData))
        } else {
            Ok(Ref::map_split(data, |mut data| {
                (from_bytes(data.advance(size_of::<R>())), data)
            }))
        }
    }

    fn remaining_data_mut_with_arg(
        data: RefMut<'a, [u8]>,
        _arg: (),
    ) -> Result<(Self::DataMut, RefMut<'a, [u8]>)> {
        if data.len() < size_of::<R>() {
            Err(error!(UtilError::NotEnoughData))
        } else {
            Ok(RefMut::map_split(data, |mut data| {
                (from_bytes_mut(data.advance(size_of::<R>())), data)
            }))
        }
    }
}
impl<R> RemainingDataSizeWithArg<()> for PodValue<R>
where
    R: SafeZeroCopy,
{
    fn data_size_with_arg(_arg: ()) -> Result<usize> {
        Ok(size_of::<R>())
    }
}

/// An optional extra value at the end of a struct.
#[derive(Debug)]
pub struct OptionalPod<R>(PhantomData<fn() -> R>);
impl<R> Clone for OptionalPod<R> {
    fn clone(&self) -> Self {
        Self(PhantomData)
    }
}
impl<R> Copy for OptionalPod<R> {}
impl<R> Default for OptionalPod<R> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
impl<'a, R> RemainingDataWithArg<'a, ()> for OptionalPod<R>
where
    R: SafeZeroCopy,
{
    type Data = Option<Ref<'a, R>>;
    type DataMut = Option<RefMut<'a, R>>;

    fn remaining_data_with_arg(
        data: Ref<'a, [u8]>,
        _arg: (),
    ) -> Result<(Self::Data, Ref<'a, [u8]>)> {
        if data.len() < size_of::<R>() {
            Ok((None, data))
        } else {
            let out = Ref::map_split(data, |mut data| {
                (from_bytes(data.advance(size_of::<R>())), data)
            });
            Ok((Some(out.0), out.1))
        }
    }

    fn remaining_data_mut_with_arg(
        data: RefMut<'a, [u8]>,
        _arg: (),
    ) -> Result<(Self::DataMut, RefMut<'a, [u8]>)> {
        if data.len() < size_of::<R>() {
            Ok((None, data))
        } else {
            let out = RefMut::map_split(data, |mut data| {
                (from_bytes_mut(data.advance(size_of::<R>())), data)
            });
            Ok((Some(out.0), out.1))
        }
    }
}
