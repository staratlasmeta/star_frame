use crate::account_set::{AccountSet, AccountSetCleanup, AccountSetDecode, AccountSetValidate};
#[cfg(feature = "idl")]
use crate::idl::AccountSetToIdl;
#[cfg(feature = "idl")]
use crate::impls::vec::idl_impl::VecSize;
use crate::sys_calls::SysCallInvoke;
use solana_program::account_info::AccountInfo;
use std::marker::PhantomData;

#[derive(AccountSet)]
#[account_set(skip_default_decode)]
#[validate(generics = [<A> where T: AccountSetValidate<'info, A>, A: Clone], arg = A)]
#[cleanup(generics = [<A> where T: AccountSetCleanup<'info, A>, A: Clone], arg = A)]
#[cfg_attr(feature = "idl", idl(generics = [<A> where T: AccountSetToIdl<'info, A>, A: Clone], arg = A))]
pub struct Rest<'info, T>(
    #[validate(arg = (arg,))]
    #[cleanup(arg = (arg,))]
    #[idl(arg = (VecSize{ min: 0, max: None }, arg))]
    Vec<T>,
    PhantomData<&'info ()>,
)
where
    T: AccountSet<'info>;
impl<'a, 'info, A, T> AccountSetDecode<'a, 'info, A> for Rest<'info, T>
where
    T: AccountSetDecode<'a, 'info, A>,
    A: Clone,
{
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo<'info>],
        decode_input: A,
        sys_calls: &mut impl SysCallInvoke,
    ) -> crate::Result<Self> {
        let mut out = vec![];
        while !accounts.is_empty() {
            out.push(T::decode_accounts(
                accounts,
                decode_input.clone(),
                sys_calls,
            )?);
        }
        Ok(Self(out, PhantomData))
    }
}
