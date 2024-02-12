use crate::account_set::{
    AccountSet, AccountSetDecode, AccountSetValidate, SignedAccount, SingleAccountSet,
    WritableAccount,
};
use crate::Result;
use anyhow::bail;
use derive_more::{Deref, DerefMut};
use solana_program::account_info::AccountInfo;
use solana_program::program_error::ProgramError;
use star_frame::account_set::AccountSetCleanup;

#[derive(AccountSet, Copy, Clone, Debug, Deref, DerefMut)]
#[repr(transparent)]
#[account_set(skip_default_idl, generics = [where T: AccountSet<'info>])]
#[decode(generics = [<A> where T: AccountSetDecode<'a, 'info, A>], arg = A)]
#[validate(
    generics = [<A> where for<'a> T: AccountSetValidate<'a, 'info, A> + SingleAccountSet<'info>], arg = A,
    extra_validation = self.check_signer(),
)]
#[cleanup(generics = [<A> where T: AccountSetCleanup<'cleanup, 'info, A>], arg = A)]
pub struct Signer<T>(
    #[decode(arg = arg)]
    #[validate(arg = arg)]
    #[cleanup(arg = arg)]
    T,
);
impl<T> Signer<T> {
    pub fn new(info: T) -> Self {
        Self(info)
    }

    pub fn try_from<'info>(info: &T) -> Result<Self>
    where
        T: SingleAccountSet<'info> + Clone,
    {
        if !info.is_signer() {
            bail!(ProgramError::MissingRequiredSignature);
        }
        Ok(Signer::new(info.clone()))
    }
}

impl<'info, T> Signer<T>
where
    T: SingleAccountSet<'info>,
{
    pub fn check_signer(&self) -> Result<()> {
        if self.0.is_signer() {
            Ok(())
        } else {
            Err(ProgramError::MissingRequiredSignature.into())
        }
    }
}

impl<'info, T> SingleAccountSet<'info> for Signer<T>
where
    T: SingleAccountSet<'info>,
{
    fn account_info(&self) -> &AccountInfo<'info> {
        self.0.account_info()
    }
}
impl<'info, T> SignedAccount<'info> for Signer<T>
where
    T: SingleAccountSet<'info>,
{
    fn signer_seeds(&self) -> Option<Vec<&[u8]>> {
        None
    }
}
impl<'info, T> WritableAccount<'info> for Signer<T> where T: WritableAccount<'info> {}

#[cfg(feature = "idl")]
mod idl_impl {
    use super::*;
    use star_frame::idl::AccountSetToIdl;
    use star_frame_idl::account_set::IdlAccountSetDef;
    use star_frame_idl::IdlDefinition;

    impl<'info, T, A> AccountSetToIdl<'info, A> for Signer<T>
    where
        T: AccountSetToIdl<'info, A>,
    {
        fn account_set_to_idl(
            idl_definition: &mut IdlDefinition,
            arg: A,
        ) -> Result<IdlAccountSetDef> {
            T::account_set_to_idl(idl_definition, arg)
                .map(Box::new)
                .map(IdlAccountSetDef::Signer)
        }
    }
}
