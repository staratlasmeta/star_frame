use std::collections::BTreeSet;

use crate::prelude::*;
use crate::unsize::FromOwned;

#[unsized_type(skip_idl)]
pub struct Set<K, L = u32>
where
    K: UnsizedGenerics + Ord,
    L: ListLength,
{
    #[unsized_start]
    list: List<K, L>,
}

impl<K, L> From<BTreeSet<K>> for SetOwned<K, L>
where
    K: UnsizedGenerics + Ord,
    L: ListLength,
{
    fn from(btree_set: BTreeSet<K>) -> Self {
        let mut set = Self::new();
        for key in btree_set {
            set.insert(key);
        }
        set
    }
}

impl<K, L> FromIterator<K> for SetOwned<K, L>
where
    K: UnsizedGenerics + Ord,
    L: ListLength,
{
    fn from_iter<I: IntoIterator<Item = K>>(iter: I) -> Self {
        let mut set = Self::new();
        for key in iter {
            set.insert(key);
        }
        set
    }
}

impl<K, L> SetOwned<K, L>
where
    K: UnsizedGenerics + Ord,
    L: ListLength,
{
    pub fn to_btree_set(self) -> BTreeSet<K> {
        self.list.into_iter().collect()
    }

    pub fn new() -> Self {
        Self { list: vec![] }
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    pub fn insert(&mut self, key: K) -> Option<K> {
        match self.list.binary_search(&key) {
            Ok(existing_index) => {
                let old = core::mem::replace(&mut self.list[existing_index], key);
                Some(old)
            }
            Err(insertion_index) => {
                self.list.insert(insertion_index, key);
                None
            }
        }
    }

    pub fn remove(&mut self, key: &K) -> Option<K> {
        match self.list.binary_search(key) {
            Ok(existing_index) => {
                let old = self.list.remove(existing_index);
                Some(old)
            }
            Err(_) => None,
        }
    }

    pub fn clear(&mut self) {
        self.list.clear();
    }

    pub fn as_inner(&self) -> &Vec<K> {
        &self.list
    }
}

impl<K, L> FromOwned for Set<K, L>
where
    K: UnsizedGenerics + Ord,
    L: ListLength,
{
    fn byte_size(owned: &Self::Owned) -> usize {
        List::<K, L>::byte_size(&owned.list)
    }

    fn from_owned(owned: Self::Owned, out: &mut [u8]) -> Result<usize> {
        List::<K, L>::from_owned(owned.list, out)
    }
}

#[unsized_impl(inherent)]
impl<V, L> Set<V, L>
where
    V: UnsizedGenerics + Ord,
    L: ListLength,
{
    #[must_use]
    pub fn len(&self) -> usize {
        self.list.len()
    }

    #[must_use]
    pub fn contains(&self, value: &V) -> bool {
        self.list.binary_search(value).is_ok()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    #[exclusive]
    pub fn insert(&mut self, value: V) -> Result<usize> {
        match self.list.binary_search(&value) {
            Ok(existing_index) => Ok(existing_index),
            Err(insertion_index) => {
                self.list().insert(insertion_index, value)?;
                Ok(insertion_index)
            }
        }
    }

    #[exclusive]
    pub fn remove(&mut self, value: &V) -> Result<bool> {
        match self.list.binary_search(value) {
            Ok(existing_index) => {
                self.list().remove(existing_index)?;
                Ok(true)
            }
            Err(_) => Ok(false),
        }
    }

    #[exclusive]
    pub fn clear(&mut self) -> Result<()> {
        self.list().remove_range(..)?;
        Ok(())
    }
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use super::*;
    use crate::idl::TypeToIdl;
    use star_frame_idl::ty::IdlTypeDef;
    use star_frame_idl::IdlDefinition;

    impl<V, L> TypeToIdl for Set<V, L>
    where
        V: UnsizedGenerics + TypeToIdl + Ord,
        L: ListLength + TypeToIdl,
    {
        type AssociatedProgram = System;

        fn type_to_idl(idl_definition: &mut IdlDefinition) -> Result<IdlTypeDef> {
            Ok(IdlTypeDef::Set {
                len_ty: L::type_to_idl(idl_definition)?.into(),
                item_ty: V::type_to_idl(idl_definition)?.into(),
            })
        }
    }
}
