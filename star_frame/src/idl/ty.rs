//! [`TypeToIdl`] implementations for common types.
//!
//! For zero-copy types that are supported by [`bytemuck`], we use that representation. For all other types, we use the [`borsh`] representation.
use crate::{data_types::PodBool, idl::TypeToIdl, program::system::System, Result};
use solana_pubkey::Pubkey;
use star_frame_idl::{
    ty::{IdlStructField, IdlTypeDef},
    IdlDefinition,
};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, LinkedList, VecDeque};

macro_rules! impl_type_to_idl_for_primitive {
    (@impl $ty:ty: $ident:ident) => {
        impl $crate::idl::TypeToIdl for $ty {
            type AssociatedProgram = $crate::program::system::System;

            fn type_to_idl(_idl_definition: &mut $crate::star_frame_idl::IdlDefinition) -> $crate::Result<$crate::star_frame_idl::ty::IdlTypeDef> {
                Ok($crate::star_frame_idl::ty::IdlTypeDef::$ident)
            }
        }
    };
    ($($ty:ty: $ident:ident),* $(,)?) => {
        $(impl_type_to_idl_for_primitive!(@impl $ty: $ident);)*
    };
}

pub(crate) use impl_type_to_idl_for_primitive;

impl_type_to_idl_for_primitive!(
    PodBool: Bool,
    bool: Bool,
    u8: U8,
    i8: I8,
    u16: U16,
    i16: I16,
    u32: U32,
    i32: I32,
    f32: F32,
    u64: U64,
    i64: I64,
    f64: F64,
    u128: U128,
    i128: I128,
    String: String,
    Pubkey: Pubkey,
);

impl<T: TypeToIdl> TypeToIdl for Option<T> {
    type AssociatedProgram = System;
    fn type_to_idl(idl_definition: &mut IdlDefinition) -> Result<IdlTypeDef> {
        Ok(IdlTypeDef::Option {
            ty: Box::new(T::type_to_idl(idl_definition)?),
            fixed: false,
        })
    }
}

impl<T: TypeToIdl> TypeToIdl for Box<T> {
    type AssociatedProgram = System;
    fn type_to_idl(idl_definition: &mut IdlDefinition) -> Result<IdlTypeDef> {
        T::type_to_idl(idl_definition)
    }
}

impl<T: TypeToIdl> TypeToIdl for Vec<T> {
    type AssociatedProgram = System;
    fn type_to_idl(idl_definition: &mut IdlDefinition) -> Result<IdlTypeDef> {
        Ok(IdlTypeDef::List {
            item_ty: Box::new(T::type_to_idl(idl_definition)?),
            // The only serialization format currently used that
            // supports `Vec<T>` is Borsh, which uses u32 for length.
            // Any other serialization would have to be implemented by the user using something
            // like the NewType pattern.
            len_ty: Box::new(IdlTypeDef::U32),
        })
    }
}

impl<T: TypeToIdl> TypeToIdl for VecDeque<T> {
    type AssociatedProgram = System;
    fn type_to_idl(idl_definition: &mut IdlDefinition) -> Result<IdlTypeDef> {
        <Vec<T>>::type_to_idl(idl_definition)
    }
}

impl<T: TypeToIdl> TypeToIdl for LinkedList<T> {
    type AssociatedProgram = System;
    fn type_to_idl(idl_definition: &mut IdlDefinition) -> Result<IdlTypeDef> {
        <Vec<T>>::type_to_idl(idl_definition)
    }
}

impl<T: TypeToIdl, const N: usize> TypeToIdl for [T; N] {
    type AssociatedProgram = System;
    fn type_to_idl(idl_definition: &mut IdlDefinition) -> Result<IdlTypeDef> {
        Ok(IdlTypeDef::Array(
            Box::new(T::type_to_idl(idl_definition)?),
            N,
        ))
    }
}

impl<T: TypeToIdl> TypeToIdl for BTreeSet<T> {
    type AssociatedProgram = System;
    fn type_to_idl(idl_definition: &mut IdlDefinition) -> Result<IdlTypeDef> {
        Ok(IdlTypeDef::Set {
            item_ty: Box::new(T::type_to_idl(idl_definition)?),
            len_ty: Box::new(IdlTypeDef::U32),
        })
    }
}

impl<K: TypeToIdl, V: TypeToIdl> TypeToIdl for BTreeMap<K, V> {
    type AssociatedProgram = System;
    fn type_to_idl(idl_definition: &mut IdlDefinition) -> Result<IdlTypeDef> {
        Ok(IdlTypeDef::Map {
            key_ty: Box::new(K::type_to_idl(idl_definition)?),
            value_ty: Box::new(V::type_to_idl(idl_definition)?),
            len_ty: Box::new(IdlTypeDef::U32),
        })
    }
}

impl<T: TypeToIdl, S> TypeToIdl for HashSet<T, S> {
    type AssociatedProgram = System;
    fn type_to_idl(idl_definition: &mut IdlDefinition) -> Result<IdlTypeDef> {
        <BTreeSet<T>>::type_to_idl(idl_definition)
    }
}

impl<K: TypeToIdl, V: TypeToIdl, S> TypeToIdl for HashMap<K, V, S> {
    type AssociatedProgram = System;
    fn type_to_idl(idl_definition: &mut IdlDefinition) -> Result<IdlTypeDef> {
        <BTreeMap<K, V>>::type_to_idl(idl_definition)
    }
}

macro_rules! impl_type_to_idl_for_tuple {
    ($($ty:ident),*) => {
        impl<$($ty: TypeToIdl),*> TypeToIdl for ($($ty,)*) {
            type AssociatedProgram = System;
            fn type_to_idl(idl_definition: &mut IdlDefinition) -> Result<IdlTypeDef> {
                Ok(IdlTypeDef::Struct(vec![
                    $(
                        IdlStructField {
                            path: None,
                            description: vec![],
                            type_def: $ty::type_to_idl(idl_definition)?,
                        },
                    )*
                ]))
            }
        }
    };
}

// Implement for tuples of size 1 to 16
impl_type_to_idl_for_tuple!(A);
impl_type_to_idl_for_tuple!(A, B);
impl_type_to_idl_for_tuple!(A, B, C);
impl_type_to_idl_for_tuple!(A, B, C, D);
impl_type_to_idl_for_tuple!(A, B, C, D, E);
impl_type_to_idl_for_tuple!(A, B, C, D, E, F);
impl_type_to_idl_for_tuple!(A, B, C, D, E, F, G);
impl_type_to_idl_for_tuple!(A, B, C, D, E, F, G, H);
impl_type_to_idl_for_tuple!(A, B, C, D, E, F, G, H, I);
impl_type_to_idl_for_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_type_to_idl_for_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_type_to_idl_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_type_to_idl_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_type_to_idl_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_type_to_idl_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_type_to_idl_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);

#[cfg(test)]
mod tests {
    // todo: add tests
}
