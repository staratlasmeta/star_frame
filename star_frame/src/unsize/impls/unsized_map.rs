use crate::prelude::*;
use std::{collections::BTreeMap, iter::FusedIterator};

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

fn unsized_map_owned_from_ref<K, V>(r: &UnsizedMapRef<'_, K, V>) -> Result<BTreeMap<K, V::Owned>>
where
    K: Pod + Ord + Align1,
    V: UnsizedType + ?Sized,
{
    let mut owned = BTreeMap::new();
    for result in r.list.iter_with_offsets() {
        let (item, ord_offset) = result?;
        let owned_item = V::owned_from_ref(&item)?;
        owned.insert(ord_offset.key, owned_item);
    }
    Ok(owned)
}

#[unsized_type(skip_idl, owned_type = BTreeMap<K, V::Owned>, owned_from_ref = unsized_map_owned_from_ref::<K, V>)]
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
        UnsizedList::<V, OrdOffset<K>>::from_owned_byte_size(owned.iter().map(|(_, v)| v))
    }

    fn from_owned(owned: Self::Owned, bytes: &mut &mut [u8]) -> Result<usize> {
        UnsizedList::<V, OrdOffset<K>>::from_owned_from_iter(
            owned.into_iter().map(|(k, v)| (v, k)),
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
        let (start, _end) = self.list.get_unsized_range(index).expect("Index exists");
        unsafe {
            ExclusiveWrapper::try_map_mut::<V, _>(self, |data| {
                unsized_list_exclusive_fn(&raw mut (*data).list, start)
            })
        }
        .map(Some)
    }

    /// Accesses the items from the underlying list by index. Returns `None` if the index is out of bounds.
    /// This makes no guarantees about the order of the items when adding or removing elements.
    pub fn get_by_index(&self, index: usize) -> Result<Option<(K, V::Ref<'_>)>> {
        if let Some(offset) = self.list.offset_list.get(index) {
            let offset = *offset;
            let item = self.list.get(index)?.expect("Index exists");
            Ok(Some((offset.key, item)))
        } else {
            Ok(None)
        }
    }

    /// Accesses the items from the underlying list by index. Returns `None` if the index is out of bounds.
    /// This makes no guarantees about the order of the items when adding or removing elements.
    pub fn get_by_index_mut(&mut self, index: usize) -> Result<Option<(K, V::Mut<'_>)>> {
        if let Some(offset) = self.list.offset_list.get(index) {
            let offset = *offset;
            let item = self.list.get_mut(index)?.expect("Index exists");
            Ok(Some((offset.key, item)))
        } else {
            Ok(None)
        }
    }

    /// Inserts or modifies an item into the map, returning true if the item was newly inserted, and false otherwise.
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
                Ok(false)
            }
            Err(insertion_index) => {
                self.list()
                    .insert_with_offset(insertion_index, value, key)?;
                Ok(true)
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
    use star_frame_idl::{
        ty::{IdlStructField, IdlTypeDef},
        IdlDefinition,
    };

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
    use pretty_assertions::assert_eq;

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

    #[test]
    fn test_unsized_map_crud() -> Result<()> {
        let owned_map = vec![
            (0, vec![0, 1, 2]),
            (1, vec![10, 11, 12]),
            (2, vec![20, 21, 22]),
        ]
        .into_iter()
        .collect();
        let map = UnsizedMap::<u8, List<u8>>::new_byte_set(owned_map)?;
        let mut data = map.data_mut()?;
        // insert a few elements
        data.insert(1, [15, 16, 17])?;

        let mut second_item = data.get_exclusive(&1)?.expect("Second item exists");
        second_item.push(18)?;
        second_item.insert(0, 14)?;

        let mut last_item = data.get_exclusive(&2)?.expect("Last item exists");
        last_item.insert(0, 19)?;
        last_item.push_all([23, 24])?;

        drop(data);
        {
            let _data_again = map.data_mut()?;
        }
        let owned = map.owned()?;
        assert_eq!(
            owned,
            [
                (0, vec![0, 1, 2]),
                (1, vec![14, 15, 16, 17, 18]),
                (2, vec![19, 20, 21, 22, 23, 24])
            ]
            .into_iter()
            .collect()
        );
        Ok(())
    }
}
