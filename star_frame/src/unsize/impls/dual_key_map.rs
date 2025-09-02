use crate::prelude::*;
use crate::unsize::impls::{ListItemSized, ListLength, MapMut, MapRef, UnsizedGenerics};
use crate::unsize::{FromOwned, UnsizedType};
use bytemuck::AnyBitPattern;
use itertools::Itertools;
use std::collections::BTreeMap;

pub trait MultiMapKey: 'static {
    type KeyUnsized<L: ListLength>: UnsizedType;
    type KeyOwned: Ord;
    type KeyRef<'a>;
}
impl<K1, K2> MultiMapKey for (K1, K2)
where
    K1: UnsizedGenerics + Ord,
    K2: UnsizedGenerics + Ord,
{
    type KeyUnsized<L: ListLength> = DualKey<K1, K2, L>;
    type KeyOwned = (K1, K2);
    type KeyRef<'a> = (&'a K1, &'a K2);
}
#[unsized_type(skip_idl)]
pub struct DualKey<K1, K2, L>
where
    K1: UnsizedGenerics + Ord,
    K2: UnsizedGenerics + Ord,
    L: ListLength,
{
    #[unsized_start]
    k1: Map<K1, PackedValue<L>, L>,
    k2: Map<K2, PackedValue<L>, L>,
}

#[derive(Zeroable, Debug, Copy, Clone, Align1)]
#[repr(C, packed)]
struct DualKeyMapItem<K1, K2, V> {
    k1: K1,
    k2: K2,
    v: V,
}
unsafe impl<K1, K2, V> CheckedBitPattern for DualKeyMapItem<K1, K2, V>
where
    K1: CheckedBitPattern + Align1,
    K2: CheckedBitPattern + Align1,
    V: CheckedBitPattern + Align1,
{
    type Bits = DualKeyMapItemBits<K1::Bits, K2::Bits, V::Bits>;

    fn is_valid_bit_pattern(bits: &Self::Bits) -> bool {
        <K1 as CheckedBitPattern>::is_valid_bit_pattern(&bits.k1)
            && <K2 as CheckedBitPattern>::is_valid_bit_pattern(&bits.k2)
            && <V as CheckedBitPattern>::is_valid_bit_pattern(&bits.v)
    }
}
unsafe impl<K1, K2, V> NoUninit for DualKeyMapItem<K1, K2, V>
where
    K1: NoUninit + Align1,
    K2: NoUninit + Align1,
    V: NoUninit + Align1,
{
}
#[derive(Zeroable, Copy, Clone, Debug, Align1)]
struct DualKeyMapItemBits<K1, K2, V> {
    k1: K1,
    k2: K2,
    v: V,
}
unsafe impl<K1, K2, V> AnyBitPattern for DualKeyMapItemBits<K1, K2, V>
where
    K1: AnyBitPattern,
    K2: AnyBitPattern,
    V: AnyBitPattern,
{
}

#[unsized_type(skip_idl, owned_type = BTreeMap<(K1, K2), V>, owned_from_ref = multi_key_map_owned_from_ref::<K1, K2, V, L>, skip_init_struct)]
pub struct DualKeyMap<K1, K2, V, L = u32>
where
    K1: UnsizedGenerics + Ord,
    K2: UnsizedGenerics + Ord,
    V: UnsizedGenerics,
    L: ListLength,
{
    #[unsized_start]
    k1: Map<K1, PackedValue<L>, L>,
    k2: Map<K2, PackedValue<L>, L>,
    list: List<DualKeyMapItem<K1, K2, V>, L>,
}

fn multi_key_map_owned_from_ref<K1, K2, V, L>(
    r: &DualKeyMapRef<'_, K1, K2, V, L>,
) -> Result<BTreeMap<(K1, K2), V>>
where
    K1: UnsizedGenerics + Ord,
    K2: UnsizedGenerics + Ord,
    V: UnsizedGenerics,
    L: ListLength,
{
    todo!()
}

#[unsized_impl]
impl<K1, K2, V, L> DualKeyMap<K1, K2, V, L>
where
    K1: UnsizedGenerics + Ord,
    K2: UnsizedGenerics + Ord,
    V: UnsizedGenerics,
    L: ListLength,
{
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.k2.len()
    }

    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.k2.is_empty()
    }

    pub fn contains_key_left(&self, key: &K1) -> bool {
        self.k1.contains_key(key)
    }

    pub fn contains_key_right(&self, key: &K2) -> bool {
        self.k2.contains_key(key)
    }

    pub fn get_by_left(&self, key: &K1) -> Option<(&K2, &V)> {
        self.k1.get(key).map(|item| {
            let item = self.list.get(item.to_usize().unwrap()).unwrap();
            (item.k2, item.v)
        })
    }

    pub fn get_by_right(&self, key: &K2) -> Option<(&K1, &V)> {
        self.k2.get(key).map(|item| {
            let item = self.list.get(item.to_usize().unwrap()).unwrap();
            (item.k1, item.v)
        })
    }

    pub fn get_mut_by_left(&mut self, key: &K1) -> Option<(&K2, &mut V)> {
        self.k1.get(key).map(|item| {
            let item = self.list.get_mut(item.to_usize().unwrap()).unwrap();
            (item.k2, &mut item.v)
        })
    }

    pub fn get_mut_by_right(&mut self, key: &K2) -> Option<(&K1, &mut V)> {
        self.k2.get(key).map(|item| {
            let item = self.list.get_mut(item.to_usize().unwrap()).unwrap();
            (item.k1, &mut item.v)
        })
    }

    /// Inserts all key-value pairs from the provided iterator into the map.
    #[exclusive]
    pub fn insert_all(&mut self, values: impl IntoIterator<Item = (K1, K2, V)>) -> Result<()> {
        values
            .into_iter()
            .try_for_each(|(k1, k2, value)| self.insert(k1, k2, value).map(|_| ()))
    }

    #[exclusive]
    pub fn insert(&mut self, k1: K1, k2: K2, v: V) -> Result<Option<V>> {
        match (self.k1.get(&k1), self.k2.get(k2)) {
            (Some(existing_index1), Some(existing_index2))
                if existing_index1 == existing_index2 =>
            {
                let old =
                    core::mem::replace(&mut self.list[existing_index1.to_usize().unwrap()].v, v);
                Ok(Some(old))
            }
            (Some(_), Some(_)) => {
                bail!("Key 1 and Key 2 point to different values");
            }
            (None, None) => {
                let index = self.list.len();
                self.list().push(DualKeyMapItem { k1, k2, v })?;
                match self.k1().insert(k1, PackedValue(self.list.len() - 1)) {
                    Err(e) => {
                        self.list().remove(index).with_context(|| {
                            anyhow!("Failed to remove item when failing to insert, DualKeyMap in bad state: {e}")
                        })?;
                        return Err(e);
                    }
                    Ok(Some(_)) => {
                        unreachable!()
                    }
                    Ok(None) => {}
                }
                match self.k2().insert(k2, PackedValue(self.list.len() - 1)) {
                    Err(e) => {
                        self.k1().remove(&k1).with_context(|| {
                            anyhow!("Failed to remove item when failing to insert, DualKeyMap in bad state: {e}")
                        })?;
                        self.list().remove(index).with_context(|| {
                            anyhow!("Failed to remove item when failing to insert, DualKeyMap in bad state: {e}")
                        })?;
                        return Err(e);
                    }
                    Ok(Some(_)) => {
                        unreachable!()
                    }
                    Ok(None) => {}
                }
                Ok(None)
            }
            _ => {
                bail!("DualKeyMap in bad state, should never happen");
            }
        }
    }

    #[exclusive]
    pub fn remove_by_left(&mut self, key: &K1) -> Result<Option<(K2, V)>> {
        match self.get_index(key) {
            Ok(existing_index) => {
                let item = self.list[existing_index];
                self.k2.remove(&item.k2)?;
                self.k1.remove(key)?;
                self.list().remove(existing_index)?;
                Ok(Some((item.k2, item.v)))
            }
            Err(_) => Ok(None),
        }
    }

    #[exclusive]
    pub fn remove_by_right(&mut self, key: &K2) -> Result<Option<(K1, V)>> {
        match self.get_index(key) {
            Ok(existing_index) => {
                let item = self.list[existing_index];
                self.k1.remove(&item.k1)?;
                self.k2.remove(key)?;
                self.list().remove(existing_index)?;
                Ok(Some((item.k1, item.v)))
            }
            Err(_) => Ok(None),
        }
    }

    #[exclusive]
    pub fn clear(&mut self) -> Result<()> {
        self.k1.clear()?;
        self.k2
            .clear()
            .with_context(|| "Failed to clear k2, DualKeyMap in bad state")?;
        self.list()
            .remove_range(..)
            .with_context(|| "Failed to clear list, DualKeyMap in bad state")
    }
}

impl<'a, K, V, L> IntoIterator for &'a DualKeyMapMut<'_, K, V, L>
where
    K: UnsizedGenerics + Ord,
    V: UnsizedGenerics,
    L: ListLength,
{
    type Item = (&'a K, &'a V);
    type IntoIter = DualKeyMapIter<'a, K, V, L>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K, V, L> IntoIterator for &'a DualKeyMapRef<'_, K, V, L>
where
    K: UnsizedGenerics + Ord,
    V: UnsizedGenerics,
    L: ListLength,
{
    type Item = (&'a K, &'a V);
    type IntoIter = DualKeyMapIter<'a, K, V, L>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K, V, L> IntoIterator for &'a mut DualKeyMapMut<'_, K, V, L>
where
    K: UnsizedGenerics + Ord,
    V: UnsizedGenerics,
    L: ListLength,
{
    type Item = (&'a K, &'a mut V);
    type IntoIter = DualKeyMapIterMut<'a, K, V, L>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
