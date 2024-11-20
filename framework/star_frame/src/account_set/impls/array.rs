use crate::account_set::{AccountSet, AccountSetCleanup, AccountSetDecode, AccountSetValidate};
use crate::client::{ClientAccountSet, CpiAccountSet};
use crate::prelude::SyscallAccountCache;
use crate::syscalls::SyscallInvoke;
use crate::Result;
use array_init::try_array_init;
use solana_program::account_info::AccountInfo;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;

impl<'info, A, const N: usize> AccountSet<'info> for [A; N]
where
    A: AccountSet<'info>,
{
    fn set_account_cache(&mut self, _syscalls: &mut impl SyscallAccountCache<'info>) {}
}

impl<'info, A, const N: usize> CpiAccountSet<'info> for [A; N]
where
    A: CpiAccountSet<'info>,
{
    type CpiAccounts<'a> = [A::CpiAccounts<'a>; N];
    const MIN_LEN: usize = N * A::MIN_LEN;
    fn extend_account_infos(
        accounts: Self::CpiAccounts<'info>,
        infos: &mut Vec<AccountInfo<'info>>,
    ) {
        for a in accounts {
            A::extend_account_infos(a, infos);
        }
    }
    fn extend_account_metas(
        program_id: &Pubkey,
        accounts: &Self::CpiAccounts<'info>,
        metas: &mut Vec<AccountMeta>,
    ) {
        for a in accounts {
            A::extend_account_metas(program_id, a, metas);
        }
    }
}

impl<A, const N: usize> ClientAccountSet for [A; N]
where
    A: ClientAccountSet,
{
    type ClientAccounts = [A::ClientAccounts; N];
    const MIN_LEN: usize = N * A::MIN_LEN;
    fn extend_account_metas(
        program_id: &Pubkey,
        accounts: &Self::ClientAccounts,
        metas: &mut Vec<AccountMeta>,
    ) {
        for a in accounts {
            A::extend_account_metas(program_id, a, metas);
        }
    }
}

impl<'a, 'info, A, const N: usize, DArg> AccountSetDecode<'a, 'info, [DArg; N]> for [A; N]
where
    A: AccountSetDecode<'a, 'info, DArg>,
{
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo<'info>],
        decode_input: [DArg; N],
        syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<Self> {
        let mut decode_input = decode_input.into_iter();
        try_array_init(|_| A::decode_accounts(accounts, decode_input.next().unwrap(), syscalls))
    }
}
impl<'a, 'info, A, const N: usize, DArg> AccountSetDecode<'a, 'info, (DArg,)> for [A; N]
where
    A: AccountSetDecode<'a, 'info, DArg>,
    DArg: Clone,
{
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo<'info>],
        decode_input: (DArg,),
        syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<Self> {
        try_array_init(|_| A::decode_accounts(accounts, decode_input.0.clone(), syscalls))
    }
}
impl<'a, 'info, A, const N: usize> AccountSetDecode<'a, 'info, ()> for [A; N]
where
    A: AccountSetDecode<'a, 'info, ()>,
{
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo<'info>],
        decode_input: (),
        syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<Self> {
        Self::decode_accounts(accounts, (decode_input,), syscalls)
    }
}

impl<'info, A, const N: usize, VArg> AccountSetValidate<'info, [VArg; N]> for [A; N]
where
    A: AccountSetValidate<'info, VArg>,
{
    fn validate_accounts(
        &mut self,
        validate_input: [VArg; N],
        syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()> {
        for (a, v) in self.iter_mut().zip(validate_input) {
            a.validate_accounts(v, syscalls)?;
        }
        Ok(())
    }
}
impl<'info, A, const N: usize, VArg> AccountSetValidate<'info, (VArg,)> for [A; N]
where
    A: AccountSetValidate<'info, VArg>,
    VArg: Clone,
{
    fn validate_accounts(
        &mut self,
        validate_input: (VArg,),
        syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()> {
        for a in self {
            a.validate_accounts(validate_input.0.clone(), syscalls)?;
        }
        Ok(())
    }
}
impl<'info, A, const N: usize> AccountSetValidate<'info, ()> for [A; N]
where
    A: AccountSetValidate<'info, ()>,
{
    fn validate_accounts(
        &mut self,
        validate_input: (),
        syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()> {
        for a in self {
            a.validate_accounts(validate_input, syscalls)?;
        }
        Ok(())
    }
}

impl<'info, A, const N: usize, VArg> AccountSetCleanup<'info, [VArg; N]> for [A; N]
where
    A: AccountSetCleanup<'info, VArg>,
{
    fn cleanup_accounts(
        &mut self,
        cleanup_input: [VArg; N],
        syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()> {
        for (a, v) in self.iter_mut().zip(cleanup_input) {
            a.cleanup_accounts(v, syscalls)?;
        }
        Ok(())
    }
}
impl<'info, A, const N: usize, VArg> AccountSetCleanup<'info, (VArg,)> for [A; N]
where
    A: AccountSetCleanup<'info, VArg>,
    VArg: Clone,
{
    fn cleanup_accounts(
        &mut self,
        cleanup_input: (VArg,),
        syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()> {
        for a in self {
            a.cleanup_accounts(cleanup_input.0.clone(), syscalls)?;
        }
        Ok(())
    }
}
impl<'info, A, const N: usize> AccountSetCleanup<'info, ()> for [A; N]
where
    A: AccountSetCleanup<'info, ()>,
{
    fn cleanup_accounts(
        &mut self,
        cleanup_input: (),
        syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()> {
        for a in self {
            a.cleanup_accounts(cleanup_input, syscalls)?;
        }
        Ok(())
    }
}

#[cfg(feature = "idl")]
pub mod idl_impl {
    use crate::idl::AccountSetToIdl;
    use star_frame_idl::account_set::IdlAccountSetDef;
    use star_frame_idl::IdlDefinition;

    impl<'info, T, const N: usize> AccountSetToIdl<'info, ()> for [T; N]
    where
        T: AccountSetToIdl<'info, ()>,
    {
        fn account_set_to_idl(
            idl_definition: &mut IdlDefinition,
            arg: (),
        ) -> crate::Result<IdlAccountSetDef> {
            let account_set = Box::new(T::account_set_to_idl(idl_definition, arg)?);
            Ok(IdlAccountSetDef::Many {
                account_set,
                min: N,
                max: Some(N),
            })
        }
    }
}
