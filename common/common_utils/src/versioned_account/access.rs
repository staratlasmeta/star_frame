//! Access object for an account info's data.

use crate::versioned_account::context::AccountDataMutContext;
use crate::versioned_account::data_section::AccountDataSection;
use crate::UtilError;
use anchor_lang::error;
use common_utils::util::{MaybeMutRef, MaybeRef};
use common_utils::versioned_account::context::AccountDataRefContext;
use common_utils::versioned_account::unsized_data::UnsizedData;
use num_traits::ToPrimitive;
use solana_program::entrypoint::MAX_PERMITTED_DATA_INCREASE;
use solana_program::system_instruction::MAX_PERMITTED_DATA_LENGTH;
use std::cell::{Ref, RefMut};
use std::ops::{Deref, DerefMut};
use std::ptr;
use std::ptr::NonNull;

/// Generic trait for both [`AccountDataAccess`] and [`AccountDataAccessMut`].
pub trait AccountData: Sized + Deref<Target = AccountDataSection> {
    /// Gets the original data length.
    fn original_data_length(this: &Self) -> usize;
}

/// Generic trait for [`AccountDataAccessMut`].
pub trait AccountDataMut: AccountData + DerefMut {}
impl<T> AccountDataMut for T where T: AccountData + DerefMut {}

/// Immutable access to an account's data.
#[derive(Debug)]
pub struct AccountDataAccess<'a> {
    pub(super) original_data_length: usize,
    pub(super) data_section: Ref<'a, AccountDataSection>,
}
impl<'a> AccountDataAccess<'a> {
    /// Gets an immutable context object.
    #[must_use]
    pub fn context<T: ?Sized + UnsizedData>(&self) -> AccountDataRefContext<T> {
        let mut bytes = &self.data_section.data;
        let (data, meta) = T::from_bytes(&mut bytes).unwrap();
        AccountDataRefContext {
            data,
            meta: MaybeRef::Owned(meta),
        }
    }
}

impl<'a> Deref for AccountDataAccess<'a> {
    type Target = AccountDataSection;

    fn deref(&self) -> &Self::Target {
        &self.data_section
    }
}

impl<'a> AccountData for AccountDataAccess<'a> {
    fn original_data_length(this: &Self) -> usize {
        this.original_data_length
    }
}

/// Mutable access to an account's data.
#[derive(Debug)]
pub struct AccountDataAccessMut<'a> {
    data_ptr: NonNull<&'a mut [u8]>,
    original_data_length: usize,
    /// This should never be none, just an option so we can replace it.
    /// Also will be niche optimized to the same size.
    data_section: Option<RefMut<'a, AccountDataSection>>,
}

impl<'a> Deref for AccountDataAccessMut<'a> {
    type Target = AccountDataSection;

    fn deref(&self) -> &Self::Target {
        self.data_section.as_ref().unwrap()
    }
}

impl<'a> DerefMut for AccountDataAccessMut<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data_section.as_mut().unwrap()
    }
}

impl<'a> AccountData for AccountDataAccessMut<'a> {
    fn original_data_length(this: &Self) -> usize {
        this.original_data_length
    }
}

impl<'a> AccountDataAccessMut<'a> {
    /// # Safety
    /// Must be the valid original data length for the data section.
    /// Data ptr must point to the same data as the data section.
    #[must_use]
    pub unsafe fn new(
        data_ptr: NonNull<&'a mut [u8]>,
        original_data_length: usize,
        data_section: RefMut<'a, AccountDataSection>,
    ) -> Self {
        Self {
            data_ptr,
            original_data_length,
            data_section: Some(data_section),
        }
    }

    /// Sets the data length of the account info.
    pub fn set_data_length(&mut self, length: usize) -> common_utils::Result<()> {
        if length > self.original_data_length + MAX_PERMITTED_DATA_INCREASE
            || length > MAX_PERMITTED_DATA_LENGTH.to_usize().unwrap()
        {
            Err(error!(UtilError::ReallocError).into())
        } else {
            self.data_section = Some(RefMut::map(
                self.data_section.take().unwrap(),
                |mut data| {
                    // Safety: We checked that the length is valid.
                    unsafe {
                        AccountDataSection::set_length(&mut data, length);
                    }
                    data
                },
            ));

            Ok(())
        }
    }

    /// Gets an immutable context object.
    #[must_use]
    pub fn context<T: ?Sized + UnsizedData>(&self) -> AccountDataRefContext<T> {
        let AccountDataSection { data: bytes, .. } = &**self.data_section.as_ref().unwrap();
        let mut bytes = bytes;
        let (data, meta) = T::from_bytes(&mut bytes).unwrap();
        AccountDataRefContext {
            data,
            meta: MaybeRef::Owned(meta),
        }
    }

    /// Gets a mutable context object.
    #[must_use]
    pub fn context_mut<T: ?Sized + UnsizedData>(&mut self) -> AccountDataMutContext<T> {
        let original_data_length = self.original_data_length;
        let AccountDataSection { data: bytes, .. } = &mut **self.data_section.as_mut().unwrap();
        let mut bytes = bytes;
        let (data, data_meta) = T::from_mut_bytes(&mut bytes).unwrap();
        let data = NonNull::from(data);
        AccountDataMutContext {
            original_data_length,
            data_meta: MaybeMutRef::Owned(data_meta),
            set_length: Box::new(|length, _new_meta| {
                self.set_data_length(length)?;
                // Safety: We are only changing the length, and that length was wer properly.
                unsafe {
                    *self.data_ptr.as_mut() = &mut *ptr::from_raw_parts_mut(
                        self.data_ptr.as_mut().as_mut_ptr().cast(),
                        length,
                    );
                }
                Ok(())
            }),
            data,
        }
    }
}
