use crate::account_set::{
    AccountSet, AccountSetDecode, AccountSetValidate, SignedAccount, SingleAccountSet,
};

use crate::Result;
use derive_more::{Deref, DerefMut};
use solana_program::account_info::AccountInfo;
use star_frame::account_set::AccountSetCleanup;
use std::fmt::Debug;

#[derive(AccountSet, Copy, Clone, Debug, Deref, DerefMut)]
#[repr(transparent)]
#[account_set(skip_default_idl, generics = [where T: AccountSet<'info>])]
#[decode(generics = [<A> where T: AccountSetDecode<'a, 'info, A>], arg = A)]
#[validate(
    generics = [<A> where T: AccountSetValidate<'info, A> + SingleAccountSet<'info>], arg = A,
    extra_validation = self.check_signer(),
)]
#[cleanup(generics = [<A> where T: AccountSetCleanup<'info, A>], arg = A)]
pub struct Signer<T>(
    #[decode(arg = arg)]
    #[validate(arg = arg)]
    #[cleanup(arg = arg)]
    #[single_account_set(skip_signed_account)]
    pub(crate) T,
);

pub type SignerInfo<'info> = Signer<AccountInfo<'info>>;

impl<'info, T> SignedAccount<'info> for Signer<T>
where
    T: SingleAccountSet<'info>,
{
    fn signer_seeds(&self) -> Option<Vec<&[u8]>> {
        None
    }
}

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
