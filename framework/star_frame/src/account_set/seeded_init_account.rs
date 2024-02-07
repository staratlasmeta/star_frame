use crate::account_set::init_account::{Create, CreateIfNeeded, CreateSplit, InitCreateArg};
use crate::account_set::SignedAccount;
use crate::prelude::*;
use crate::serialize::FrameworkInit;
use derivative::Derivative;
use star_frame_proc::AccountSet;

#[derive(AccountSet, Derivative)]
#[derivative(
    Debug(bound = "SeededAccount<InitAccount<'info, T>, T::Seeds>: Debug"),
    Copy(bound = "SeededAccount<InitAccount<'info, T>, T::Seeds>: Copy"),
    Clone(bound = "SeededAccount<InitAccount<'info, T>, T::Seeds>: Clone")
)]
#[account_set(skip_default_validate)]
#[validate(
    id = "create",
    generics = [
        <IC>
        where
            IC: InitCreateArg<'info>,
            T: SingleAccountSet<'info> + FrameworkInit<IC::FrameworkInitArg>,
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
            T: SingleAccountSet<'info> + FrameworkInit<IC::FrameworkInitArg>,
    ],
    arg = CreateIfNeeded<SeededInit<T::Seeds, IC>>,
    extra_validation = seed_init_validate_if_needed(self, arg.0, sys_calls)
)]
#[cleanup(
    generics = [<A> where SeededAccount<InitAccount<'info, T>, T::Seeds>: AccountSetCleanup<'info, A>],
    arg = A,
)]
pub struct SeededInitAccount<'info, T>(
    #[validate(id = "create", skip)]
    #[validate(id = "create_if_needed", skip)]
    #[cleanup(arg = arg)]
    SeededAccount<InitAccount<'info, T>, T::Seeds>,
)
where
    T: SeededAccountData + UnsizedType + ?Sized;

impl<'info, T> SingleAccountSet<'info> for SeededInitAccount<'info, T>
where
    T: SeededAccountData + UnsizedType,
{
    fn account_info(&self) -> &AccountInfo<'info> {
        self.0.account_info()
    }
}
impl<'info, T> SignedAccount<'info> for SeededInitAccount<'info, T>
where
    T: SeededAccountData + UnsizedType,
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
    type FunderSeeds = IC::FunderSeeds;

    fn system_program(&self) -> &Program<'info, SystemProgram> {
        self.init_arg.system_program()
    }

    fn split<'b>(
        &'b mut self,
    ) -> CreateSplit<
        'b,
        'info,
        Self::FrameworkInitArg,
        Self::AccountSeeds,
        Self::FunderAccount,
        Self::FunderSeeds,
    > {
        let split = self.init_arg.split();
        CreateSplit {
            arg: split.arg,
            account_seeds: Some(self.seeds),
            system_program: split.system_program,
            funder: split.funder,
        }
    }
}

fn seed_init_validate<'info, T, IC>(
    account: &mut SeededInitAccount<'info, T>,
    arg: SeededInit<T::Seeds, IC>,
    sys_calls: &mut impl SysCallInvoke,
) -> Result<()>
where
    T: SeededAccountData + UnsizedType + FrameworkInit<IC::FrameworkInitArg> + ?Sized,
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

fn seed_init_validate_if_needed<'info, T, IC>(
    account: &mut SeededInitAccount<'info, T>,
    arg: SeededInit<T::Seeds, IC>,
    sys_calls: &mut impl SysCallInvoke,
) -> Result<()>
where
    T: SeededAccountData + UnsizedType + FrameworkInit<IC::FrameworkInitArg> + ?Sized,
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
