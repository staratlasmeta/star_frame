use crate::account_set::{
    AccountSet, AccountSetValidate, CanInitSeeds, SignedAccount, SingleAccountSet,
};

use crate::prelude::{SingleSetMeta, SyscallInvoke};
use crate::Result;
use derive_more::{Deref, DerefMut};
use solana_program::account_info::AccountInfo;
use std::fmt::Debug;

#[derive(AccountSet, Copy, Clone, Debug, Deref, DerefMut)]
#[repr(transparent)]
#[account_set(skip_default_idl)]
#[validate(
    extra_validation = if SIGNER { self.check_signer() } else { Ok(()) }
)]
pub struct MaybeSigner<const SIGNER: bool, T>(
    #[single_account_set(meta = SingleSetMeta { signer: SIGNER, ..T::META}, skip_signed_account, skip_can_init_seeds)]
    pub(crate) T,
);

/// A signed account
pub type Signer<T> = MaybeSigner<true, T>;

pub type SignerInfo<'info> = Signer<AccountInfo<'info>>;

impl<'info, T> SignedAccount<'info> for MaybeSigner<true, T>
where
    T: SingleAccountSet<'info>,
{
    fn signer_seeds(&self) -> Option<Vec<&[u8]>> {
        None
    }
}

// A false `MaybeSigner` just acts as a pass-through, so we need to pass this through!
impl<'info, T> SignedAccount<'info> for MaybeSigner<false, T>
where
    T: SignedAccount<'info>,
{
    fn signer_seeds(&self) -> Option<Vec<&[u8]>> {
        self.0.signer_seeds()
    }
}

// `CanInitSeeds` on `Signer` is a no-op
impl<'info, T, A> CanInitSeeds<'info, A> for MaybeSigner<true, T>
where
    Self: SingleAccountSet<'info> + AccountSetValidate<'info, A>,
{
    fn init_seeds(&mut self, _arg: &A, _syscalls: &impl SyscallInvoke<'info>) -> Result<()> {
        Ok(())
    }
}

// A false `MaybeSigner` just acts as a pass-through, so we need to pass this through!
impl<'info, T, A> CanInitSeeds<'info, A> for MaybeSigner<false, T>
where
    T: CanInitSeeds<'info, A>,
{
    fn init_seeds(&mut self, arg: &A, syscalls: &impl SyscallInvoke<'info>) -> Result<()> {
        self.0.init_seeds(arg, syscalls)
    }
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use super::*;
    use star_frame::idl::AccountSetToIdl;
    use star_frame_idl::account_set::IdlAccountSetDef;
    use star_frame_idl::IdlDefinition;

    impl<'info, const SIGNER: bool, T, A> AccountSetToIdl<'info, A> for MaybeSigner<SIGNER, T>
    where
        T: AccountSetToIdl<'info, A>,
    {
        fn account_set_to_idl(
            idl_definition: &mut IdlDefinition,
            arg: A,
        ) -> Result<IdlAccountSetDef> {
            let mut set = T::account_set_to_idl(idl_definition, arg)?;
            if SIGNER {
                set.single()?.signer = true;
            }
            Ok(set)
        }
    }
}
