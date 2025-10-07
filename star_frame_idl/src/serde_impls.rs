use std::collections::HashMap;

pub mod serde_base58_address {
    use serde::Deserialize;
    use solana_address::Address;
    use std::str::FromStr;

    pub fn serialize<S>(val: &Address, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&val.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Address, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Address::from_str(&s).map_err(serde::de::Error::custom)
    }
}

pub mod serde_base58_address_option {
    use serde::Deserialize;
    use solana_address::Address;
    use std::str::FromStr;

    pub fn serialize<S>(val: &Option<Address>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match *val {
            Some(ref value) => {
                let string = value.to_string();
                serializer.serialize_some(&string)
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Address>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = Option::<String>::deserialize(deserializer)?;
        s.map(|s| Address::from_str(&s).map_err(serde::de::Error::custom))
            .transpose()
    }
}

pub trait Len {
    fn len(&self) -> usize;
    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T> Len for Vec<T> {
    #[inline]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl<K, V, S> Len for HashMap<K, V, S> {
    #[inline]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

// pub mod serde_as_option {
//     use crate::serde_impls::Len;
//     use serde::{Deserialize, Serialize};
//
//     pub fn serialize<S, L: Len + Serialize>(val: &L, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         if val.is_empty() {
//             Option::<L>::None.serialize(serializer)
//         } else {
//             Some(val).serialize(serializer)
//         }
//     }
//
//     pub fn deserialize<'de, D, L: Deserialize<'de> + Default>(
//         deserializer: D,
//     ) -> Result<L, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         Option::<L>::deserialize(deserializer).map(|v| v.unwrap_or_default())
//     }
// }
