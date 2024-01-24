use crate::account_set::{
    AccountSet, AccountSetCleanup, AccountSetDecode, AccountSetValidate, SingleAccountSet,
};
use solana_program::account_info::AccountInfo;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;

// Structs
#[derive(Debug)]
// #[derive(AccountSet, Debug)]
// #[account_set(skip_default_idl, generics = [where T: AccountSet<'info>])]
// #[decode(generics = [<A> where T: AccountSetDecode<'a, 'info, A>], arg = A)]
// #[validate(
// generics = [<A> where T: AccountSetValidate<'info, A> + SingleAccountSet<'info>], arg = A,
// )]
// #[cleanup(generics = [<A> where T: AccountSetCleanup<'info, A>], arg = A)]
pub struct SeededAccount<T, S: Seeds> {
    // #[decode(arg = arg)]
    // #[validate(arg = arg)]
    // #[cleanup(arg = arg)]
    account: T,
    seeds: Option<SeedsWithBump<S>>,
}

// TODO - Macro tries to implement these for both `account` and `seeds` but it blows up because
// `SeedsWithBump` is not `AccountSet`
// the trait bound `seeds::SeedsWithBump<S>: account_set::AccountSet<'_>` is not satisfied

impl<'info, T, S> SingleAccountSet<'info> for SeededAccount<T, S>
where
    T: SingleAccountSet<'info>,
    S: Seeds,
{
    fn account_info(&self) -> &AccountInfo<'info> {
        self.account.account_info()
    }
}

#[derive(Debug)]
struct SeedsWithBump<T: Seeds> {
    seeds: T,
    bump: u8,
}

// Traits
pub trait Seeds {
    fn seeds(&self) -> Vec<&[u8]>;
}

pub trait Seed {
    fn seed(&self) -> &[u8];
}

// Implementations
impl<T> Seeds for T
where
    T: Seed,
{
    fn seeds(&self) -> Vec<&[u8]> {
        vec![self.seed()]
    }
}

impl<T> Seeds for SeedsWithBump<T>
where
    T: Seed,
{
    fn seeds(&self) -> Vec<&[u8]> {
        vec![self.seeds.seed()]
    }
}

impl<T> Seeds for &SeedsWithBump<T>
where
    T: Seed,
{
    fn seeds(&self) -> Vec<&[u8]> {
        vec![self.seeds.seed()]
    }
}

impl Seed for Pubkey {
    fn seed(&self) -> &[u8] {
        self.as_ref()
    }
}

#[automatically_derived]
impl<'info, T, S: Seeds> AccountSet<'info> for SeededAccount<T, S>
where
    T: AccountSet<'info>,
{
    fn try_to_accounts<'__a, __E>(
        &'__a self,
        mut add_account: impl FnMut(&'__a AccountInfo<'info>) -> star_frame::Result<(), __E>,
    ) -> star_frame::Result<(), __E>
    where
        'info: '__a,
    {
        <T as AccountSet<'info>>::try_to_accounts(&self.account, &mut add_account)?;
        Ok(())
    }
    fn to_accounts<'__a>(&'__a self, mut add_account: impl FnMut(&'__a AccountInfo<'info>))
    where
        'info: '__a,
    {
        <T as AccountSet<'info>>::to_accounts(&self.account, &mut add_account);
    }
    fn to_account_metas(
        &self,
        mut add_account_meta: impl FnMut(solana_program::instruction::AccountMeta),
    ) {
        <T as AccountSet<'info>>::to_account_metas(&self.account, &mut add_account_meta);
    }
}
#[automatically_derived]
impl<'info, 'a, T, S: Seeds, A> AccountSetDecode<'a, 'info, A> for SeededAccount<T, S>
where
    T: AccountSetDecode<'a, 'info, A>,
{
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo<'info>],
        decode_input: A,
        sys_calls: &mut impl star_frame::sys_calls::SysCallInvoke,
    ) -> crate::Result<Self> {
        Ok(Self {
            account: T::decode_accounts(accounts, decode_input, sys_calls)?,
            seeds: None,
        })
    }
}
#[automatically_derived]
impl<'info, T, S> AccountSetValidate<'info, SeedsWithBump<S>> for SeededAccount<T, S>
where
    T: AccountSetValidate<'info, S> + SingleAccountSet<'info>,
    S: Seeds,
{
    fn validate_accounts(
        &mut self,
        arg: SeedsWithBump<S>,
        _sys_calls: &mut impl star_frame::sys_calls::SysCallInvoke,
    ) -> star_frame::Result<()> {
        let arg_seeds = arg.seeds.seeds();
        let arg_bump = arg.bump;
        let (address, bump) = Pubkey::find_program_address(&arg_seeds, self.account_info().owner);
        if self.account.account_info().key != &address || arg_bump != bump {
            return Err(ProgramError::Custom(20));
        }
        self.seeds = Some(arg);
        Ok(())
    }
}
#[automatically_derived]
impl<'info, T, S: Seeds, A> AccountSetCleanup<'info, A> for SeededAccount<T, S>
where
    T: AccountSetCleanup<'info, A>,
{
    fn cleanup_accounts(
        &mut self,
        arg: A,
        sys_calls: &mut impl star_frame::sys_calls::SysCallInvoke,
    ) -> star_frame::Result<()> {
        <T as AccountSetCleanup<'info, _>>::cleanup_accounts(&mut self.account, arg, sys_calls)?;
        Ok(())
    }
}
