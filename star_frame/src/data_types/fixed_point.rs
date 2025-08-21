#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use crate::{idl::TypeToIdl, program::system::System};
    use fixed::*;
    use star_frame_idl::{ty::IdlTypeDef, IdlDefinition};
    use typenum::Unsigned;

    macro_rules! impl_type_to_idl_for_fixed {
        (@impl $ty:ident, $def_ident:ident) => {
            impl<Frac> TypeToIdl for $ty<Frac>
            where
                Frac: Unsigned,
            {
                type AssociatedProgram = System;

                fn type_to_idl(_idl_definition: &mut IdlDefinition) -> crate::Result<IdlTypeDef> {
                    Ok(IdlTypeDef::FixedPoint {
                        ty: Box::new(IdlTypeDef::$def_ident),
                        frac: Frac::U8,
                    })
                }
            }
        };
        ($([$ty:ident, $def_ident:ident]),* $(,)?) => {
            $(
                impl_type_to_idl_for_fixed!(@impl $ty, $def_ident);
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
}
