use crate::prelude::*;

#[unsized_type(skip_idl)]
pub struct UnsizedMap<K: UnsizedGenerics + Ord, V: UnsizedType, L: ListLength> {
    #[unsized_start]
    keys: List<K, L>,
    value: V,
}

#[unsized_type(skip_idl)]
pub struct UnsizedSet<V: UnsizedType + Ord, L: ListLength> {
    #[unsized_start]
    length: List<u8, L>,
    value: V,
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use super::*;
    use crate::idl::TypeToIdl;
    use star_frame_idl::ty::IdlTypeDef;
    use star_frame_idl::IdlDefinition;

    impl<K, V, L> TypeToIdl for UnsizedMap<K, V, L>
    where
        K: UnsizedGenerics + TypeToIdl + Ord,
        V: UnsizedType + TypeToIdl,
        L: ListLength + TypeToIdl,
    {
        type AssociatedProgram = System;

        fn type_to_idl(idl_definition: &mut IdlDefinition) -> Result<IdlTypeDef> {
            // todo: use the actual type layout for unsized types once that is implemented
            Ok(IdlTypeDef::Map {
                len_ty: L::type_to_idl(idl_definition)?.into(),
                key_ty: K::type_to_idl(idl_definition)?.into(),
                value_ty: V::type_to_idl(idl_definition)?.into(),
            })
        }
    }

    impl<V, L> TypeToIdl for UnsizedSet<V, L>
    where
        V: UnsizedType + Ord + TypeToIdl,
        L: ListLength + TypeToIdl,
    {
        type AssociatedProgram = System;

        fn type_to_idl(idl_definition: &mut IdlDefinition) -> Result<IdlTypeDef> {
            // todo: use the actual type layout for unsized types once that is implemented
            Ok(IdlTypeDef::Set {
                len_ty: L::type_to_idl(idl_definition)?.into(),
                item_ty: V::type_to_idl(idl_definition)?.into(),
            })
        }
    }
}
