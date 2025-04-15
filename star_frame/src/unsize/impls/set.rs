use std::collections::BTreeSet;

use crate::prelude::*;

/// A resizable set of unique, fixed-size elements. The [`UnsizedType`] version of [`BTreeSet`].
///
/// Under the hood, a `Set` is a sorted [`List`] of unique elements.
/// Searches are performed using binary search.
///
/// ## Unsized Type System
/// See [`SetRef`] and [`SetMut`]. These will be used often in the `UnsizedType` system.
/// For exclusive methods that change the underlying data size, see [`SetExclusiveImpl`].
#[unsized_type(skip_idl, owned_attributes = [doc = "The [`UnsizedType::Owned`] variant of [`Set`]. 
    It is generally easier to create an initial [`BTreeSet`] or iterator of `T`
    and convert to this type vs working on it directly."])]
pub struct Set<T, L = u32>
where
    T: UnsizedGenerics + Ord,
    L: ListLength,
{
    #[unsized_start]
    list: List<T, L>,
}

impl<T, L> From<BTreeSet<T>> for SetOwned<T, L>
where
    T: UnsizedGenerics + Ord,
    L: ListLength,
{
    fn from(btree_set: BTreeSet<T>) -> Self {
        let mut set = Self::new();
        for key in btree_set {
            set.insert(key);
        }
        set
    }
}

impl<T, L> FromIterator<T> for SetOwned<T, L>
where
    T: UnsizedGenerics + Ord,
    L: ListLength,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut set = Self::new();
        for key in iter {
            set.insert(key);
        }
        set
    }
}

impl<T, L> SetOwned<T, L>
where
    T: UnsizedGenerics + Ord,
    L: ListLength,
{
    /// Consumes the set and returns a `BTreeSet` containing all the elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use star_frame::prelude::*;
    /// use std::collections::BTreeSet;
    /// let mut set: SetOwned<u8> = [10u8, 11, 12].into_iter().collect();
    /// assert_eq!(set.to_btree_set(), BTreeSet::from([10u8, 11, 12]));
    /// ```
    #[must_use]
    pub fn to_btree_set(self) -> BTreeSet<T> {
        self.list.into_iter().collect()
    }

    /// Creates a new empty set.
    ///
    /// # Examples
    ///
    /// ```
    /// use star_frame::prelude::*;
    /// let set = SetOwned::<u8>::new();
    /// assert_eq!(set.len(), 0);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self { list: vec![] }
    }

    /// Returns the number of elements in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use star_frame::prelude::*;
    /// let set = SetOwned::<u8>::new();
    /// assert_eq!(set.len(), 0);
    /// let set: SetOwned<u8> = [10u8, 11, 12].into_iter().collect();
    /// assert_eq!(set.len(), 3);
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
    /// let mut set: SetOwned<u8> = SetOwned::new();
    /// assert!(set.is_empty());
    /// set.insert(10u8);
    /// assert!(!set.is_empty());
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
    /// let mut set = SetOwned::<u8>::new();
    /// assert!(!set.contains(&10u8));
    /// set.insert(10u8);
    /// assert!(set.contains(&10u8));
    /// ```
    #[must_use]
    pub fn contains(&self, key: &T) -> bool {
        self.list.binary_search(key).is_ok()
    }

    /// Returns `true` if the set did not already contain the specified element.
    ///
    /// If the set already contains this element, the method will return `false` and
    /// leave the set unchanged.
    ///
    /// # Examples
    ///
    /// ```
    /// use star_frame::prelude::*;
    /// let mut set = SetOwned::<u8>::new();
    /// assert_eq!(set.insert(10u8), true);
    /// assert_eq!(set.insert(10u8), false);
    /// ```
    pub fn insert(&mut self, key: T) -> bool {
        match self.list.binary_search(&key) {
            Ok(_existing_index) => false,
            Err(insertion_index) => {
                self.list.insert(insertion_index, key);
                true
            }
        }
    }

    /// Removes a key from the set.
    ///
    /// Returns `true` if the set contained the element to be removed, and `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use star_frame::prelude::*;
    /// let mut set = SetOwned::<u8>::new();
    /// set.insert(10u8);
    /// assert_eq!(set.remove(&10u8), true);
    /// assert_eq!(set.remove(&10u8), false);
    /// ```
    pub fn remove(&mut self, key: &T) -> bool {
        match self.list.binary_search(key) {
            Ok(existing_index) => {
                self.list.remove(existing_index);
                true
            }
            Err(_) => false,
        }
    }

    /// Removes all elements from the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use star_frame::prelude::*;
    /// let mut set: SetOwned<u8> = [1u8, 2, 3].into_iter().collect();
    /// assert!(!set.is_empty());
    /// set.clear();
    /// assert!(set.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.list.clear();
    }

    /// Returns a reference to the inner list.
    #[must_use]
    pub fn as_inner(&self) -> &Vec<T> {
        &self.list
    }
}

#[unsized_impl]
impl<T, L> Set<T, L>
where
    T: UnsizedGenerics + Ord,
    L: ListLength,
{
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
    /// set.exclusive().insert(10)?;
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

    /// Adds a value to the set. Returns whether the value was already present. If the value is already present, the set is unchanged.
    ///
    /// # Examples
    ///
    /// ```
    /// use star_frame::prelude::*;
    /// # fn main() -> Result<()> {
    /// let bytes = <Set<u8>>::new_default_byte_set()?;
    /// let mut set = bytes.data_mut()?;
    /// let mut set = set.exclusive();
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
    /// let mut set = set.exclusive();
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
    /// set.exclusive().clear()?;
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
    use star_frame_idl::ty::IdlTypeDef;
    use star_frame_idl::IdlDefinition;

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
