//! Versioned accounts

pub mod access;
pub mod account_info;
pub mod combined;
pub mod context;
pub mod data_section;
pub mod list;
pub mod to_from_usize;
pub mod unsized_data;

use crate::versioned_account::unsized_data::UnsizedData;
use crate::SafeZeroCopyAccount;

/// Trait for versioned accounts.
pub trait VersionedAccount: SafeZeroCopyAccount {
    /// Enum of possible references.
    type VersionsRef;
    /// Enum of possible mutable references.
    type VersionsRefMut;

    /// Gets the version of this account.
    fn version(&self) -> u8;
}

/// Claims a version number for an account.
pub trait VersionClaim<const VERSION: u8> {
    /// The claiming type.
    type Claim;
}

/// Trait for versioned accounts with a specific version.
pub trait VersionedAccountVersion<const VERSION: u8>: UnsizedData
where
    for<'a> <Self::Account as VersionedAccount>::VersionsRef: From<&'a Self>,
    for<'a> <Self::Account as VersionedAccount>::VersionsRefMut: From<&'a mut Self>,
{
    /// The account type.
    type Account: VersionedAccount + VersionClaim<VERSION, Claim = Self>;
}

#[cfg(test)]
mod test {
    use bytemuck::{Pod, Zeroable};
    use common_proc::Align1;

    #[repr(C, packed)]
    #[derive(Align1, Pod, Zeroable, Copy, Clone, Debug, Eq, PartialEq)]
    struct Data2 {
        val1: u32,
        val2: u64,
    }
}
