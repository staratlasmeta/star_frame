use crate::account_set::{
    AccountSet, AccountSetValidate, CanInitSeeds, SignedAccount, SingleAccountSet,
};

use crate::prelude::{SingleSetMeta, SyscallInvoke};
use crate::Result;
use derive_more::{Deref, DerefMut};
use pinocchio::account_info::AccountInfo;
use std::fmt::Debug;

#[derive(AccountSet, Copy, Clone, Debug, Deref, DerefMut)]
#[repr(transparent)]
#[account_set(skip_default_idl)]
#[validate(
    extra_validation = if SIGNER { self.check_signer() } else { Ok(()) }
)]
pub struct MaybeSigner<const SIGNER: bool, T>(
    #[single_account_set(meta = SingleSetMeta { signer: SIGNER, ..T::meta() }, skip_signed_account, skip_can_init_seeds)]
    pub(crate) T,
);

/// A signed account
pub type Signer<T> = MaybeSigner<true, T>;

pub type SignerInfo = Signer<AccountInfo>;

impl<T> SignedAccount for MaybeSigner<true, T>
where
    T: SingleAccountSet,
{
    fn signer_seeds(&self) -> Option<Vec<&[u8]>> {
        None
    }
}

// A false `MaybeSigner` just acts as a pass-through, so we need to pass this through!
impl<T> SignedAccount for MaybeSigner<false, T>
where
    T: SignedAccount,
{
    fn signer_seeds(&self) -> Option<Vec<&[u8]>> {
        self.0.signer_seeds()
    }
}

// `CanInitSeeds` on `Signer` is a no-op
impl<T, A> CanInitSeeds<A> for MaybeSigner<true, T>
where
    Self: SingleAccountSet + AccountSetValidate<A>,
{
    fn init_seeds(&mut self, _arg: &A, _syscalls: &impl SyscallInvoke) -> Result<()> {
        Ok(())
    }
}

// A false `MaybeSigner` just acts as a pass-through, so we need to pass this through!
impl<T, A> CanInitSeeds<A> for MaybeSigner<false, T>
where
    T: CanInitSeeds<A>,
{
    fn init_seeds(&mut self, arg: &A, syscalls: &impl SyscallInvoke) -> Result<()> {
        self.0.init_seeds(arg, syscalls)
    }
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use super::*;
    use star_frame::idl::AccountSetToIdl;
    use star_frame_idl::account_set::IdlAccountSetDef;
    use star_frame_idl::IdlDefinition;

    impl<const SIGNER: bool, T, A> AccountSetToIdl<A> for MaybeSigner<SIGNER, T>
    where
        T: AccountSetToIdl<A>,
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
