use crate::prelude::*;
use crate::unsize::UnsizedType;
use derive_more::{Deref, DerefMut};

pub trait AccountValidate<ValidateArg>: UnsizedType {
    fn validate(self_ref: &Self::Ref<'_>, arg: ValidateArg) -> Result<()>;
}

#[derive(AccountSet, Debug, Deref, DerefMut, derive_where::DeriveWhere)]
#[derive_where(Clone)]
#[validate(generics = [<ValidateArg> where T: AccountValidate<ValidateArg>], arg = ValidateArg, extra_validation = T::validate(&*self.account.data()?, arg))]
#[idl(generics = [<A> where T: AccountToIdl, Account<'info, T>: AccountSetToIdl<'info, A>], arg = A)]
pub struct ValidatedAccount<'info, T>
where
    T: ProgramAccount + UnsizedType + ?Sized,
{
    #[single_account_set]
    #[idl(arg = arg)]
    account: Account<'info, T>,
}
