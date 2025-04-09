use crate::prelude::*;
use crate::unsize::impls::unsized_list::unsized_list_exclusive;
use std::collections::HashMap;
use std::hash::Hash;
use std::iter::FusedIterator;

#[derive(Align1, Zeroable, Debug, Copy, Clone)]
#[repr(C)]
pub struct OrdOffset<K>
where
    K: Pod + Ord + Align1,
{
    offset: PackedValue<u32>,
    key: K,
}

unsafe impl<K> Pod for OrdOffset<K> where K: Pod + Ord + Align1 {}

unsafe impl<K> UnsizedListOffset for OrdOffset<K>
where
    K: Pod + Ord + Align1,
{
    type ListOffsetInit = K;

    #[inline]
    fn to_usize_offset(&self) -> usize {
        self.offset.to_usize_offset()
    }

    #[inline]
    fn as_mut_offset(&mut self) -> &mut PackedValue<u32> {
        self.offset.as_mut_offset()
    }

    #[inline]
    fn as_offset(&self) -> &PackedValue<u32> {
        self.offset.as_offset()
    }

    #[inline]
    fn from_usize_offset(offset: usize, init: Self::ListOffsetInit) -> Result<Self> {
        Ok(OrdOffset {
            offset: <PackedValue<u32>>::from_usize_offset(offset, ())?,
            key: init,
        })
    }
}

#[derive(derive_where::DeriveWhere)]
#[derive_where(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd; Vec<(K, V::Owned)>)]
pub struct UnsizedMapOwned<K, V>
where
    K: Pod + Ord + Align1,
    V: UnsizedType + ?Sized,
{
    pub list: Vec<(K, V::Owned)>,
}

impl<K, V> UnsizedMapOwned<K, V>
where
    K: Pod + Ord + Align1 + Hash,
    V: UnsizedType + ?Sized,
{
    #[must_use]
    pub fn into_hash_map(self) -> HashMap<K, V::Owned> {
        self.list.into_iter().collect()
    }
}

fn unsized_map_owned_from_ref<K, V>(r: UnsizedMapRef<'_, K, V>) -> Result<UnsizedMapOwned<K, V>>
where
    K: Pod + Ord + Align1,
    V: UnsizedType + ?Sized,
{
    let mut owned = UnsizedMapOwned::default();
    for result in r.list.iter_with_offsets() {
        let (item, ord_offset) = result?;
        let owned_item = V::owned_from_ref(item)?;
        owned.list.push((ord_offset.key, owned_item));
    }
    Ok(owned)
}

#[unsized_type(skip_idl, owned_type = UnsizedMapOwned<K, V>, owned_from_ref = unsized_map_owned_from_ref::<K, V>)]
pub struct UnsizedMap<K, V>
where
    K: Pod + Ord + Align1,
    V: UnsizedType + ?Sized,
{
    #[unsized_start]
    list: UnsizedList<V, OrdOffset<K>>,
}

impl<K, V> Default for UnsizedMapOwned<K, V>
where
    K: Pod + Ord + Align1,
    V: UnsizedType + ?Sized,
{
    fn default() -> Self {
        Self { list: vec![] }
    }
}

#[unsized_impl(inherent)]
impl<K, V> UnsizedMap<K, V>
where
    K: Pod + Ord + Align1,
    V: UnsizedType + ?Sized,
{
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.list.len()
    }

    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    fn get_index(&self, key: &K) -> Result<usize, usize> {
        self.list
            .offset_list
            .binary_search_by(|probe| { probe.key }.cmp(key))
    }

    pub fn get(&self, key: &K) -> Result<Option<V::Ref<'_>>> {
        match self.get_index(key) {
            Ok(existing_index) => self.list.get(existing_index),
            Err(_) => Ok(None),
        }
    }

    pub fn get_mut(&mut self, key: &K) -> Result<Option<V::Mut<'_>>> {
        match self.get_index(key) {
            Ok(existing_index) => self.list.get_mut(existing_index),
            Err(_) => Ok(None),
        }
    }

    #[exclusive]
    pub fn get_exclusive<'child>(
        &'child mut self,
        key: &K,
    ) -> Result<Option<ExclusiveWrapper<'child, 'top, 'info, V::Mut<'ptr>, O, A>>> {
        let Ok(index) = self.get_index(key) else {
            return Ok(None);
        };
        unsafe {
            ExclusiveWrapper::try_map_ref(self, |data| {
                let list = &mut data.list;
                let (start, end) = list.get_unsized_range(index).expect("Index exists");
                unsized_list_exclusive!(<V> list start..end)
            })
        }
        .map(Some)
    }

    /// Inserts or modifies an item into the map, returning true if the item already existed, and false otherwise.
    #[exclusive]
    pub fn insert<I>(&mut self, key: K, value: I) -> Result<bool>
    where
        V: UnsizedInit<I>,
    {
        match self.get_index(&key) {
            Ok(existing_index) => {
                // TODO: optimize this by just modifying bytes to fit and then writing to them
                self.list().remove(existing_index)?;
                self.list().insert_with_offset(existing_index, value, key)?;
                Ok(true)
            }
            Err(insertion_index) => {
                self.list()
                    .insert_with_offset(insertion_index, value, key)?;
                Ok(false)
            }
        }
    }

    /// Removes an item from the map, returning true if the item existed, and false otherwise.
    #[exclusive]
    pub fn remove(&mut self, key: &K) -> Result<bool> {
        match self.get_index(key) {
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

#[unsized_impl]
impl<K, V> UnsizedMap<K, V>
where
    K: Pod + Ord + Align1,
    V: UnsizedType + ?Sized,
{
    #[inline]
    #[must_use]
    pub fn iter(&self) -> UnsizedMapIter<'_, K, V> {
        UnsizedMapIter {
            iter: self.list.iter_with_offsets(),
        }
    }

    #[inline]
    #[must_use]
    pub fn iter_mut(&mut self) -> UnsizedMapIterMut<'_, K, V> {
        UnsizedMapIterMut {
            iter: self.list.iter_with_offsets_mut(),
        }
    }
}

impl<'a, 'ptr, K, V> IntoIterator for &'a UnsizedMapRef<'ptr, K, V>
where
    K: Pod + Ord + Align1,
    V: UnsizedType + ?Sized,
{
    type Item = Result<(K, <V as UnsizedType>::Ref<'a>)>;
    type IntoIter = UnsizedMapIter<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, 'ptr, K, V> IntoIterator for &'a UnsizedMapMut<'ptr, K, V>
where
    K: Pod + Ord + Align1,
    V: UnsizedType + ?Sized,
{
    type Item = Result<(K, <V as UnsizedType>::Ref<'a>)>;
    type IntoIter = UnsizedMapIter<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, 'ptr, K, V> IntoIterator for &'a mut UnsizedMapMut<'ptr, K, V>
where
    K: Pod + Ord + Align1,
    V: UnsizedType + ?Sized,
{
    type Item = Result<(K, <V as UnsizedType>::Mut<'a>)>;
    type IntoIter = UnsizedMapIterMut<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

macro_rules! map_iter {
    ($name:ident $(: $extra_derive:path)?, $iter:ident, $item:ident)  => {
        #[derive(Debug, $($extra_derive)*)]
        pub struct $name<'a, K, V>
        where
            K: Pod + Ord + Align1,
            V: UnsizedType + ?Sized,
        {
            iter: $iter<'a, V, OrdOffset<K>>,
        }

        impl<'a, K, V> Iterator for $name<'a, K, V>
        where
            K: Pod + Ord + Align1,
            V: UnsizedType + ?Sized,
        {
            type Item = Result<(K, V::$item<'a>)>;

            fn next(&mut self) -> Option<Self::Item> {
                self.iter.next().map(|item| item.map(|(item, offset)| (offset.key, item)))
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                self.iter.size_hint()
            }
        }

        impl<K, V> ExactSizeIterator for $name<'_, K, V>
        where
            K: Pod + Ord + Align1,
            V: UnsizedType + ?Sized,
        {
            fn len(&self) -> usize {
                self.iter.len()
            }
        }

        impl<K, V> FusedIterator for $name<'_, K, V>
        where
            K: Pod + Ord + Align1,
            V: UnsizedType + ?Sized,
        {
        }
    };
}

map_iter!(UnsizedMapIter: Clone, UnsizedListWithOffsetIter, Ref);
map_iter!(UnsizedMapIterMut, UnsizedListWithOffsetIterMut, Mut);

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use super::*;
    use crate::idl::TypeToIdl;
    use star_frame_idl::ty::{IdlStructField, IdlTypeDef};
    use star_frame_idl::IdlDefinition;

    impl<K> TypeToIdl for OrdOffset<K>
    where
        K: Pod + Ord + Align1 + TypeToIdl,
    {
        type AssociatedProgram = System;

        fn type_to_idl(idl_definition: &mut IdlDefinition) -> Result<IdlTypeDef> {
            // TODO: is there a way to make this structure shared in codama?
            Ok(IdlTypeDef::Struct(vec![
                IdlStructField {
                    path: Some("offset".to_string()),
                    description: vec![],
                    type_def: IdlTypeDef::U32,
                },
                IdlStructField {
                    path: Some("key".to_string()),
                    description: vec![],
                    type_def: K::type_to_idl(idl_definition)?,
                },
            ]))
        }
    }

    impl<K, V> TypeToIdl for UnsizedMap<K, V>
    where
        K: Pod + Ord + Align1 + TypeToIdl,
        V: UnsizedType + ?Sized + TypeToIdl,
    {
        type AssociatedProgram = System;

        fn type_to_idl(idl_definition: &mut IdlDefinition) -> Result<IdlTypeDef> {
            <UnsizedList<V, OrdOffset<K>>>::type_to_idl(idl_definition)
        }
    }
}
