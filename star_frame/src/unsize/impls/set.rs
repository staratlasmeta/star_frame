use crate::{
    prelude::*,
    unsize::{
        impls::{ListIter, ListLength, UnsizedGenerics},
        FromOwned,
    },
};
use std::collections::BTreeSet;

/// A resizable set of unique, fixed-size elements. The [`UnsizedType`] version of [`BTreeSet`].
///
/// Under the hood, a `Set` is a sorted [`List`] of unique elements.
/// Searches are performed using binary search.
///
/// ## Unsized Type System
/// See [`SetRef`] and [`SetMut`]. These will be used often in the `UnsizedType` system.
/// For exclusive methods that change the underlying data size, see [`SetExclusiveImpl`].
#[unsized_type(skip_idl, owned_type = BTreeSet<T>, owned_from_ref = unsized_set_owned_from_ref::<T, L>, skip_init_struct)]
pub struct Set<T, L = u32>
where
    T: UnsizedGenerics + Ord,
    L: ListLength,
{
    #[unsized_start]
    list: List<T, L>,
}

impl<T, L> FromOwned for Set<T, L>
where
    T: UnsizedGenerics + Ord,
    L: ListLength,
{
    fn byte_size(owned: &Self::Owned) -> usize {
        List::<T, L>::byte_size_from_len(owned.len())
    }

    fn from_owned(owned: Self::Owned, bytes: &mut &mut [u8]) -> Result<usize> {
        List::<T, L>::from_owned_from_iter(owned, bytes)
    }
}

#[allow(clippy::unnecessary_wraps)]
fn unsized_set_owned_from_ref<T, L>(r: &SetRef<'_, T, L>) -> Result<BTreeSet<T>>
where
    T: UnsizedGenerics + Ord,
    L: ListLength,
{
    Ok(r.list.iter().copied().collect())
}

#[unsized_impl]
impl<T, L> Set<T, L>
where
    T: UnsizedGenerics + Ord,
    L: ListLength,
{
    // TODO: Potentially figure out a way to make these docs only on mut or ref, so the doc tests aren't repeated. Perhaps "```ignore" on Mut?
    /// Returns the number of elements in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use star_frame::prelude::*;
    /// # fn main() -> Result<()> {
    /// let bytes = <Set<u8>>::new_byte_set(vec![1u8,2,3].into_iter().collect())?;
    /// let mut set = bytes.data_mut()?;
    /// assert_eq_with_shared!(set => set.len(), 3);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.list.len()
    }

    /// Returns `true` if the set contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use star_frame::prelude::*;
    /// # fn main() -> Result<()> {
    /// let bytes = <Set<u8>>::new_default_byte_set()?;
    /// let mut set = bytes.data_mut()?;
    /// assert_with_shared!(set => set.is_empty());
    /// set.insert(10)?;
    /// assert_with_shared!(set => !set.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    /// Returns `true` if the set contains the specified value.
    ///
    /// # Examples
    ///
    /// ```
    /// use star_frame::prelude::*;
    /// # fn main() -> Result<()> {
    /// let bytes = <Set<u8>>::new_byte_set(vec![1u8,2,3].into_iter().collect())?;
    /// let mut set = bytes.data_mut()?;
    /// assert_eq_with_shared!(set => set.contains(&2), true);
    /// assert_eq_with_shared!(set => set.contains(&4), false);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn contains(&self, value: &T) -> bool {
        self.list.binary_search(value).is_ok()
    }

    /// Accesses the items from the underlying list by index. Returns `None` if the index is out of bounds.
    /// This makes no guarantees about the order of the items when adding or removing elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use star_frame::prelude::*;
    /// # fn main() -> Result<()> {
    /// let bytes = <Set<u8>>::new_byte_set(vec![1u8, 2, 3].into_iter().collect())?;
    /// let set = bytes.data()?;
    /// assert_eq!(set.get_by_index(0), Some(&1));
    /// assert_eq!(set.get_by_index(1), Some(&2));
    /// assert_eq!(set.get_by_index(2), Some(&3));
    /// # Ok(())
    /// # }
    #[must_use]
    pub fn get_by_index(&self, index: usize) -> Option<&T> {
        self.list.get(index)
    }

    /// Adds a value to the set.
    ///
    /// Returns whether the value was newly inserted. If the value is already present, the set is unchanged and `false` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use star_frame::prelude::*;
    /// # fn main() -> Result<()> {
    /// let bytes = <Set<u8>>::new_default_byte_set()?;
    /// let mut set = bytes.data_mut()?;
    /// assert_eq_with_shared!(set => set.len(), 0);
    ///
    /// assert_eq!(set.insert(1)?, true);
    /// assert_eq!(set.insert(1)?, false);
    /// assert_eq_with_shared!(set => set.len(), 1);
    /// # Ok(())
    /// # }
    /// ```
    #[exclusive]
    pub fn insert(&mut self, value: T) -> Result<bool> {
        match self.list.binary_search(&value) {
            Ok(_existing_index) => Ok(false),
            Err(insertion_index) => {
                self.list().insert(insertion_index, value)?;
                Ok(true)
            }
        }
    }

    /// Removes a value from the set. Returns whether the value was present in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use star_frame::prelude::*;
    /// # fn main() -> Result<()> {
    /// let bytes = <Set<u8>>::new_default_byte_set()?;
    /// let mut set = bytes.data_mut()?;
    /// set.insert(2)?;
    /// assert_eq!(set.remove(&2)?, true);
    /// assert_eq!(set.remove(&2)?, false);
    /// # Ok(())
    /// # }
    /// ```
    #[exclusive]
    pub fn remove(&mut self, value: &T) -> Result<bool> {
        match self.list.binary_search(value) {
            Ok(existing_index) => {
                self.list().remove(existing_index)?;
                Ok(true)
            }
            Err(_) => Ok(false),
        }
    }

    /// Removes all values from the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use star_frame::prelude::*;
    /// # fn main() -> Result<()> {
    /// let bytes = <Set<u8>>::new_byte_set(vec![1u8,2,3].into_iter().collect())?;
    /// let mut set = bytes.data_mut()?;
    /// set.clear()?;
    /// assert_with_shared!(set => set.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    #[exclusive]
    pub fn clear(&mut self) -> Result<()> {
        self.list().remove_range(..)?;
        Ok(())
    }

    /// Returns an iterator over the elements in the set. The iterator is ordered over the sorted elements in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use star_frame::prelude::*;
    /// # fn main() -> Result<()> {
    /// let bytes = <Set<u8>>::new_byte_set(vec![1u8,2].into_iter().collect())?;
    /// let set = bytes.data()?;
    /// let mut iter = set.iter();
    /// assert_eq!(iter.next(), Some(&1));
    /// assert_eq!(iter.next(), Some(&2));
    /// assert_eq!(iter.next(), None);
    ///
    /// drop(set);
    /// let set = bytes.data_mut()?;
    /// let mut iter = set.iter();
    /// assert_eq!(iter.next(), Some(&1));
    /// assert_eq!(iter.next(), Some(&2));
    /// assert_eq!(iter.next(), None);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn iter(&self) -> ListIter<'_, T, L> {
        self.list.iter()
    }
}

macro_rules! set_iter {
    ($name:ident) => {
        impl<'a, T, L> IntoIterator for &'a $name<'_, T, L>
        where
            T: UnsizedGenerics + Ord,
            L: ListLength,
        {
            type Item = &'a T;
            type IntoIter = ListIter<'a, T, L>;
            fn into_iter(self) -> Self::IntoIter {
                self.list.into_iter()
            }
        }
    };
}
set_iter!(SetMut);
set_iter!(SetRef);

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use super::*;
    use crate::idl::TypeToIdl;
    use star_frame_idl::{ty::IdlTypeDef, IdlDefinition};

    impl<T, L> TypeToIdl for Set<T, L>
    where
        T: UnsizedGenerics + TypeToIdl + Ord,
        L: ListLength + TypeToIdl,
    {
        type AssociatedProgram = System;

        fn type_to_idl(idl_definition: &mut IdlDefinition) -> Result<IdlTypeDef> {
            Ok(IdlTypeDef::Set {
                len_ty: L::type_to_idl(idl_definition)?.into(),
                item_ty: T::type_to_idl(idl_definition)?.into(),
            })
        }
    }
}
