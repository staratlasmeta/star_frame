use bytemuck::{AnyBitPattern, CheckedBitPattern, NoUninit, Pod, Zeroable};
use derivative::Derivative;
use derive_more::From;
use num_traits::{FromPrimitive, ToPrimitive};
use star_frame::align1::Align1;
use std::fmt::Debug;

/// Packs a given `T` to be align 1.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Align1, Pod, Zeroable, Derivative, From)]
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
impl<T> FromPrimitive for PackedValue<T>
where
    T: FromPrimitive,
{
    fn from_isize(n: isize) -> Option<Self> {
        T::from_isize(n).map(Self)
    }

    fn from_i8(n: i8) -> Option<Self> {
        T::from_i8(n).map(Self)
    }

    fn from_i16(n: i16) -> Option<Self> {
        T::from_i16(n).map(Self)
    }

    fn from_i32(n: i32) -> Option<Self> {
        T::from_i32(n).map(Self)
    }

    fn from_i64(n: i64) -> Option<Self> {
        T::from_i64(n).map(Self)
    }

    fn from_i128(n: i128) -> Option<Self> {
        T::from_i128(n).map(Self)
    }

    fn from_usize(n: usize) -> Option<Self> {
        T::from_usize(n).map(Self)
    }

    fn from_u8(n: u8) -> Option<Self> {
        T::from_u8(n).map(Self)
    }

    fn from_u16(n: u16) -> Option<Self> {
        T::from_u16(n).map(Self)
    }

    fn from_u32(n: u32) -> Option<Self> {
        T::from_u32(n).map(Self)
    }

    fn from_u64(n: u64) -> Option<Self> {
        T::from_u64(n).map(Self)
    }

    fn from_u128(n: u128) -> Option<Self> {
        T::from_u128(n).map(Self)
    }

    fn from_f32(n: f32) -> Option<Self> {
        T::from_f32(n).map(Self)
    }

    fn from_f64(n: f64) -> Option<Self> {
        T::from_f64(n).map(Self)
    }
}
impl<T> ToPrimitive for PackedValue<T>
where
    T: ToPrimitive + Copy,
{
    fn to_isize(&self) -> Option<isize> {
        { self.0 }.to_isize()
    }

    fn to_i8(&self) -> Option<i8> {
        { self.0 }.to_i8()
    }

    fn to_i16(&self) -> Option<i16> {
        { self.0 }.to_i16()
    }

    fn to_i32(&self) -> Option<i32> {
        { self.0 }.to_i32()
    }

    fn to_i64(&self) -> Option<i64> {
        { self.0 }.to_i64()
    }

    fn to_i128(&self) -> Option<i128> {
        { self.0 }.to_i128()
    }

    fn to_usize(&self) -> Option<usize> {
        { self.0 }.to_usize()
    }

    fn to_u8(&self) -> Option<u8> {
        { self.0 }.to_u8()
    }

    fn to_u16(&self) -> Option<u16> {
        { self.0 }.to_u16()
    }

    fn to_u32(&self) -> Option<u32> {
        { self.0 }.to_u32()
    }

    fn to_u64(&self) -> Option<u64> {
        { self.0 }.to_u64()
    }

    fn to_u128(&self) -> Option<u128> {
        { self.0 }.to_u128()
    }

    fn to_f32(&self) -> Option<f32> {
        { self.0 }.to_f32()
    }

    fn to_f64(&self) -> Option<f64> {
        { self.0 }.to_f64()
    }
}

macro_rules! packed_eq {
    ($($ident:ident),* $(,)?) => {
        $(
            impl<T: Copy + PartialEq> PartialEq<T> for $ident<T> {
                fn eq(&self, other: &T) -> bool {
                    { self.0 }.eq(other)
                }
            }
        )*
    };
}
packed_eq!(PackedValue, PackedValueChecked);

/// Equivalent to [`PackedValue`] but [`CheckedBitPattern`] instead of [`Pod`].
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
unsafe impl<T> NoUninit for PackedValueChecked<T> where T: NoUninit {}
unsafe impl<T> Zeroable for PackedValueChecked<T> where T: Zeroable {}

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
