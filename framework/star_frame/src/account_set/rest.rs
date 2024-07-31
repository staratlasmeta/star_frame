#[cfg(feature = "idl")]
use crate::account_set::impls::vec::idl_impl::VecSize;
use crate::account_set::{AccountSet, AccountSetCleanup, AccountSetDecode, AccountSetValidate};
#[cfg(feature = "idl")]
use crate::idl::AccountSetToIdl;
use crate::syscalls::SyscallInvoke;
use derive_more::{Deref, DerefMut};
use solana_program::account_info::AccountInfo;

#[derive(AccountSet, Debug, Deref, DerefMut)]
#[account_set(skip_default_decode, generics = [where T: AccountSet<'info>])]
#[validate(generics = [<A> where T: AccountSetValidate<'info, A>, A: Clone], arg = A)]
#[cleanup(generics = [<A> where T: AccountSetCleanup<'info, A>, A: Clone], arg = A)]
#[cfg_attr(feature = "idl", idl(generics = [<A> where T: AccountSetToIdl<'info, A>, A: Clone], arg = A))]
pub struct Rest<T>(
    #[validate(arg = (arg,))]
    #[cleanup(arg = (arg,))]
    #[idl(arg = (VecSize{ min: 0, max: None }, arg))]
    Vec<T>,
);

impl<'a, 'info, A, T> AccountSetDecode<'a, 'info, A> for Rest<T>
where
    T: AccountSetDecode<'a, 'info, A>,
    A: Clone,
{
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo<'info>],
        decode_input: A,
        syscalls: &mut impl SyscallInvoke,
    ) -> crate::Result<Self> {
        let mut out = vec![];
        while !accounts.is_empty() {
            out.push(T::decode_accounts(
                accounts,
                decode_input.clone(),
                syscalls,
            )?);
        }
        Ok(Self(out))
    }
}
