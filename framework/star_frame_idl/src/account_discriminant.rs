use crate::DiscriminantId;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::num::{NonZeroU128, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8};

pub mod _serde_account_discriminant {

    use super::{AccountDiscriminant, DiscriminantId};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer, D: AccountDiscriminant>(
        val: &D,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        val.discriminant().serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>, AD: AccountDiscriminant>(
        deserializer: D,
    ) -> Result<AD, D::Error> {
        let discriminant = DiscriminantId::deserialize(deserializer)?;
        AD::from_discriminant(&discriminant).ok_or_else(move || {
            serde::de::Error::custom(format!("Invalid discriminant value: {:?}", discriminant))
        })
    }
}

pub trait AccountDiscriminant: Sized {
    type DiscriminantValue: Serialize + DeserializeOwned;

    fn discriminant(&self) -> DiscriminantId;
    fn from_discriminant(discriminant: &DiscriminantId) -> Option<Self>;
}

macro_rules! impl_account_discriminant {
    (@impl $ty:ident: $ty_val:ty) => {
        #[derive(Copy, Clone, Debug, Eq, PartialEq)]
        pub struct $ty;
        impl AccountDiscriminant for $ty {
            type DiscriminantValue = $ty_val;

            fn discriminant(&self) -> DiscriminantId{
                DiscriminantId::$ty
            }
            fn from_discriminant(discriminant: &DiscriminantId) -> Option<Self>{
                match discriminant {
                    DiscriminantId::$ty => Some(Self),
                    _ => None,
                }
            }
        }
    };
    ($($ty:ident: $ty_val:ty),* $(,)?) => {
        $(
            impl_account_discriminant!(@impl $ty: $ty_val);
        )*
    };
}
impl_account_discriminant!(
    U8: NonZeroU8,
    U16: NonZeroU16,
    U32: NonZeroU32,
    U64: NonZeroU64,
    U128: NonZeroU128,
    Anchor: [u8; 8],
);
