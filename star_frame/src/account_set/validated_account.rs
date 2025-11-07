//! Account wrapper with custom validation logic.
//!
//! The `ValidatedAccount<T>` type extends the basic `Account<T>` wrapper with additional
//! custom validation that runs during the account validation phase. This allows account
//! types to implement domain-specific validation rules beyond the standard owner and
//! discriminant checks.

use crate::prelude::*;
use derive_more::{Deref, DerefMut};

pub trait AccountValidate<ValidateArg>: UnsizedType {
    fn validate_account(self_ref: &Self::Ptr, arg: ValidateArg) -> Result<()>;
}

/// An account wrapper that performs additional custom validation during the validation phase.
///
/// This type wraps an `Account<T>` and adds an extra validation step that calls the account's
/// `AccountValidate::validate_account` method with the provided validation arguments. This is
/// useful for accounts that need domain-specific validation beyond owner and discriminant checks.
#[derive(AccountSet, Debug, Deref, DerefMut, derive_where::DeriveWhere)]
#[derive_where(Clone)]
#[validate(generics = [<ValidateArg> where T: AccountValidate<ValidateArg>], arg = ValidateArg, extra_validation = T::validate_account(&*self.account.data()?, arg))]
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
                fn validate_account(self_ref: &Self::Ptr, arg: ($($generic,)*)) -> star_frame::prelude::Result<()> {
                    let ($([<$generic:snake>],)*) = arg;
                    $(
                        <Acc as star_frame::prelude::AccountValidate<$generic>>::validate_account(self_ref, [<$generic:snake>])?;
                    )*
                    Ok(())
                }
            }
        }
    }
}

account_validate_tuple!(A B C D E F G H I J K L M N O P);
