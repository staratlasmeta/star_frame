use crate::data_types::PodBool;
use crate::idl::TypeToIdl;
use crate::program::system_program::System;
use crate::Result;
use solana_program::pubkey::Pubkey;
use star_frame_idl::ty::IdlStructField;
use star_frame_idl::ty::IdlTypeDef;
use star_frame_idl::IdlDefinition;

macro_rules! impl_type_to_idl_for_primitive {
    (@impl $ty:ty: $ident:ident) => {
        impl $crate::idl::TypeToIdl for $ty {
            type AssociatedProgram = $crate::program::system_program::System;

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
        Ok(IdlTypeDef::Option(Box::new(T::type_to_idl(
            idl_definition,
        )?)))
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
            len_ty: Box::new(u32::type_to_idl(idl_definition)?),
        })
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
