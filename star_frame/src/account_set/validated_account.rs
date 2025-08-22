use crate::prelude::*;
use derive_more::{Deref, DerefMut};

pub trait AccountValidate<ValidateArg>: UnsizedType {
    fn validate(self_ref: &Self::Ref<'_>, arg: ValidateArg) -> Result<()>;
}

#[derive(AccountSet, Debug, Deref, DerefMut, derive_where::DeriveWhere)]
#[derive_where(Clone)]
#[validate(generics = [<ValidateArg> where T: AccountValidate<ValidateArg>], arg = ValidateArg, extra_validation = T::validate(&*self.account.data()?, arg))]
#[idl(generics = [<A> where T: AccountToIdl, Account<T>: AccountSetToIdl<A>], arg = A)]
pub struct ValidatedAccount<T>
where
    T: ProgramAccount + UnsizedType + ?Sized,
{
    #[single_account_set]
    #[idl(arg = arg)]
    account: Account<T>,
}

macro_rules! account_validate_tuple {
    ($($idents:ident)*) => {
        account_validate_tuple!(| $($idents)*);
    };
    ($($idents:ident)* |) => {};
    ($($initial:ident)* | $($after:ident $($last:ident)*)?) => {
        account_validate_tuple!(inner: $($initial)* $($after)*);
        account_validate_tuple!($($initial)* $($after)* | $($($last)*)?);
    };
    (inner: $($generic:ident)*) => {
        star_frame::paste::paste!{
            impl<Acc, $($generic,)*> star_frame::prelude::AccountValidate<($($generic,)*)> for Acc
            where
                $(Acc: star_frame::prelude::AccountValidate<$generic>),*
            {
                fn validate(self_ref: &Self::Ref<'_>, arg: ($($generic,)*)) -> star_frame::prelude::Result<()> {
                    let ($([<$generic:snake>],)*) = arg;
                    $(
                        <Acc as star_frame::prelude::AccountValidate<$generic>>::validate(self_ref, [<$generic:snake>])?;
                    )*
                    Ok(())
                }
            }
        }
    }
}

account_validate_tuple!(A B C D E F G H I J K L M N O P);
