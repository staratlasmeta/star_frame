//! Extensions to [`AccountInfo`]

use crate::Result;
use common_utils::versioned_account::access::{AccountDataAccess, AccountDataAccessMut};
use common_utils::versioned_account::data_section::AccountDataSection;
use solana_program::account_info::AccountInfo;
use std::cell::{Ref, RefMut};
use std::ptr;
use std::ptr::{metadata, NonNull};

/// Extension trait for [`AccountInfo`].
pub trait AccountInfoData {
    /// Gets the data + length from the account info.
    /// # Safety
    /// Requires that the [`AccountInfo`] was validly created.
    unsafe fn data_section(&self) -> Result<Ref<AccountDataSection>>;
    /// Gets the data + length from the account info.
    /// # Safety
    /// Requires that the [`AccountInfo`] was validly created.
    unsafe fn data_section_mut(&self) -> Result<RefMut<AccountDataSection>>;
    /// # Safety
    /// Same requirements as [`AccountInfoData::data_section`] and [`AccountInfo::original_data_len`].
    unsafe fn data_access(&self) -> Result<AccountDataAccess>;
    /// # Safety
    /// Same requirements as [`AccountInfoData::data_section_mut`] and [`AccountInfo::original_data_len`].
    unsafe fn data_access_mut(&self) -> Result<AccountDataAccessMut>;
}

impl<'info> AccountInfoData for AccountInfo<'info> {
    unsafe fn data_section(&self) -> Result<Ref<AccountDataSection>> {
        Ok(Ref::map(self.try_borrow_data()?, |data| {
            &*ptr::from_raw_parts(data.as_ptr().sub(8).cast(), metadata(*data))
        }))
    }

    unsafe fn data_section_mut(&self) -> Result<RefMut<AccountDataSection>> {
        Ok(RefMut::map(self.try_borrow_mut_data()?, |data| {
            &mut *ptr::from_raw_parts_mut(data.as_mut_ptr().sub(8).cast(), metadata(*data))
        }))
    }

    unsafe fn data_access(&self) -> Result<AccountDataAccess> {
        Ok(AccountDataAccess {
            original_data_length: self.original_data_len(),
            data_section: self.data_section()?,
        })
    }

    unsafe fn data_access_mut(&self) -> Result<AccountDataAccessMut> {
        let data_ptr = NonNull::from(&mut *self.try_borrow_mut_data()?);
        Ok(AccountDataAccessMut::new(
            data_ptr,
            self.original_data_len(),
            self.data_section_mut()?,
        ))
    }
}
