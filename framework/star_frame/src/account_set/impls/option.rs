use crate::account_set::{AccountSet, AccountSetCleanup, AccountSetDecode, AccountSetValidate};
use crate::syscalls::SyscallInvoke;
use crate::Result;
use anyhow::bail;
use solana_program::account_info::AccountInfo;
use solana_program::instruction::AccountMeta;
use solana_program::msg;
use solana_program::program_error::ProgramError;

impl<'info, A> AccountSet<'info> for Option<A>
where
    A: AccountSet<'info>,
{
    fn try_to_accounts<'a, E>(
        &'a self,
        add_account: impl FnMut(&'a AccountInfo<'info>) -> crate::Result<(), E>,
    ) -> crate::Result<(), E>
    where
        'info: 'a,
    {
        if let Some(s) = self {
            s.try_to_accounts(add_account)
        } else {
            Ok(())
        }
    }

    fn to_account_metas(&self, add_account_meta: impl FnMut(AccountMeta)) {
        if let Some(s) = self {
            s.to_account_metas(add_account_meta);
        }
    }
}

impl<'a, 'info, A, DArg> AccountSetDecode<'a, 'info, Option<DArg>> for Option<A>
where
    A: AccountSetDecode<'a, 'info, DArg>,
{
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo<'info>],
        decode_input: Option<DArg>,
        syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<Self> {
        match decode_input {
            None => Ok(None),
            Some(arg) => Ok(Some(A::decode_accounts(accounts, arg, syscalls)?)),
        }
    }
}
impl<'a, 'info, A> AccountSetDecode<'a, 'info, bool> for Option<A>
where
    A: AccountSetDecode<'a, 'info, ()>,
{
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo<'info>],
        decode_input: bool,
        syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<Self> {
        Self::decode_accounts(
            accounts,
            if decode_input { Some(()) } else { None },
            syscalls,
        )
    }
}
#[derive(Debug, Copy, Clone)]
pub struct Remaining<Arg>(pub Arg);
impl<'a, 'info, A, Arg> AccountSetDecode<'a, 'info, Remaining<Arg>> for Option<A>
where
    A: AccountSetDecode<'a, 'info, Arg>,
{
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo<'info>],
        decode_input: Remaining<Arg>,
        syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<Self> {
        if accounts.is_empty() {
            Ok(None)
        } else {
            Ok(Some(A::decode_accounts(
                accounts,
                decode_input.0,
                syscalls,
            )?))
        }
    }
}
#[derive(Debug, Copy, Clone)]
pub struct ProgramIdOption<Arg>(Arg);
impl<'a, 'info, A, Arg> AccountSetDecode<'a, 'info, ProgramIdOption<Arg>> for Option<A>
where
    A: AccountSetDecode<'a, 'info, Arg>,
{
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo<'info>],
        decode_input: ProgramIdOption<Arg>,
        syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<Self> {
        if accounts.is_empty() {
            bail!(ProgramError::NotEnoughAccountKeys)
        } else if accounts[0].key == syscalls.current_program_id() {
            Ok(None)
        } else {
            Ok(Some(A::decode_accounts(
                accounts,
                decode_input.0,
                syscalls,
            )?))
        }
    }
}
impl<'info, A, VArg> AccountSetValidate<'info, Option<VArg>> for Option<A>
where
    A: AccountSetValidate<'info, VArg>,
{
    fn validate_accounts(
        &mut self,
        validate_input: Option<VArg>,
        sys_calls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()> {
        match (self, validate_input) {
            (Some(s), Some(i)) => s.validate_accounts(i, sys_calls),
            (Some(_), None) => {
                msg!("Optional account set provided with validate arg `None` when self is `Some`");
                bail!(ProgramError::InvalidArgument)
            }
            _ => Ok(()),
        }
    }
}
impl<'info, A, VArg> AccountSetValidate<'info, (VArg,)> for Option<A>
where
    A: AccountSetValidate<'info, VArg>,
{
    fn validate_accounts(
        &mut self,
        validate_input: (VArg,),
        sys_calls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()> {
        self.validate_accounts(Some(validate_input.0), sys_calls)
    }
}
impl<'info, A> AccountSetValidate<'info, ()> for Option<A>
where
    A: AccountSetValidate<'info, ()>,
{
    fn validate_accounts(
        &mut self,
        validate_input: (),
        sys_calls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()> {
        self.validate_accounts(Some(validate_input), sys_calls)
    }
}

impl<'info, A, CArg> AccountSetCleanup<'info, Option<CArg>> for Option<A>
where
    A: AccountSetCleanup<'info, CArg>,
{
    fn cleanup_accounts(
        &mut self,
        cleanup_input: Option<CArg>,
        sys_calls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()> {
        match (self, cleanup_input) {
            (Some(s), Some(i)) => s.cleanup_accounts(i, sys_calls),
            (Some(_), None) => {
                msg!("Optional account set provided with cleanup arg `None` when self is `Some`");
                bail!(ProgramError::InvalidArgument)
            }
            _ => Ok(()),
        }
    }
}
impl<'info, A, VArg> AccountSetCleanup<'info, (VArg,)> for Option<A>
where
    A: AccountSetCleanup<'info, VArg>,
{
    fn cleanup_accounts(
        &mut self,
        cleanup_input: (VArg,),
        sys_calls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()> {
        self.cleanup_accounts(Some(cleanup_input.0), sys_calls)
    }
}
impl<'info, A> AccountSetCleanup<'info, ()> for Option<A>
where
    A: AccountSetCleanup<'info, ()>,
{
    fn cleanup_accounts(
        &mut self,
        cleanup_input: (),
        sys_calls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()> {
        self.cleanup_accounts(Some(cleanup_input), sys_calls)
    }
}

mod idl_impl {
    use crate::idl::AccountSetToIdl;
    use crate::Result;
    use star_frame_idl::account_set::IdlAccountSetDef;
    use star_frame_idl::IdlDefinition;

    // todo: figure out our optionals for IDLs. Thinking we should remove our separate decode
    //  strategies and just use the program id method. This would make using option much simpler on
    //  arg side and be more in line with how the rest of the ecosystem handles optionals.
    impl<'info, A, Arg> AccountSetToIdl<'info, Arg> for Option<A>
    where
        A: AccountSetToIdl<'info, Arg>,
    {
        fn account_set_to_idl(
            idl_definition: &mut IdlDefinition,
            arg: Arg,
        ) -> Result<IdlAccountSetDef> {
            let mut set = A::account_set_to_idl(idl_definition, arg)?;
            if let Ok(inner) = set.single() {
                inner.optional = true;
                return Ok(set);
            }
            Ok(IdlAccountSetDef::Or(vec![
                set,
                IdlAccountSetDef::empty_struct(),
            ]))
        }
    }
}
