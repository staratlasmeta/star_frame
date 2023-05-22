use crate::prelude::*;
use std::cell::{Ref, RefMut};
use std::marker::PhantomData;

/// Chains remaining data together.
#[derive(Debug)]
pub struct ChainedData<R1, R2>(PhantomData<(R1, R2)>);
impl<'a, R1, R2> RemainingDataWithArg<'a, ()> for ChainedData<R1, R2>
where
    R1: RemainingDataWithArg<'a, ()>,
    R2: RemainingDataWithArg<'a, ()>,
{
    type Data = <Self as RemainingDataWithArg<'a, ((), ())>>::Data;
    type DataMut = <Self as RemainingDataWithArg<'a, ((), ())>>::DataMut;

    fn remaining_data_with_arg(
        data: Ref<'a, [u8]>,
        arg: (),
    ) -> Result<(Self::Data, Ref<'a, [u8]>)> {
        <Self as RemainingDataWithArg<'a, ((), ())>>::remaining_data_with_arg(data, (arg, arg))
    }

    fn remaining_data_mut_with_arg(
        data: RefMut<'a, [u8]>,
        arg: (),
    ) -> Result<(Self::DataMut, RefMut<'a, [u8]>)> {
        <Self as RemainingDataWithArg<'a, ((), ())>>::remaining_data_mut_with_arg(data, (arg, arg))
    }
}
impl<'a, R1, R2, A1, A2> RemainingDataWithArg<'a, (A1, A2)> for ChainedData<R1, R2>
where
    R1: RemainingDataWithArg<'a, A1>,
    R2: RemainingDataWithArg<'a, A2>,
{
    type Data = (R1::Data, R2::Data);
    type DataMut = (R1::DataMut, R2::DataMut);

    fn remaining_data_with_arg(
        data: Ref<'a, [u8]>,
        arg: (A1, A2),
    ) -> Result<(Self::Data, Ref<'a, [u8]>)> {
        let (arg1, arg2) = arg;
        let (r1, data) = R1::remaining_data_with_arg(data, arg1)?;
        let (r2, data) = R2::remaining_data_with_arg(data, arg2)?;
        Ok(((r1, r2), data))
    }

    fn remaining_data_mut_with_arg(
        data: RefMut<'a, [u8]>,
        arg: (A1, A2),
    ) -> Result<(Self::DataMut, RefMut<'a, [u8]>)> {
        let (arg1, arg2) = arg;
        let (r1, data) = R1::remaining_data_mut_with_arg(data, arg1)?;
        let (r2, data) = R2::remaining_data_mut_with_arg(data, arg2)?;
        Ok(((r1, r2), data))
    }
}
