//! Account modifier for signature validation and signer requirements.
//!
//! The `Signer<T>` modifier wraps accounts that must be signed by their corresponding private key.
//! It validates that the account was included as a signer in the transaction.

use crate::{
    account_set::{
        modifiers::{CanInitSeeds, SignedAccount},
        single_set::SingleSetMeta,
        AccountSetValidate,
    },
    prelude::*,
};
use derive_more::{Deref, DerefMut};

/// A conditionally signed account modifier that validates signature requirements.
///
/// This type wraps another account type and adds signature validation when `SIGNER` is true.
/// It's primarily used through the `Signer<T>` type alias which sets `SIGNER` to true, ensuring
/// the wrapped account must be signed in the transaction.
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
pub type Signer<T = AccountInfo> = MaybeSigner<true, T>;

impl<T> SignedAccount for MaybeSigner<true, T>
where
    T: SingleAccountSet,
{
    #[inline]
    fn signer_seeds(&self) -> Option<Vec<&[u8]>> {
        None
    }
}

// A false `MaybeSigner` just acts as a pass-through, so we need to pass this through!
impl<T> SignedAccount for MaybeSigner<false, T>
where
    T: SignedAccount,
{
    #[inline]
    fn signer_seeds(&self) -> Option<Vec<&[u8]>> {
        self.0.signer_seeds()
    }
}

// `CanInitSeeds` on `Signer` is a no-op
impl<T, A> CanInitSeeds<A> for MaybeSigner<true, T>
where
    Self: SingleAccountSet + AccountSetValidate<A>,
{
    #[inline]
    fn init_seeds(&mut self, _arg: &A, _ctx: &Context) -> Result<()> {
        Ok(())
    }
}

// A false `MaybeSigner` just acts as a pass-through, so we need to pass this through!
impl<T, A> CanInitSeeds<A> for MaybeSigner<false, T>
where
    T: CanInitSeeds<A>,
{
    #[inline]
    fn init_seeds(&mut self, arg: &A, ctx: &Context) -> Result<()> {
        self.0.init_seeds(arg, ctx)
    }
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use super::*;
    use star_frame::idl::AccountSetToIdl;
    use star_frame_idl::{account_set::IdlAccountSetDef, IdlDefinition};

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
