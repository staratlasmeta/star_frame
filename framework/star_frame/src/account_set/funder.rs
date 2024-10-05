use itertools::Itertools;
use star_frame::prelude::*;

/// A (mostly) internal helper [`AccountSet`] used for the [`SyscallAccountCache`] funder. This intentionally does not
/// implement any of the `AccountSet` lifecycle traits.
#[derive(AccountSet, Debug, Clone)]
#[account_set(
    skip_default_decode,
    skip_default_validate,
    skip_default_cleanup,
    skip_default_idl
)]
pub struct Funder<'info> {
    #[single_account_set(metadata = SingleAccountSetMetadata {
        should_sign: true,
        should_mut: true,
        ..SingleAccountSetMetadata::DEFAULT
    }, skip_signed_account)]
    inner: Writable<SignerInfo<'info>>,
    #[account_set(skip = None)]
    seeds: Option<Vec<Vec<u8>>>,
}

impl<'info> Funder<'info> {
    pub(crate) fn new(account: &(impl WritableAccount<'info> + SignedAccount<'info>)) -> Self {
        let inner = Writable(Signer(account.account_info_cloned()));
        let seeds = account
            .signer_seeds()
            .map(|seeds| seeds.iter().map(|s| s.to_vec()).collect_vec());
        Self { inner, seeds }
    }
}

impl<'info> SignedAccount<'info> for Funder<'info> {
    fn signer_seeds(&self) -> Option<Vec<&[u8]>> {
        self.seeds
            .as_ref()
            .map(|seeds| seeds.iter().map(Vec::as_slice).collect())
    }
}
