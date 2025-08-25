use crate::{
    prelude::*,
    unsize::{
        impls::{ListIter, ListIterMut, ListLength, UnsizedGenerics},
        FromOwned,
    },
};
use bytemuck::AnyBitPattern;
use std::{collections::BTreeMap, iter::FusedIterator};

#[derive(Align1, Copy, Clone, Debug, Default, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[repr(C)]
pub struct ListItemSized<K: UnsizedGenerics, V: UnsizedGenerics> {
    pub key: K,
    pub value: V,
}

const _: fn() = || {
    #[allow(clippy::missing_const_for_fn, unused)]
    #[doc(hidden)]
    fn check<K: UnsizedGenerics, V: UnsizedGenerics>() {
        fn assert_impl<T: NoUninit + Zeroable + CheckedBitPattern>() {}
        assert_impl::<K>();
        assert_impl::<V>();
    }
};
unsafe impl<K: UnsizedGenerics, V: UnsizedGenerics> Zeroable for ListItemSized<K, V> {}
unsafe impl<K: UnsizedGenerics, V: UnsizedGenerics> NoUninit for ListItemSized<K, V> {}
#[derive_where::derive_where(Debug, Copy, Clone; <K as CheckedBitPattern>::Bits, <V as CheckedBitPattern>::Bits)]
#[repr(C)]
pub struct ListItemSizedBits<K: UnsizedGenerics, V: UnsizedGenerics> {
    pub key: <K as CheckedBitPattern>::Bits,
    pub value: <V as CheckedBitPattern>::Bits,
}
unsafe impl<K: UnsizedGenerics, V: UnsizedGenerics> Zeroable for ListItemSizedBits<K, V> {}
unsafe impl<K: UnsizedGenerics, V: UnsizedGenerics> AnyBitPattern for ListItemSizedBits<K, V> {}
unsafe impl<K: UnsizedGenerics, V: UnsizedGenerics> CheckedBitPattern for ListItemSized<K, V> {
    type Bits = ListItemSizedBits<K, V>;
    #[inline]
    fn is_valid_bit_pattern(bits: &Self::Bits) -> bool {
        <K as CheckedBitPattern>::is_valid_bit_pattern(&{ bits.key })
            && <V as CheckedBitPattern>::is_valid_bit_pattern(&{ bits.value })
    }
}

#[unsized_type(skip_idl, owned_type = BTreeMap<K, V>, owned_from_ref = map_owned_from_ref::<K, V, L>, skip_init_struct)]
pub struct Map<K, V, L = u32>
where
    K: UnsizedGenerics + Ord,
    V: UnsizedGenerics,
    L: ListLength,
{
    #[unsized_start]
    list: List<ListItemSized<K, V>, L>,
}

#[allow(clippy::unnecessary_wraps)]
fn map_owned_from_ref<K, V, L>(r: &MapRef<'_, K, V, L>) -> Result<BTreeMap<K, V>>
where
    K: UnsizedGenerics + Ord,
    V: UnsizedGenerics,
    L: ListLength,
{
    Ok(r.list.iter().map(|item| (item.key, item.value)).collect())
}

impl<K, V, L> FromOwned for Map<K, V, L>
where
    K: UnsizedGenerics + Ord,
    V: UnsizedGenerics,
    L: ListLength,
{
    fn byte_size(owned: &Self::Owned) -> usize {
        List::<ListItemSized<K, V>, L>::byte_size_from_len(owned.len())
    }

    fn from_owned(owned: Self::Owned, bytes: &mut &mut [u8]) -> Result<usize> {
        List::<ListItemSized<K, V>, L>::from_owned_from_iter(
            owned
                .into_iter()
                .map(|(key, value)| ListItemSized { key, value }),
            bytes,
        )
    }
}

#[unsized_impl]
impl<K, V, L> Map<K, V, L>
where
    K: UnsizedGenerics + Ord,
    V: UnsizedGenerics,
    L: ListLength,
{
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.list.len()
    }

    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    #[inline]
    fn get_index(&self, key: &K) -> Result<usize, usize> {
        self.list.binary_search_by(|probe| probe.key.cmp(key))
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.get_index(key).is_ok()
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        match self.get_index(key) {
            Ok(existing_index) => Some(&self.list[existing_index].value),
            Err(_) => None,
        }
    }

    /// Accesses the items from the underlying list by index. Returns `None` if the index is out of bounds.
    /// This makes no guarantees about the order of the items when adding or removing elements.
    #[must_use]
    pub fn get_by_index(&self, index: usize) -> Option<(&K, &V)> {
        self.list.get(index).map(|item| (&item.key, &item.value))
    }

    /// Accesses the items from the underlying list by index. Returns `None` if the index is out of bounds.
    /// This makes no guarantees about the order of the items when adding or removing elements.
    #[must_use]
    pub fn get_by_index_mut(&mut self, index: usize) -> Option<(&K, &mut V)> {
        self.list
            .get_mut(index)
            .map(|item| (&item.key, &mut item.value))
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        match self.get_index(key) {
            Ok(existing_index) => Some(&mut self.list[existing_index].value),
            Err(_) => None,
        }
    }

    #[exclusive]
    pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>> {
        match self.get_index(&key) {
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

    #[exclusive]
    pub fn remove(&mut self, key: &K) -> Result<Option<V>> {
        match self.get_index(key) {
            Ok(existing_index) => {
                let to_return = self.list[existing_index].value;
                self.list().remove(existing_index)?;
                Ok(Some(to_return))
            }
            Err(_) => Ok(None),
        }
    }

    #[exclusive]
    pub fn clear(&mut self) -> Result<()> {
        self.list().remove_range(..)
    }

    #[must_use]
    #[inline]
    pub fn iter(&self) -> MapIter<'_, K, V, L> {
        MapIter {
            iter: self.list.iter(),
        }
    }

    #[must_use]
    #[inline]
    pub fn iter_mut(&mut self) -> MapIterMut<'_, K, V, L> {
        MapIterMut {
            iter: self.list.iter_mut(),
        }
    }

    #[must_use]
    #[inline]
    pub fn keys(&self) -> MapKeys<'_, K, V, L> {
        MapKeys {
            iter: self.list.iter(),
        }
    }

    #[must_use]
    #[inline]
    pub fn values(&self) -> MapValues<'_, K, V, L> {
        MapValues {
            iter: self.list.iter(),
        }
    }

    #[must_use]
    #[inline]
    pub fn values_mut(&mut self) -> MapValuesMut<'_, K, V, L> {
        MapValuesMut {
            iter: self.list.iter_mut(),
        }
    }
}

macro_rules! map_iter {
    ($name:ident $(: $extra_derive:path)?, $iter:ident, $item:ty, $next_arg:ident => $next:expr)  => {
        #[derive(Debug, $($extra_derive)*)]
        pub struct $name<'a, K, V, L>
        where
            K: UnsizedGenerics + Ord,
            V: UnsizedGenerics,
            L: ListLength,
        {
            iter: $iter<'a, ListItemSized<K, V>, L>,
        }

        impl<'a, K, V, L> Iterator for $name<'a, K, V, L>
        where
            K: UnsizedGenerics + Ord,
            V: UnsizedGenerics,
            L: ListLength,
        {
            type Item = $item;

            fn next(&mut self) -> Option<Self::Item> {
                let $next_arg = self;
                $next_arg.iter.next().map($next)
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                self.iter.size_hint()
            }
        }

        impl<K, V, L> ExactSizeIterator for $name<'_, K, V, L>
        where
            K: UnsizedGenerics + Ord,
            V: UnsizedGenerics,
            L: ListLength,
        {
            fn len(&self) -> usize {
                self.iter.len()
            }
        }

        impl<K, V, L> FusedIterator for $name<'_, K, V, L>
        where
            K: UnsizedGenerics + Ord,
            V: UnsizedGenerics,
            L: ListLength,
        {
        }
    };
}

map_iter!(MapIter: Clone, ListIter, (&'a K, &'a V), this => |item| (&item.key, &item.value));
map_iter!(MapIterMut, ListIterMut, (&'a K, &'a mut V), this => |item| (&item.key, &mut item.value));
map_iter!(MapKeys: Clone, ListIter, &'a K, this => |item| &item.key);
map_iter!(MapValues: Clone, ListIter, &'a V, this => |item| &item.value);
map_iter!(MapValuesMut, ListIterMut, &'a mut V, this => |item| &mut item.value);

impl<'a, K, V, L> IntoIterator for &'a MapMut<'_, K, V, L>
where
    K: UnsizedGenerics + Ord,
    V: UnsizedGenerics,
    L: ListLength,
{
    type Item = (&'a K, &'a V);
    type IntoIter = MapIter<'a, K, V, L>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K, V, L> IntoIterator for &'a MapRef<'_, K, V, L>
where
    K: UnsizedGenerics + Ord,
    V: UnsizedGenerics,
    L: ListLength,
{
    type Item = (&'a K, &'a V);
    type IntoIter = MapIter<'a, K, V, L>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K, V, L> IntoIterator for &'a mut MapMut<'_, K, V, L>
where
    K: UnsizedGenerics + Ord,
    V: UnsizedGenerics,
    L: ListLength,
{
    type Item = (&'a K, &'a mut V);
    type IntoIter = MapIterMut<'a, K, V, L>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use super::*;
    use crate::idl::TypeToIdl;
    use star_frame_idl::{ty::IdlTypeDef, IdlDefinition};
    impl<K, V, L> TypeToIdl for Map<K, V, L>
    where
        K: UnsizedGenerics + TypeToIdl + Ord,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_from_owned() -> Result<()> {
        let owned: BTreeMap<u8, u8> = vec![(1, 10), (3, 30), (2, 20)].into_iter().collect();
        let map = Map::<u8, u8>::new_byte_set(owned.clone())?;
        map.data_mut()?;
        let map_owned = map.owned()?;
        assert_eq!(map_owned, owned);
        Ok(())
    }
}
