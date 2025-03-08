use crate::prelude::*;

#[unsized_type(skip_idl, owned_attributes = [derive(Eq, PartialEq)])]
pub struct Set<K, L = u32>
where
    K: UnsizedGenerics + Ord,
    L: ListLength,
{
    #[unsized_start]
    list: List<K, L>,
}

#[unsized_impl]
impl<V, L> Set<V, L>
where
    V: UnsizedGenerics + Ord,
    L: ListLength,
{
    #[must_use] pub fn len(&self) -> usize {
        self.list.len()
    }

    #[must_use] pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    pub fn insert(self, value: V) -> Result<usize> {
        let mut list = self.list()?;
        match list.binary_search(&value) {
            Ok(existing_index) => Ok(existing_index),
            Err(insertion_index) => {
                list.insert(insertion_index, value)?;
                Ok(insertion_index)
            }
        }
    }

    pub fn remove(self, value: &V) -> Result<bool> {
        let mut list = self.list_exclusive();
        match list.binary_search(value) {
            Ok(existing_index) => {
                list.remove(existing_index)?;
                Ok(true)
            }
            Err(_) => Ok(false),
        }
    }

    pub fn clear(self) -> Result<()> {
        let mut list = self.list_exclusive();
        list.remove_range(..)?;
        Ok(())
    }

    pub fn contains(&self, value: &V) -> bool {
        self.list.binary_search(value).is_ok()
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
