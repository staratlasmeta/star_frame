use crate::prelude::*;
use bytemuck::AnyBitPattern;
use star_frame_proc::unsized_impl;
use std::collections::HashMap;
use std::hash::Hash;

#[star_frame_proc::derivative(Copy, Clone, Debug, PartialEq, Eq)]
#[derive(Align1)]
#[repr(C)]
pub struct ListItemSized<K: UnsizedGenerics, V: UnsizedGenerics> {
    pub key: K,
    pub value: V,
}

const _: fn() = || {
    #[allow(clippy::missing_const_for_fn)]
    #[doc(hidden)]
    fn check<K: UnsizedGenerics, V: UnsizedGenerics>() {
        fn assert_impl<T: NoUninit + Zeroable + CheckedBitPattern>() {}
        assert_impl::<K>();
        assert_impl::<V>();
    }
};
unsafe impl<K: UnsizedGenerics, V: UnsizedGenerics> Zeroable for ListItemSized<K, V> {}
unsafe impl<K: UnsizedGenerics, V: UnsizedGenerics> NoUninit for ListItemSized<K, V> {}
#[star_frame_proc::derivative(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct ListItemSizedBits<K: UnsizedGenerics, V: UnsizedGenerics> {
    pub key: <K as CheckedBitPattern>::Bits,
    pub value: <V as CheckedBitPattern>::Bits,
}
unsafe impl<K: UnsizedGenerics, V: UnsizedGenerics> Zeroable for ListItemSizedBits<K, V> {}
unsafe impl<K: UnsizedGenerics, V: UnsizedGenerics> AnyBitPattern for ListItemSizedBits<K, V> {}
unsafe impl<K: UnsizedGenerics, V: UnsizedGenerics> CheckedBitPattern for ListItemSized<K, V> {
    type Bits = ListItemSizedBits<K, V>;
    #[inline]
    #[allow(clippy::double_comparisons)]
    fn is_valid_bit_pattern(bits: &Self::Bits) -> bool {
        <K as CheckedBitPattern>::is_valid_bit_pattern(&{ bits.key })
            && <V as CheckedBitPattern>::is_valid_bit_pattern(&{ bits.value })
    }
}

fn map_owned_from_ref<K, V, L>(r: MapRef<'_, K, V, L>) -> Result<HashMap<K, V>>
where
    K: UnsizedGenerics + Ord + Hash,
    V: UnsizedGenerics,
    L: ListLength,
{
    let mut map = HashMap::new();
    for ListItemSized { key, value } in r.list.as_checked_slice()?.iter().copied() {
        map.insert(key, value);
    }
    Ok(map)
}

#[unsized_type(skip_idl, owned_type = HashMap<K, V>, owned_from_ref = map_owned_from_ref::<K, V, L>)]
pub struct Map<K, V, L = u32>
where
    K: UnsizedGenerics + Ord + Hash,
    V: UnsizedGenerics,
    L: ListLength,
{
    #[unsized_start]
    list: List<ListItemSized<K, V>, L>,
}

#[unsized_impl]
impl<K, V, L> Map<K, V, L>
where
    K: UnsizedGenerics + Ord + Hash,
    V: UnsizedGenerics,
    L: ListLength,
{
    #[must_use]
    pub fn len(&self) -> usize {
        self.list.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    #[exclusive]
    pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>> {
        match self.list.binary_search_by(|probe| { probe.key }.cmp(&key)) {
            Ok(existing_index) => {
                let old = core::mem::replace(&mut self.list[existing_index].value, value);
                Ok(Some(old))
            }
            Err(insertion_index) => {
                self.list()
                    .insert(insertion_index, ListItemSized { key, value })?;
                Ok(None)
            }
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let list = &self.list;
        match list.binary_search_by(|probe| { probe.key }.cmp(key)) {
            Ok(existing_index) => Some(&list[existing_index].value),
            Err(_) => None,
        }
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let list = &mut self.list;
        match list.binary_search_by(|probe| { probe.key }.cmp(key)) {
            Ok(existing_index) => Some(&mut list[existing_index].value),
            Err(_) => None,
        }
    }

    #[exclusive]
    pub fn remove(&mut self, key: &K) -> Result<Option<V>> {
        match self.list.binary_search_by(|probe| { probe.key }.cmp(key)) {
            Ok(existing_index) => {
                let to_return = self.list[existing_index].value;
                self.list().remove(existing_index)?;
                Ok(Some(to_return))
            }
            Err(_) => Ok(None),
        }
    }
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use super::*;
    use crate::idl::TypeToIdl;
    use star_frame_idl::ty::IdlTypeDef;
    use star_frame_idl::IdlDefinition;
    impl<K, V, L> TypeToIdl for Map<K, V, L>
    where
        K: UnsizedGenerics + TypeToIdl + Ord + Hash,
        V: UnsizedGenerics + TypeToIdl,
        L: ListLength + TypeToIdl,
    {
        type AssociatedProgram = System;

        fn type_to_idl(idl_definition: &mut IdlDefinition) -> Result<IdlTypeDef> {
            Ok(IdlTypeDef::Map {
                len_ty: L::type_to_idl(idl_definition)?.into(),
                key_ty: K::type_to_idl(idl_definition)?.into(),
                value_ty: V::type_to_idl(idl_definition)?.into(),
            })
        }
    }
}
