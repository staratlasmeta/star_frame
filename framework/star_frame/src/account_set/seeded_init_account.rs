use crate::account_set::init_account::CreateSplit;
use crate::account_set::init_account::InitCreateArg;
use crate::account_set::seeded_account::SeedProgram;
use crate::account_set::seeded_account::Skip;
use crate::account_set::SignedAccount;
use crate::prelude::*;
use derivative::Derivative;
use derive_more::{Deref, DerefMut};
use star_frame::account_set::seeded_account::CurrentProgram;
use star_frame_proc::AccountSet;

#[derive(AccountSet, Derivative, Deref, DerefMut)]
#[derivative(
    Debug(bound = "SeededAccount<InitAccount<'info, T>, T::Seeds, P>: Debug"),
    Copy(bound = "SeededAccount<InitAccount<'info, T>, T::Seeds, P>: Copy"),
    Clone(bound = "SeededAccount<InitAccount<'info, T>, T::Seeds, P>: Clone")
)]
#[account_set(skip_default_validate)]
#[validate(
    id = "create",
    generics = [
        <IC>
        where
            IC: InitCreateArg<'info>,
            T: UnsizedInit<IC::StarFrameInitArg>,
    ],
    arg = Create<SeededInit<T::Seeds, IC>>,
    extra_validation = seed_init_validate(self, arg.0, sys_calls)
)]
#[validate(
    id = "create_if_needed",
    generics = [
        <IC>
        where
            IC: InitCreateArg<'info>,
            T: UnsizedInit<IC::StarFrameInitArg>,
    ],
    arg = CreateIfNeeded<SeededInit<T::Seeds, IC>>,
    extra_validation = seed_init_validate_if_needed(self, arg.0, sys_calls)
)]
#[cleanup(
    generics = [<A> where SeededAccount<InitAccount<'info, T>, T::Seeds, P>: AccountSetCleanup<'info, A>],
    arg = A,
)]
pub struct SeededInitAccount<'info, T, P: SeedProgram = CurrentProgram>(
    #[validate(id = "create", skip)]
    #[validate(id = "create_if_needed", skip)]
    #[cleanup(arg = arg)]
    SeededAccount<InitAccount<'info, T>, T::Seeds, P>,
)
where
    T: SeededAccountData + UnsizedType + ?Sized;

impl<'info, T, P: SeedProgram> SingleAccountSet<'info> for SeededInitAccount<'info, T, P>
where
    T: SeededAccountData + UnsizedType + ?Sized,
{
    fn account_info(&self) -> &AccountInfo<'info> {
        self.0.account_info()
    }
}
impl<'info, T, P: SeedProgram> SignedAccount<'info> for SeededInitAccount<'info, T, P>
where
    T: SeededAccountData + UnsizedType + ?Sized,
    SeededAccount<InitAccount<'info, T>, T::Seeds, P>: SignedAccount<'info>,
{
    fn signer_seeds(&self) -> Option<Vec<&[u8]>> {
        self.0.signer_seeds()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct SeededInit<S, IC> {
    pub seeds: S,
    pub init_create: IC,
}

struct SeededInitArg<'a, S, IC>
where
    S: GetSeeds,
{
    seeds: &'a SeedsWithBump<S>,
    init_arg: IC,
}
impl<'a, 'info, S, IC> InitCreateArg<'info> for SeededInitArg<'a, S, IC>
where
    S: GetSeeds,
    IC: InitCreateArg<'info>,
{
    type FrameworkInitArg = IC::FrameworkInitArg;
    type AccountSeeds = S;
    type FunderAccount = IC::FunderAccount;

    fn system_program(&self) -> &Program<'info, SystemProgram> {
        self.init_arg.system_program()
    }

    fn split<'b>(
        &'b mut self,
    ) -> CreateSplit<'b, 'info, Self::FrameworkInitArg, Self::AccountSeeds, Self::FunderAccount>
    {
        let split = self.init_arg.split();
        CreateSplit {
            arg: split.arg,
            account_seeds: Some(self.seeds),
            system_program: split.system_program,
            funder: split.funder,
        }
    }
}

fn seed_init_validate<'info, T, IC, P: SeedProgram>(
    account: &mut SeededInitAccount<'info, T, P>,
    arg: SeededInit<T::Seeds, IC>,
    sys_calls: &mut impl SysCallInvoke,
) -> Result<()>
where
    T: SeededAccountData + UnsizedType + UnsizedInit<IC::FrameworkInitArg> + ?Sized,
    IC: InitCreateArg<'info>,
{
    SeededAccount::validate_accounts(&mut account.0, (Skip, Seeds(arg.seeds)), sys_calls)?;
    InitAccount::validate_accounts(
        &mut account.0.account,
        Create(SeededInitArg {
            seeds: account.0.seeds.as_ref().unwrap(),
            init_arg: arg.init_create,
        }),
        sys_calls,
    )?;

    Ok(())
}

fn seed_init_validate_if_needed<'info, T, IC, P: SeedProgram>(
    account: &mut SeededInitAccount<'info, T, P>,
    arg: SeededInit<T::Seeds, IC>,
    sys_calls: &mut impl SysCallInvoke,
) -> Result<()>
where
    T: SeededAccountData + UnsizedType + UnsizedInit<IC::FrameworkInitArg> + ?Sized,
    IC: InitCreateArg<'info>,
{
    SeededAccount::validate_accounts(&mut account.0, (Skip, Seeds(arg.seeds)), sys_calls)?;
    InitAccount::validate_accounts(
        &mut account.0.account,
        CreateIfNeeded(SeededInitArg {
            seeds: account.0.seeds.as_ref().unwrap(),
            init_arg: arg.init_create,
        }),
        sys_calls,
    )?;

    Ok(())
}
