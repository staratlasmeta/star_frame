use solana_program::account_info::AccountInfo;
use solana_program::program_error::ProgramError;
use solana_program::system_program;
use star_frame::account_set::{
    AccountSet, AccountSetCleanup, AccountSetDecode, AccountSetValidate, SingleAccountSet,
};
use star_frame::idl::AccountSetToIdl;
use std::marker::PhantomData;

#[derive(AccountSet, Debug)]
#[decode(generics = [<A> where T: AccountSetDecode<'a, 'info, A>], arg = A)]
#[validate(
    generics = [<A> where T: SingleAccountSet<'info> + AccountSetValidate<'info, A>],
    arg = A,
    extra_validation = if self.0.owner() != &system_program::ID { Err(ProgramError::IllegalOwner) } else { Ok(()) },
)]
#[cleanup(generics = [<A> where T: AccountSetCleanup<'info, A>], arg = A)]
pub struct SystemAccount<'info, T = AccountInfo<'info>>(
    #[decode(arg = arg)]
    #[validate(arg = arg)]
    #[cleanup(arg = arg)]
    T,
    PhantomData<&'info ()>,
)
where
    T: AccountSet<'info>,
    T: AccountSetToIdl<'info, ()>;
