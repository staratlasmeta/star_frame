use std::collections::HashMap;

pub mod serde_base58_pubkey {
    use serde::Deserialize;
    use solana_sdk::pubkey::Pubkey;
    use std::str::FromStr;

    pub fn serialize<S>(val: &Pubkey, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&val.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Pubkey, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Pubkey::from_str(&s).map_err(serde::de::Error::custom)
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

pub mod serde_as_option {
    use crate::serde_impls::Len;
    use serde::{Deserialize, Serialize};

    pub fn serialize<S, L: Len + Serialize>(val: &L, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if val.is_empty() {
            Option::<L>::None.serialize(serializer)
        } else {
            Some(val).serialize(serializer)
        }
    }

    pub fn deserialize<'de, D, L: Deserialize<'de> + Default>(
        deserializer: D,
    ) -> Result<L, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Option::<L>::deserialize(deserializer).map(|v| v.unwrap_or_default())
    }
}
