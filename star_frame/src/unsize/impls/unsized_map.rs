use crate::prelude::*;
use crate::unsize::impls::unsized_list::unsized_list_exclusive;
use std::collections::BTreeMap;
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

/// The [`UnsizedType::Owned`] variant of [`UnsizedMap`].
/// It is generally easier to create an initial [`BTreeMap`] or iterator of [`(K, V::Owned)`] and convert to this
/// type vs working on it directly.
#[derive(derive_where::DeriveWhere)]
#[derive_where(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd; Vec<(K, V::Owned)>)]
pub struct UnsizedMapOwned<K, V>
where
    K: Pod + Ord + Align1,
    V: UnsizedType + ?Sized,
{
    list: Vec<(K, V::Owned)>,
}

impl<K, V> From<BTreeMap<K, V::Owned>> for UnsizedMapOwned<K, V>
where
    K: Pod + Ord + Align1,
    V: UnsizedType + ?Sized,
{
    fn from(btree_map: BTreeMap<K, V::Owned>) -> Self {
        btree_map.into_iter().collect()
    }
}
impl<K, V> FromIterator<(K, V::Owned)> for UnsizedMapOwned<K, V>
where
    K: Pod + Ord + Align1,
    V: UnsizedType + ?Sized,
{
    fn from_iter<I: IntoIterator<Item = (K, V::Owned)>>(iter: I) -> Self {
        let mut map = Self::new();
        for (key, value) in iter {
            map.insert(key, value);
        }
        map
    }
}

impl<K, V> UnsizedMapOwned<K, V>
where
    K: Pod + Ord + Align1,
    V: UnsizedType + ?Sized,
{
    #[must_use]
    pub fn to_btree_map(self) -> BTreeMap<K, V::Owned> {
        self.list.into_iter().collect()
    }

    #[must_use]
    pub fn new() -> Self {
        Self { list: vec![] }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.list.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.list.binary_search_by(|probe| probe.0.cmp(key)).is_ok()
    }

    pub fn get(&self, key: &K) -> Option<&V::Owned> {
        match self.list.binary_search_by(|probe| probe.0.cmp(key)) {
            Ok(existing_index) => Some(&self.list[existing_index].1),
            Err(_) => None,
        }
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V::Owned> {
        match self.list.binary_search_by(|probe| probe.0.cmp(key)) {
            Ok(existing_index) => Some(&mut self.list[existing_index].1),
            Err(_) => None,
        }
    }

    pub fn insert(&mut self, key: K, value: V::Owned) -> Option<V::Owned> {
        match self.list.binary_search_by(|probe| probe.0.cmp(&key)) {
            Ok(existing_index) => {
                let old = core::mem::replace(&mut self.list[existing_index].1, value);
                Some(old)
            }
            Err(insertion_point) => {
                self.list.insert(insertion_point, (key, value));
                None
            }
        }
    }

    pub fn remove(&mut self, key: &K) -> Option<V::Owned> {
        match self.list.binary_search_by(|probe| probe.0.cmp(key)) {
            Ok(existing_index) => Some(self.list.remove(existing_index).1),
            Err(_) => None,
        }
    }

    pub fn clear(&mut self) {
        self.list.clear();
    }

    #[must_use]
    pub fn as_inner(&self) -> &Vec<(K, V::Owned)> {
        &self.list
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

unsafe impl<K, V> FromOwned for UnsizedMap<K, V>
where
    K: Pod + Ord + Align1,
    V: UnsizedType + FromOwned + ?Sized,
{
    fn byte_size(owned: &Self::Owned) -> usize {
        UnsizedList::<V, OrdOffset<K>>::from_owned_byte_size(owned.list.iter().map(|(_, v)| v))
    }

    fn from_owned(owned: Self::Owned, bytes: &mut &mut [u8]) -> Result<usize> {
        UnsizedList::<V, OrdOffset<K>>::from_owned_from_iter(
            owned.list.into_iter().map(|(k, v)| (v, k)),
            bytes,
        )
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
            .binary_search_by(|probe| probe.key.cmp(key))
    }

    #[must_use]
    pub fn contains_key(&self, key: &K) -> bool {
        self.get_index(key).is_ok()
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
    ) -> Result<Option<ExclusiveWrapper<'child, 'top, V::Mut<'top>, Self>>> {
        let Ok(index) = self.get_index(key) else {
            return Ok(None);
        };
        unsafe {
            ExclusiveWrapper::try_map_mut::<V, _>(self, |data| {
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

    #[inline]
    #[must_use]
    pub fn keys(&self) -> UnsizedMapKeys<'_, K, V> {
        UnsizedMapKeys {
            iter: self.list.iter_with_offsets(),
        }
    }

    #[inline]
    #[must_use]
    pub fn values(&self) -> UnsizedMapValues<'_, K, V> {
        UnsizedMapValues {
            iter: self.list.iter_with_offsets(),
        }
    }

    #[inline]
    #[must_use]
    pub fn values_mut(&mut self) -> UnsizedMapValuesMut<'_, K, V> {
        UnsizedMapValuesMut {
            iter: self.list.iter_with_offsets_mut(),
        }
    }
}

impl<'a, K, V> IntoIterator for &'a UnsizedMapRef<'_, K, V>
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

impl<'a, K, V> IntoIterator for &'a UnsizedMapMut<'_, K, V>
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

impl<'a, K, V> IntoIterator for &'a mut UnsizedMapMut<'_, K, V>
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
    ($name:ident $(: $extra_derive:path)?, $iter:ident, $item:ty, $next_arg:ident => $next:expr)  => {
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
            type Item = Result<$item>;

            fn next(&mut self) -> Option<Self::Item> {
                let $next_arg = self;
                $next_arg.iter.next().map(|item| item.map($next))
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

map_iter!(UnsizedMapIter: Clone, UnsizedListWithOffsetIter, (K, V::Ref<'a>), this => |(item, offset)| (offset.key, item));
map_iter!(UnsizedMapIterMut, UnsizedListWithOffsetIterMut, (K, V::Mut<'a>), this => |(item, offset)| (offset.key, item));
map_iter!(UnsizedMapKeys: Clone, UnsizedListWithOffsetIter, K, this => |(_item, offset)| offset.key);
map_iter!(UnsizedMapValues: Clone, UnsizedListWithOffsetIter, V::Ref<'a>, this => |(item, _offset)| item);
map_iter!(UnsizedMapValuesMut, UnsizedListWithOffsetIterMut, V::Mut<'a>, this => |(item, _offset)| item);

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

#[cfg(all(test, feature = "test_helpers"))]
mod tests {
    use super::*;
    use crate::unsize::TestByteSet;

    #[test]
    fn test_from_owned() -> Result<()> {
        type MyMap = UnsizedMap<Pubkey, List<Pubkey>>;
        let owned: <MyMap as UnsizedType>::Owned = [
            (Pubkey::new_unique(), vec![Pubkey::new_unique()]),
            (
                Pubkey::new_unique(),
                vec![
                    Pubkey::new_unique(),
                    Pubkey::new_unique(),
                    Pubkey::new_unique(),
                ],
            ),
            (Pubkey::new_unique(), vec![Pubkey::new_unique()]),
        ]
        .into_iter()
        .collect();
        let test_bytes = TestByteSet::<MyMap>::new(owned.clone())?;
        assert_eq!(test_bytes.owned()?, owned);
        Ok(())
    }
}
