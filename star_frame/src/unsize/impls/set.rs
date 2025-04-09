use crate::prelude::*;

#[unsized_type(skip_idl)]
pub struct Set<K, L = u32>
where
    K: UnsizedGenerics + Ord,
    L: ListLength,
{
    #[unsized_start]
    list: List<K, L>,
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
