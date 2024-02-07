use crate::account_set::{AccountSet, SingleAccountSet};
use crate::packed_value::PackedValue;
use crate::prelude::SysCallCore;
use crate::program::StarFrameProgram;
use crate::serialize::{FrameworkFromBytes, FrameworkFromBytesMut};
use crate::Result;
use anyhow::bail;
use bytemuck::{bytes_of, from_bytes, from_bytes_mut};
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::MAX_PERMITTED_DATA_INCREASE;
use solana_program::program_error::ProgramError;
use solana_program::system_instruction::MAX_PERMITTED_DATA_LENGTH;
use star_frame::serialize::unsized_type::UnsizedType;
use std::cell::{Ref, RefMut};
use std::marker::PhantomData;
use std::mem::{size_of, size_of_val};
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

pub trait ProgramAccount {
    type OwnerProgram: StarFrameProgram;
    const DISCRIMINANT: <Self::OwnerProgram as StarFrameProgram>::AccountDiscriminant;

    fn account_data_size(&self) -> usize {
        size_of::<<Self::OwnerProgram as StarFrameProgram>::AccountDiscriminant>()
            + size_of_val(self)
    }
}

fn validate_data_account<T>(account: &DataAccount<T>, sys_calls: &impl SysCallCore) -> Result<()>
where
    T: ProgramAccount + UnsizedType + ?Sized,
{
    if account.info.owner != &T::OwnerProgram::program_id(sys_calls)? {
        bail!(ProgramError::IllegalOwner);
    }

    let data = account.info.try_borrow_data()?;
    if data.len() < size_of::<<T::OwnerProgram as StarFrameProgram>::AccountDiscriminant>() {
        bail!(ProgramError::InvalidAccountData);
    }
    let discriminant: &<T::OwnerProgram as StarFrameProgram>::AccountDiscriminant = from_bytes(
        &data[0..size_of::<<T::OwnerProgram as StarFrameProgram>::AccountDiscriminant>()],
    );
    if discriminant != &T::DISCRIMINANT {
        bail!(ProgramError::InvalidAccountData);
    }
    Ok(())
}

#[derive(AccountSet, Debug)]
#[validate(
    extra_validation = validate_data_account(self, sys_calls),
)]
pub struct DataAccount<'info, T: ProgramAccount + UnsizedType + ?Sized> {
    info: AccountInfo<'info>,
    phantom_t: PhantomData<T>,
}
impl<'info, T> DataAccount<'info, T>
where
    T: ProgramAccount + UnsizedType + ?Sized,
{
    fn check_discriminant(bytes: &[u8]) -> Result<()> {
        if bytes.len() < size_of::<<T::OwnerProgram as StarFrameProgram>::AccountDiscriminant>()
            || from_bytes::<PackedValue<<T::OwnerProgram as StarFrameProgram>::AccountDiscriminant>>(
                &bytes[..size_of::<<T::OwnerProgram as StarFrameProgram>::AccountDiscriminant>()],
            ) != &PackedValue(T::DISCRIMINANT)
        {
            bail!(ProgramError::InvalidAccountData)
        } else {
            Ok(())
        }
    }

    pub fn data<'a>(&'a self) -> Result<DataRef<'a, T>> {
        let r: Ref<'a, _> = self.info.try_borrow_data()?;
        Self::check_discriminant(&r)?;
        let mut r_ptr: Option<NonNull<[u8]>> = None;
        let r = Ref::map(r, |bytes| {
            r_ptr = Some(NonNull::from(&**bytes));
            from_bytes(&bytes[0..0])
        });
        let data: T::Ref<'a> = T::Ref::from_bytes(&mut unsafe {
            &*r_ptr.unwrap().as_ptr().byte_add(size_of::<
                <T::OwnerProgram as StarFrameProgram>::AccountDiscriminant,
            >())
        })?;
        Ok(DataRef { _r: r, data })
    }

    pub fn data_mut<'a>(&'a mut self) -> Result<DataRefMut<'a, T>> {
        let original_data_len = unsafe { self.info.original_data_len() };
        let r: RefMut<'a, _> = self.info.try_borrow_mut_data()?;
        Self::check_discriminant(&r)?;
        let mut r_ptr: Option<NonNull<[u8]>> = None;
        let r = RefMut::map(r, |bytes| {
            r_ptr = Some(NonNull::from(&**bytes));
            from_bytes_mut(&mut bytes[0..0])
        });
        let r_ptr = r_ptr.unwrap();
        let mut data_ptr = unsafe {
            NonNull::new(r_ptr.as_ptr().byte_add(size_of::<
                <T::OwnerProgram as StarFrameProgram>::AccountDiscriminant,
            >()))
            .unwrap()
        };
        let data_len_ptr = unsafe { r_ptr.as_ptr().byte_sub(8).cast::<u64>() };
        Ok(DataRefMut {
            data: T::RefMut::from_bytes_mut(
                &mut unsafe { data_ptr.as_mut() },
                move |new_len, _| {
                    let new_len = new_len
                        + size_of::<<T::OwnerProgram as StarFrameProgram>::AccountDiscriminant>();
                    if new_len > original_data_len + MAX_PERMITTED_DATA_INCREASE
                        || new_len as u64 > MAX_PERMITTED_DATA_LENGTH
                    {
                        bail!(ProgramError::InvalidRealloc)
                    } else {
                        unsafe { data_len_ptr.write(new_len as u64) };
                        Ok(data_ptr.cast())
                    }
                },
            )?,
            _r: r,
        })
    }

    /// Closes the account
    pub fn close(&mut self) -> Result<()> {
        self.info.realloc(
            size_of::<<T::OwnerProgram as StarFrameProgram>::AccountDiscriminant>(),
            false,
        )?;
        self.info.try_borrow_mut_data()?.copy_from_slice(bytes_of(
            &<T::OwnerProgram as StarFrameProgram>::CLOSED_ACCOUNT_DISCRIMINANT,
        ));
        Ok(())
    }
}

impl<'info, T> SingleAccountSet<'info> for DataAccount<'info, T>
where
    T: ProgramAccount + UnsizedType + ?Sized,
{
    fn account_info(&self) -> &AccountInfo<'info> {
        &self.info
    }
}

#[derive(Debug)]
pub struct DataRef<'a, T>
where
    T: 'a + ProgramAccount + UnsizedType + ?Sized,
{
    data: T::Ref<'a>,
    _r: Ref<'a, [u8; 0]>,
}
impl<'a, T> Deref for DataRef<'a, T>
where
    T: 'a + ProgramAccount + UnsizedType + ?Sized,
{
    type Target = T::Ref<'a>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

#[derive(Debug)]
pub struct DataRefMut<'a, T>
where
    T: 'a + ProgramAccount + UnsizedType + ?Sized,
{
    data: T::RefMut<'a>,
    _r: RefMut<'a, [u8; 0]>,
}
impl<'a, T> Deref for DataRefMut<'a, T>
where
    T: 'a + ProgramAccount + UnsizedType + ?Sized,
{
    type Target = T::RefMut<'a>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
impl<'a, T> DerefMut for DataRefMut<'a, T>
where
    T: 'a + ProgramAccount + UnsizedType + ?Sized,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
