use crate::account_set::{
    AccountSet, AccountSetDecode, AccountSetValidate, CanInitAccount, CanSetSeeds,
    HasProgramAccount, HasSeeds, SignedAccount, SingleAccountSet, SingleAccountSetMetadata,
    WritableAccount,
};
use crate::prelude::SyscallInvoke;
use crate::Result;
use derive_more::{Deref, DerefMut};
use solana_program::account_info::AccountInfo;
use star_frame::account_set::AccountSetCleanup;

#[derive(AccountSet, Copy, Clone, Debug, Deref, DerefMut)]
#[account_set(skip_default_idl, generics = [where T: AccountSet<'info>])]
#[validate(
    generics = [<A> where T: AccountSetValidate<'info, A> + SingleAccountSet<'info>], arg = A,
    extra_validation = self.check_writable(),
)]
#[decode(generics = [<A> where T: AccountSetDecode<'a, 'info, A>], arg = A)]
#[cleanup(generics = [<A> where T: AccountSetCleanup<'info, A>], arg = A)]
#[repr(transparent)]
pub struct Writable<T>(
    #[decode(arg = arg)]
    #[validate(arg = arg)]
    #[cleanup(arg = arg)]
    pub(crate) T,
);

pub type WritableInfo<'info> = Writable<AccountInfo<'info>>;

impl<'info, T> SingleAccountSet<'info> for Writable<T>
where
    T: SingleAccountSet<'info>,
{
    const METADATA: SingleAccountSetMetadata = SingleAccountSetMetadata {
        should_mut: true,
        ..T::METADATA
    };

    fn account_info(&self) -> &AccountInfo<'info> {
        self.0.account_info()
    }
}
impl<'info, T> SignedAccount<'info> for Writable<T>
where
    T: SignedAccount<'info>,
{
    fn signer_seeds(&self) -> Option<Vec<&[u8]>> {
        self.0.signer_seeds()
    }
}

impl<'info, T> WritableAccount<'info> for Writable<T> where T: SingleAccountSet<'info> {}

impl<T> HasProgramAccount for Writable<T>
where
    T: HasProgramAccount,
{
    type ProgramAccount = T::ProgramAccount;
}

impl<T> HasSeeds for Writable<T>
where
    T: HasSeeds,
{
    type Seeds = T::Seeds;
}

impl<'info, A, T> CanSetSeeds<'info, A> for Writable<T>
where
    T: CanSetSeeds<'info, A>,
{
    fn set_seeds(&mut self, arg: &A, syscalls: &mut impl SyscallInvoke<'info>) -> Result<()> {
        T::set_seeds(&mut self.0, arg, syscalls)
    }
}

impl<'info, A, T> CanInitAccount<'info, A> for Writable<T>
where
    T: SingleAccountSet<'info> + CanInitAccount<'info, A>,
{
    fn init(
        &mut self,
        arg: A,
        syscalls: &mut impl SyscallInvoke<'info>,
        account_seeds: Option<Vec<&[u8]>>,
    ) -> Result<()> {
        self.0.init(arg, syscalls, account_seeds)
    }
}

#[cfg(feature = "idl")]
mod idl_impl {
    use super::*;
    use star_frame::idl::AccountSetToIdl;
    use star_frame_idl::account_set::IdlAccountSetDef;
    use star_frame_idl::IdlDefinition;

    impl<'info, T, A> AccountSetToIdl<'info, A> for Writable<T>
    where
        T: AccountSetToIdl<'info, A>,
    {
        fn account_set_to_idl(
            idl_definition: &mut IdlDefinition,
            arg: A,
        ) -> Result<IdlAccountSetDef> {
            T::account_set_to_idl(idl_definition, arg)
                .map(Box::new)
                .map(IdlAccountSetDef::Writable)
        }
    }
}
