use crate::idl::ty::TypeToIdl;
use crate::program::system_program::SystemProgram;
use fixed::*;
use star_frame_idl::ty::{IdlDefinedType, IdlTypeDef};
use star_frame_idl::IdlDefinition;
use typenum::Unsigned;

macro_rules! impl_type_to_idl_for_fixed {
    (@impl $ty:ident, $defined_ident:ident) => {
        impl<Frac> TypeToIdl for $ty<Frac>
        where
            Frac: Unsigned,
        {
            type AssociatedProgram = SystemProgram;

            fn type_to_idl(_idl_definition: &mut IdlDefinition) -> crate::Result<IdlTypeDef> {
                Ok(IdlTypeDef::FixedPoint {
                    ty: IdlDefinedType::$defined_ident,
                    frac: Frac::U8,
                })
            }
        }
    };
    ($([$ty:ident, $defined_ident:ident]),* $(,)?) => {
        $(
            impl_type_to_idl_for_fixed!(@impl $ty, $defined_ident);
        )*
    };
}
impl_type_to_idl_for_fixed!(
    [FixedU8, U8],
    [FixedU16, U16],
    [FixedU32, U32],
    [FixedU64, U64],
    [FixedU128, U128],
    [FixedI8, I8],
    [FixedI16, I16],
    [FixedI32, I32],
    [FixedI64, I64],
    [FixedI128, I128],
);
