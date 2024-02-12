use bytemuck::{AnyBitPattern, CheckedBitPattern, NoUninit, Pod, Zeroable};
use derivative::Derivative;
use star_frame::align1::Align1;
use std::fmt::Debug;

/// Packs a given `T` to be align 1.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Align1, Pod, Zeroable, Derivative)]
#[derivative(
    Debug(bound = "T: Debug + Copy"),
    Copy,
    Clone(bound = "T: Copy"),
    PartialEq,
    Eq,
    PartialOrd,
    Ord
)]
#[repr(C, packed)]
pub struct PackedValue<T>(pub T);

#[derive(Align1, Derivative)]
#[derivative(
    Debug(bound = "T: Debug + Copy"),
    Copy,
    Clone(bound = "T: Copy"),
    PartialEq,
    Eq,
    PartialOrd,
    Ord
)]
#[repr(C, packed)]
pub struct PackedValueChecked<T>(pub T);
unsafe impl<T> CheckedBitPattern for PackedValueChecked<T>
where
    T: CheckedBitPattern,
    PackedValue<T::Bits>: AnyBitPattern,
{
    type Bits = PackedValue<T::Bits>;

    fn is_valid_bit_pattern(bits: &Self::Bits) -> bool {
        T::is_valid_bit_pattern(&{ bits.0 })
    }
}

#[derive(Align1, Derivative, AnyBitPattern)]
#[derivative(
    Debug(bound = "T: Debug + Copy"),
    Copy,
    Clone(bound = "T: Copy"),
    PartialEq,
    Eq,
    PartialOrd,
    Ord
)]
#[repr(C, packed)]
pub struct PackedValueAnyBitPattern<T>(pub T);
unsafe impl<T> NoUninit for PackedValueAnyBitPattern<T> where T: NoUninit {}

#[cfg(feature = "idl")]
mod idl_impl {
    use super::*;
    use crate::idl::ty::TypeToIdl;
    use crate::Result;
    use star_frame_idl::ty::IdlTypeDef;
    use star_frame_idl::IdlDefinition;

    impl<T> TypeToIdl for PackedValue<T>
    where
        T: TypeToIdl,
    {
        type AssociatedProgram = T::AssociatedProgram;

        fn type_to_idl(idl_definition: &mut IdlDefinition) -> Result<IdlTypeDef> {
            T::type_to_idl(idl_definition)
        }
    }
}
