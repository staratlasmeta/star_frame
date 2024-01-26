use crate::account_set::data_account::AccountData;
use crate::account_set::{
    AccountSet, AccountSetCleanup, AccountSetDecode, AccountSetValidate, SingleAccountSet,
};
use crate::sys_calls::SysCallInvoke;
use crate::Result;
use bytemuck::Pod;
use solana_program::account_info::AccountInfo;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use star_frame::account_set::data_account::DataAccount;
use std::ops::{Deref, DerefMut};

pub trait GetSeeds {
    fn seeds(&self) -> Vec<&[u8]>;
}

impl<T> GetSeeds for T
where
    T: Seed,
{
    fn seeds(&self) -> Vec<&[u8]> {
        vec![self.seed()]
    }
}
impl<T> GetSeeds for SeedsWithBump<T>
where
    T: GetSeeds,
{
    fn seeds(&self) -> Vec<&[u8]> {
        self.seeds.seeds()
    }
}
impl<T> GetSeeds for &SeedsWithBump<T>
where
    T: GetSeeds,
{
    fn seeds(&self) -> Vec<&[u8]> {
        self.seeds.seeds()
    }
}

pub trait Seed {
    fn seed(&self) -> &[u8];
}

impl<T> Seed for T
where
    T: Pod,
{
    fn seed(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

#[derive(Debug)]
pub struct SeedsWithBump<T: GetSeeds> {
    pub seeds: T,
    pub bump: u8,
}

#[derive(Debug)]
pub struct Seeds<T>(pub T);

// Structs
#[derive(Debug)]
pub struct SeededAccount<T, S: GetSeeds> {
    account: T,
    // #[account_set(skip)] - TODO - Make this a thing
    seeds: Option<SeedsWithBump<S>>,
}

impl<T, S: GetSeeds> SeededAccount<T, S> {
    pub fn access_seeds(&self) -> &SeedsWithBump<S> {
        self.seeds.as_ref().unwrap()
    }
}

// TODO - Macro tries to implement these for both `account` and `seeds` but it blows up because
// `SeedsWithBump` is not `AccountSet`
// the trait bound `seeds::SeedsWithBump<S>: account_set::AccountSet<'_>` is not satisfied

impl<'info, T, S> SingleAccountSet<'info> for SeededAccount<T, S>
where
    T: SingleAccountSet<'info>,
    S: GetSeeds,
{
    fn account_info(&self) -> &AccountInfo<'info> {
        self.account.account_info()
    }
}

// Implementations
#[automatically_derived]
impl<'info, T, S: GetSeeds> AccountSet<'info> for SeededAccount<T, S>
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
impl<'info, 'a, T, S: GetSeeds, A> AccountSetDecode<'a, 'info, A> for SeededAccount<T, S>
where
    T: AccountSetDecode<'a, 'info, A>,
{
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo<'info>],
        decode_input: A,
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<Self> {
        Ok(Self {
            account: T::decode_accounts(accounts, decode_input, sys_calls)?,
            seeds: None,
        })
    }
}
#[automatically_derived]
impl<'info, T, S, A> AccountSetValidate<'info, (SeedsWithBump<S>, A)> for SeededAccount<T, S>
where
    T: AccountSetValidate<'info, A> + SingleAccountSet<'info>,
    S: GetSeeds,
{
    fn validate_accounts(
        &mut self,
        arg: (SeedsWithBump<S>, A),
        _sys_calls: &mut impl SysCallInvoke,
    ) -> Result<()> {
        <T as AccountSetValidate<'info, _>>::validate_accounts(
            &mut self.account,
            arg.1,
            _sys_calls,
        )?;
        let arg_seeds = arg.0.seeds.seeds();
        let arg_bump = arg.0.bump;
        let (address, bump) = Pubkey::find_program_address(&arg_seeds, self.account_info().owner);
        if self.account.account_info().key != &address || arg_bump != bump {
            return Err(ProgramError::Custom(20));
        }
        self.seeds = Some(arg.0);
        Ok(())
    }
}
#[automatically_derived]
impl<'info, T, A, S> AccountSetValidate<'info, (S, A)> for SeededAccount<T, S>
where
    T: AccountSetValidate<'info, A> + SingleAccountSet<'info>,
    S: GetSeeds,
{
    fn validate_accounts(
        &mut self,
        validate_input: (S, A),
        _sys_calls: &mut impl SysCallInvoke,
    ) -> Result<()> {
        <T as AccountSetValidate<'info, _>>::validate_accounts(
            &mut self.account,
            validate_input.1,
            _sys_calls,
        )?;
        let (address, _bump) =
            Pubkey::find_program_address(&validate_input.0.seeds(), self.account_info().owner);
        if self.account.account_info().key != &address {
            return Err(ProgramError::Custom(20));
        }
        Ok(())
    }
}
#[automatically_derived]
impl<'info, T, S: GetSeeds, A> AccountSetCleanup<'info, A> for SeededAccount<T, S>
where
    T: AccountSetCleanup<'info, A>,
{
    fn cleanup_accounts(&mut self, arg: A, sys_calls: &mut impl SysCallInvoke) -> Result<()> {
        <T as AccountSetCleanup<'info, _>>::cleanup_accounts(&mut self.account, arg, sys_calls)?;
        Ok(())
    }
}
impl<T, S: GetSeeds> Deref for SeededAccount<T, S> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.account
    }
}
impl<T, S: GetSeeds> DerefMut for SeededAccount<T, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.account
    }
}
#[cfg(feature = "idl")]
mod idl_impl {
    use super::*;
    use star_frame::idl::AccountSetToIdl;
    use star_frame_idl::account_set::IdlAccountSetDef;
    use star_frame_idl::IdlDefinition;

    impl<'info, T, A, S> AccountSetToIdl<'info, A> for SeededAccount<T, S>
    where
        T: AccountSetToIdl<'info, A> + SingleAccountSet<'info>,
        S: GetSeeds,
    {
        fn account_set_to_idl(
            idl_definition: &mut IdlDefinition,
            arg: A,
        ) -> Result<IdlAccountSetDef> {
            T::account_set_to_idl(idl_definition, arg)
                .map(Box::new)
                .map(IdlAccountSetDef::SeededAccount)
        }
    }
}

pub trait SeededAccountData: AccountData {
    type Seeds: GetSeeds;
}

#[derive(AccountSet, Debug)]
#[validate(arg = (T::Seeds,))]
#[validate(id="wo_bump", arg = Seeds<T::Seeds>)]
#[validate(id="with_bump", arg = SeedsWithBump<T::Seeds>)]
pub struct SeededDataAccount<'info, T>(
    #[validate(arg = (arg.0, ()))]
    #[validate(id="wo_bump", arg = (arg.0, ()))]
    #[validate(id="with_bump", arg = (arg, ()))]
    SeededAccount<DataAccount<'info, T>, T::Seeds>,
)
where
    T: SeededAccountData;
