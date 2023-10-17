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
    use crate::util::{MaybeMutRef, MaybeRef};
    use crate::versioned_account::context::{
        AccountDataContext, AccountDataMutContext, AccountDataRefContext,
    };
    use crate::versioned_account::list::List;
    use crate::{PackedValue, Result};
    use bytemuck::{Pod, Zeroable};
    use common_proc::Align1;
    use common_utils::versioned_account::unsized_data::UnsizedData;
    use solana_program::pubkey::Pubkey;
    use std::fmt::Debug;
    use std::mem::size_of_val;
    use std::ptr;
    use std::ptr::NonNull;

    #[repr(C, packed)]
    #[derive(Align1, Pod, Zeroable, Copy, Clone, Debug, Eq, PartialEq)]
    struct Data2 {
        val1: u32,
        val2: u64,
    }

    #[repr(C)]
    #[derive(Align1, Debug)]
    struct AccountStuff {
        header_val: PackedValue<u32>,
        other_val: PackedValue<u64>,
        key: Pubkey,
        items: List<Data2, u32>,
    }

    struct AccountStuffMetaData {
        header_val: <PackedValue<u32> as UnsizedData>::Metadata,
        other_val: <PackedValue<u64> as UnsizedData>::Metadata,
        key: <Pubkey as UnsizedData>::Metadata,
        items: <List<Data2, u32> as UnsizedData>::Metadata,
    }

    // Safety: Guarantees are met.
    unsafe impl UnsizedData for AccountStuff {
        type Metadata = AccountStuffMetaData;

        fn min_data_size() -> usize {
            <PackedValue<u32> as UnsizedData>::min_data_size()
                + <PackedValue<u64> as UnsizedData>::min_data_size()
                + <Pubkey as UnsizedData>::min_data_size()
                + <List<Data2, u32> as UnsizedData>::min_data_size()
        }

        fn from_bytes<'a>(bytes: &mut &'a [u8]) -> Result<(&'a Self, Self::Metadata)> {
            let bytes_ptr = bytes.as_ptr();
            let (_header_val, header_val_meta) =
                <PackedValue<u32> as UnsizedData>::from_bytes(bytes)?;
            let (_other_val, other_val_meta) =
                <PackedValue<u64> as UnsizedData>::from_bytes(bytes)?;
            let (_key, key_meta) = <Pubkey as UnsizedData>::from_bytes(bytes)?;
            let (items, items_meta) = <List<Data2, u32> as UnsizedData>::from_bytes(bytes)?;

            Ok((
                // Safety: This is safe because the pointers are the same as the input.
                unsafe { &*ptr::from_raw_parts(bytes_ptr.cast(), ptr::metadata(items)) },
                AccountStuffMetaData {
                    header_val: header_val_meta,
                    other_val: other_val_meta,
                    key: key_meta,
                    items: items_meta,
                },
            ))
        }

        fn from_mut_bytes<'a>(bytes: &mut &'a mut [u8]) -> Result<(&'a mut Self, Self::Metadata)> {
            let bytes_ptr = bytes.as_mut_ptr();
            let (header_val, header_val_meta) =
                <PackedValue<u32> as UnsizedData>::from_mut_bytes(bytes)?;
            assert_eq!(
                bytes.as_ptr() as usize,
                bytes_ptr as usize + size_of_val(header_val)
            );
            let (other_val, other_val_meta) =
                <PackedValue<u64> as UnsizedData>::from_mut_bytes(bytes)?;
            assert_eq!(
                bytes.as_ptr() as usize,
                header_val as *const PackedValue<u32> as usize + size_of_val(other_val)
            );
            let (key, key_meta) = <Pubkey as UnsizedData>::from_mut_bytes(bytes)?;
            assert_eq!(
                bytes.as_ptr() as usize,
                other_val as *const PackedValue<u64> as usize + size_of_val(key)
            );
            let (items, items_meta) = <List<Data2, u32> as UnsizedData>::from_mut_bytes(bytes)?;
            assert_eq!(
                bytes.as_ptr() as usize,
                key as *const Pubkey as usize + size_of_val(items)
            );

            Ok((
                // Safety: This is safe because the pointers are the same as the input.
                unsafe { &mut *ptr::from_raw_parts_mut(bytes_ptr.cast(), ptr::metadata(items)) },
                AccountStuffMetaData {
                    header_val: header_val_meta,
                    other_val: other_val_meta,
                    key: key_meta,
                    items: items_meta,
                },
            ))
        }
    }
    trait AccountStuffExtension: AccountDataContext<AccountStuff> {
        fn header_val(&self) -> AccountDataRefContext<PackedValue<u32>> {
            self.sub_context(|data, meta| (&data.header_val, MaybeRef::Ref(&meta.header_val)))
        }

        fn other_val(&self) -> AccountDataRefContext<PackedValue<u64>> {
            self.sub_context(|data, meta| (&data.other_val, MaybeRef::Ref(&meta.other_val)))
        }

        fn key(&self) -> AccountDataRefContext<Pubkey> {
            self.sub_context(|data, meta| (&data.key, MaybeRef::Ref(&meta.key)))
        }

        fn items(&self) -> AccountDataRefContext<List<Data2, u32>> {
            self.sub_context(|data, meta| (&data.items, MaybeRef::Ref(&meta.items)))
        }
    }
    impl<T> AccountStuffExtension for T where T: AccountDataContext<AccountStuff> {}
    trait AccountStuffMutExtension {
        fn header_val_mut(&mut self) -> AccountDataMutContext<PackedValue<u32>>;
        fn other_val_mut(&mut self) -> AccountDataMutContext<PackedValue<u64>>;
        fn key_mut(&mut self) -> AccountDataMutContext<Pubkey>;
        fn items_mut(&mut self) -> AccountDataMutContext<List<Data2, u32>>;
    }
    impl<'a> AccountStuffMutExtension for AccountDataMutContext<'a, AccountStuff> {
        fn header_val_mut(&mut self) -> AccountDataMutContext<PackedValue<u32>> {
            self.sub_context_mut(|args| {
                (
                    // Safety: Length change does not access this value.
                    unsafe { &mut args.data.as_mut().header_val },
                    MaybeMutRef::Mut(&mut args.data_meta.header_val),
                    Box::new(|_, _| unreachable!()),
                )
            })
        }

        fn other_val_mut(&mut self) -> AccountDataMutContext<PackedValue<u64>> {
            self.sub_context_mut(|args| {
                (
                    // Safety: Length change does not access this value.
                    unsafe { &mut args.data.as_mut().other_val },
                    MaybeMutRef::Mut(&mut args.data_meta.other_val),
                    Box::new(|_, _| unreachable!()),
                )
            })
        }

        fn key_mut(&mut self) -> AccountDataMutContext<Pubkey> {
            self.sub_context_mut(|args| {
                (
                    // Safety: Length change does not access this value.
                    unsafe { &mut args.data.as_mut().key },
                    MaybeMutRef::Mut(&mut args.data_meta.key),
                    Box::new(|_, _| unreachable!()),
                )
            })
        }

        fn items_mut(&mut self) -> AccountDataMutContext<List<Data2, u32>> {
            self.sub_context_mut(|args| {
                (
                    // This field cannot be accessed while length change is
                    unsafe { &mut args.data.as_mut().items },
                    MaybeMutRef::Mut(&mut args.data_meta.items),
                    Box::new(move |new_length, new_meta| {
                        *args.data = NonNull::from_raw_parts(args.data.cast(), new_meta);
                        (args.set_length)(new_length, new_meta)
                    }),
                )
            })
        }
    }
}
