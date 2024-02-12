use crate::account_set::seeded_account::{CurrentProgram, SeedProgram};
use crate::account_set::SignedAccount;
use crate::prelude::*;
use derive_more::{Deref, DerefMut};

pub trait SeededAccountData: ProgramAccount {
    type Seeds: GetSeeds;
}

#[derive(AccountSet, Debug, Deref, DerefMut)]
#[validate(arg = (T::Seeds,))]
#[validate(id = "wo_bump", arg = Seeds < T::Seeds >)]
#[validate(id = "with_bump", arg = SeedsWithBump < T::Seeds >)]
#[cleanup(
    generics = [<A> where SeededAccount<DataAccount<'info, T>, T::Seeds, P>: AccountSetCleanup<'cleanup, 'info, A>],
    arg = A,
)]
pub struct SeededDataAccount<'info, T, P: SeedProgram = CurrentProgram>(
    #[validate(arg = (arg.0, ()))]
    #[validate(id = "wo_bump", arg = (arg.0, ()))]
    #[validate(id = "with_bump", arg = (arg, ()))]
    #[cleanup(arg = arg)]
    SeededAccount<DataAccount<'info, T>, T::Seeds, P>,
)
where
    T: SeededAccountData + UnsizedType + ?Sized;

impl<'info, T, P: SeedProgram> SingleAccountSet<'info> for SeededDataAccount<'info, T, P>
where
    T: SeededAccountData + UnsizedType + ?Sized,
{
    fn account_info(&self) -> &AccountInfo<'info> {
        self.0.account_info()
    }
}
impl<'info, T, P: SeedProgram> SignedAccount<'info> for SeededDataAccount<'info, T, P>
where
    T: SeededAccountData + UnsizedType + ?Sized,
    SeededAccount<DataAccount<'info, T>, T::Seeds, P>: SignedAccount<'info>,
{
    fn signer_seeds(&self) -> Option<Vec<&[u8]>> {
        self.0.signer_seeds()
    }
}
