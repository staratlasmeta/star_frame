use crate::account_set::{
    AccountSet, AccountSetValidate, CanInitSeeds, SignedAccount, SingleAccountSet,
};

use crate::prelude::SyscallInvoke;
use crate::Result;
use derive_more::{Deref, DerefMut};
use solana_program::account_info::AccountInfo;
use std::fmt::Debug;

#[derive(AccountSet, Copy, Clone, Debug, Deref, DerefMut)]
#[repr(transparent)]
#[account_set(skip_default_idl)]
#[validate(
    extra_validation = self.check_signer(),
)]
pub struct Signer<T>(
    #[single_account_set(signer, skip_signed_account, skip_can_init_seeds)] pub(crate) T,
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

// CanInitSeeds on Signer is a no-op
impl<'info, T, A> CanInitSeeds<'info, A> for Signer<T>
where
    Self: SingleAccountSet<'info> + AccountSetValidate<'info, A>,
{
    fn init_seeds(&mut self, _arg: &A, _syscalls: &impl SyscallInvoke<'info>) -> Result<()> {
        Ok(())
    }
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
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
            let mut set = T::account_set_to_idl(idl_definition, arg)?;
            set.single()?.signer = true;
            Ok(set)
        }
    }
}
