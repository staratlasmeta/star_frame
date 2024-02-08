use crate::account_set::{
    AccountSet, AccountSetDecode, AccountSetValidate, SignedAccount, SingleAccountSet,
    WritableAccount,
};
use crate::Result;
use solana_program::account_info::AccountInfo;
use solana_program::program_error::ProgramError;
use star_frame::account_set::AccountSetCleanup;
use std::ops::{Deref, DerefMut};

#[derive(AccountSet, Copy, Clone, Debug)]
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

impl<'info, T> Writable<T>
where
    T: SingleAccountSet<'info>,
{
    pub fn check_writable(&self) -> Result<()> {
        if self.0.is_writable() {
            Ok(())
        } else {
            Err(ProgramError::AccountBorrowFailed.into())
        }
    }
}

impl<'info, T> SingleAccountSet<'info> for Writable<T>
where
    T: SingleAccountSet<'info>,
{
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

impl<T> Deref for Writable<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> DerefMut for Writable<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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
