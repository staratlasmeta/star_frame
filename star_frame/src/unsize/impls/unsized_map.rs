use crate::prelude::*;

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
    fn as_offset_mut(&mut self) -> &mut PackedValue<u32> {
        self.offset.as_offset_mut()
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

#[unsized_type(skip_idl)]
pub struct UnsizedMap<K: Pod + Ord + Align1, V: UnsizedType + ?Sized> {
    #[unsized_start]
    list: UnsizedList<V, OrdOffset<K>>,
}

#[unsized_impl(inherent)]
impl<K: Pod + Ord + Align1, V: UnsizedType + ?Sized> UnsizedMap<K, V> {
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
}

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
