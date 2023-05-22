mod chained_data;
mod list;
mod remaining_data;

pub use chained_data::*;
pub use list::*;
pub use remaining_data::*;

use crate::prelude::*;
use anchor_lang::Discriminator;
use bytemuck::{from_bytes, from_bytes_mut};
use common_utils::{Advance, AdvanceArray, StrongTypedStruct};
use std::cell::{Ref, RefMut};
use std::mem::size_of;

/// A wrapper allowing access to both the account data and remaining data.
#[repr(transparent)]
#[derive(Debug)]
pub struct ZeroCopyWrapper<'a, 'info, A>(&'a AccountLoader<'info, A>)
where
    A: SafeZeroCopyAccount;
#[allow(clippy::expl_impl_clone_on_copy)]
impl<A> Clone for ZeroCopyWrapper<'_, '_, A>
where
    A: SafeZeroCopyAccount,
{
    fn clone(&self) -> Self {
        Self(self.0)
    }
}
impl<A> Copy for ZeroCopyWrapper<'_, '_, A> where A: SafeZeroCopyAccount {}
impl<'a, 'info, A> AsRef<AccountInfo<'info>> for ZeroCopyWrapper<'a, 'info, A>
where
    A: SafeZeroCopyAccount,
{
    fn as_ref(&self) -> &AccountInfo<'info> {
        self.0.as_ref()
    }
}
impl<'a, 'info, A> ZeroCopyWrapper<'a, 'info, A>
where
    A: SafeZeroCopyAccount,
{
    /// Initializes the discriminant of the account.
    #[inline]
    pub fn init(&mut self) -> Result<RefMut<'_, A>> {
        let mut data = self.0.as_ref().data.borrow_mut();
        let current_data = AdvanceArray::advance_array(&mut &mut **data);
        if *current_data != [0; 8] {
            return Err(error!(ErrorCode::AccountDiscriminatorAlreadySet));
        }
        *current_data = A::discriminator();
        Ok(RefMut::map(data, |data| {
            from_bytes_mut(&mut data[8..A::MIN_DATA_SIZE])
        }))
    }

    /// Initializes the discriminant of the account and ensures strong typing
    pub fn init_strong(&mut self) -> Result<RefMut<'_, A::StrongTyped>>
    where
        A: StrongTypedStruct,
    {
        Ok(RefMut::map(self.init()?, |data| data.as_strong_typed_mut()))
    }

    /// Gets the account data of the account.
    #[inline]
    pub fn data(&self) -> Result<Ref<'_, A>> {
        Ok(self.data_and_extra()?.0)
    }

    /// Gets the account data of the account mutably.
    #[inline]
    pub fn data_mut(&mut self) -> Result<RefMut<'_, A>> {
        Ok(self.data_and_extra_mut()?.0)
    }

    /// Gets the account data of the account mutably and ensures strong typing
    pub fn data_mut_strong(&mut self) -> Result<RefMut<'_, A::StrongTyped>>
    where
        A: StrongTypedStruct,
    {
        Ok(RefMut::map(self.data_mut()?, |data| {
            data.as_strong_typed_mut()
        }))
    }

    /// Gets the account data of the account and the excess data.
    pub fn data_and_extra(&self) -> Result<(Ref<'_, A>, Ref<'_, [u8]>)>
    where
        A: SafeZeroCopyAccount,
    {
        let data_ref = self.0.as_ref().data.borrow();
        verify_discriminant::<A>(*data_ref)?;
        Ok(Ref::map_split(data_ref, |data| {
            let mut data = &data[8..];
            (from_bytes(data.advance(size_of::<A>())), data)
        }))
    }

    /// Gets the account data of the account mutably and the excess data.
    pub fn data_and_extra_mut(&mut self) -> Result<(RefMut<'_, A>, RefMut<'_, [u8]>)>
    where
        A: SafeZeroCopyAccount,
    {
        let data_ref = self.0.as_ref().data.borrow_mut();
        verify_discriminant::<A>(*data_ref)?;
        Ok(RefMut::map_split(data_ref, |data| {
            let mut data = &mut data[8..];
            (from_bytes_mut(data.advance(size_of::<A>())), data)
        }))
    }

    /// Gets the data and remaining data of the account using the provided arg.
    #[inline]
    #[allow(clippy::type_complexity)]
    pub fn data_and_remaining_with_arg<Arg>(
        &self,
        arg: Arg,
    ) -> Result<(
        Ref<A>,
        <A::RemainingData as RemainingDataWithArg<Arg>>::Data,
        Ref<[u8]>,
    )>
    where
        A: WrappableAccount<Arg>,
    {
        let (account, data) = self.data_and_extra()?;
        let (remaining, extra) = A::RemainingData::remaining_data_with_arg(data, arg)?;
        Ok((account, remaining, extra))
    }

    /// Gets the data and remaining data of the account.
    #[inline]
    #[allow(clippy::type_complexity)]
    pub fn data_and_remaining(
        &self,
    ) -> Result<(
        Ref<A>,
        <A::RemainingData as RemainingDataWithArg<()>>::Data,
        Ref<[u8]>,
    )>
    where
        A: WrappableAccount<()>,
    {
        let (account, data) = self.data_and_extra()?;
        let (remaining, extra) = A::RemainingData::remaining_data(data)?;
        Ok((account, remaining, extra))
    }

    /// Gets the data and remaining data of the account mutably using the provided arg.
    #[inline]
    #[allow(clippy::type_complexity)]
    pub fn data_and_remaining_mut_with_arg<Arg>(
        &mut self,
        arg: Arg,
    ) -> Result<(
        RefMut<A>,
        <A::RemainingData as RemainingDataWithArg<Arg>>::DataMut,
        RefMut<[u8]>,
    )>
    where
        A: WrappableAccount<Arg>,
    {
        let (account, data) = self.data_and_extra_mut()?;
        let (remaining, extra) = A::RemainingData::remaining_data_mut_with_arg(data, arg)?;
        Ok((account, remaining, extra))
    }

    /// Gets the data and remaining data of the account mutably.
    #[inline]
    #[allow(clippy::type_complexity)]
    pub fn data_and_remaining_mut(
        &mut self,
    ) -> Result<(
        RefMut<A>,
        <A::RemainingData as RemainingDataWithArg<()>>::DataMut,
        RefMut<[u8]>,
    )>
    where
        A: WrappableAccount<()>,
    {
        let (account, data) = self.data_and_extra_mut()?;
        let (remaining, extra) = A::RemainingData::remaining_data_mut(data)?;
        Ok((account, remaining, extra))
    }

    /// Gets the remaining data using an arg.
    pub fn remaining_with_arg<Arg>(
        &self,
        arg: Arg,
    ) -> Result<<A::RemainingData as RemainingDataWithArg<Arg>>::Data>
    where
        A: WrappableAccount<Arg>,
    {
        Ok(self.data_and_remaining_with_arg(arg)?.1)
    }

    /// Gets the remaining data.
    pub fn remaining(&self) -> Result<<A::RemainingData as RemainingDataWithArg<()>>::Data>
    where
        A: WrappableAccount<()>,
    {
        Ok(self.data_and_remaining()?.1)
    }

    /// Gets the remaining data mutably using an arg.
    pub fn remaining_mut_with_arg<Arg>(
        &mut self,
        arg: Arg,
    ) -> Result<<A::RemainingData as RemainingDataWithArg<Arg>>::DataMut>
    where
        A: WrappableAccount<Arg>,
    {
        Ok(self.data_and_remaining_mut_with_arg(arg)?.1)
    }

    /// Gets the remaining data mutably
    pub fn remaining_mut(
        &mut self,
    ) -> Result<<A::RemainingData as RemainingDataWithArg<()>>::DataMut>
    where
        A: WrappableAccount<()>,
    {
        Ok(self.data_and_remaining_mut()?.1)
    }

    /// Gets the internal type of the wrapper
    #[must_use]
    pub fn to_inner(&self) -> &'a AccountLoader<'info, A> {
        self.0
    }
}
impl<'a, 'info, A> From<&'a AccountLoader<'info, A>> for ZeroCopyWrapper<'a, 'info, A>
where
    A: SafeZeroCopyAccount,
{
    fn from(from: &'a AccountLoader<'info, A>) -> Self {
        Self(from)
    }
}

/// Verifies the discriminant of the account.
pub fn verify_discriminant<A: Discriminator>(mut data: &[u8]) -> Result<()> {
    if data.len() < A::discriminator().len() {
        return Err(error!(ErrorCode::AccountDiscriminatorNotFound));
    }

    let disc_bytes = data.advance_array();
    if disc_bytes != &A::discriminator() {
        return Err(error!(ErrorCode::AccountDiscriminatorMismatch));
    }

    Ok(())
}

/// An account that can be used in a [`ZeroCopyWrapper`].
pub trait WrappableAccount<A = ()>: SafeZeroCopyAccount {
    /// The remaining data in the account
    type RemainingData: for<'a> RemainingDataWithArg<'a, A>;
}
