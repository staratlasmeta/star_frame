use std::{
    cmp::Ordering,
    io::{Read, Write},
};

use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{AnyBitPattern, CheckedBitPattern, NoUninit, Pod, Zeroable};
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Sub, SubAssign};
use derive_more::From;
use num_traits::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use star_frame::align1::Align1;

/// Packs a given `T` to be align 1.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(
    Copy, Clone, Debug, Align1, Pod, PartialEq, Eq, PartialOrd, Ord, Zeroable, From, Default, Hash,
)]
#[repr(C, packed)]
pub struct PackedValue<T>(pub T);

/// Equivalent to [`PackedValue`] but [`CheckedBitPattern`] instead of [`Pod`].
#[derive(Copy, Clone, Debug, Align1, PartialEq, Eq, PartialOrd, Ord, From, Default, Hash)]
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

macro_rules! packed_ser_deser {
    ($ident:ident) => {
        impl<T> BorshSerialize for $ident<T>
        where
            T: BorshSerialize + Copy,
        {
            fn serialize<W: Write>(&self, writer: &mut W) -> borsh::io::Result<()> {
                { self.0 }.serialize(writer)
            }
        }

        impl<T> BorshDeserialize for $ident<T>
        where
            T: BorshDeserialize + Copy,
        {
            fn deserialize_reader<R: Read>(reader: &mut R) -> borsh::io::Result<Self> {
                T::deserialize_reader(reader).map(Self)
            }
        }

        impl<T> Serialize for $ident<T>
        where
            T: Serialize + Copy,
        {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                { self.0 }.serialize(serializer)
            }
        }

        impl<'de, T> Deserialize<'de> for $ident<T>
        where
            T: Deserialize<'de>,
        {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                T::deserialize(deserializer).map(Self)
            }
        }
    };
}

packed_ser_deser!(PackedValue);
packed_ser_deser!(PackedValueChecked);

macro_rules! packed_comparisons {
    ($ident:ident) => {
        impl<T: Copy + PartialEq> PartialEq<T> for $ident<T> {
            fn eq(&self, other: &T) -> bool {
                { self.0 }.eq(other)
            }
        }

        impl<T: Copy + PartialOrd> PartialOrd<T> for $ident<T> {
            fn partial_cmp(&self, other: &T) -> Option<Ordering> {
                { self.0 }.partial_cmp(other)
            }
        }
    };
}

packed_comparisons!(PackedValue);
packed_comparisons!(PackedValueChecked);

macro_rules! ref_arithmetic_impls {
    ($packed:ident $trait:ident: $($rhs:ty)*) => {
        $(
            paste::paste! {
                impl<T> [<$trait Assign>]<&$rhs> for $packed<T>
                where
                    T: $trait<T, Output = T> + Copy,
                {
                    fn [<$trait:lower _assign>](&mut self, rhs: &$rhs) {
                        [<$trait Assign>]::[<$trait:lower _assign>](self, *rhs);
                    }
                }

                impl<T> $trait<&$rhs> for $packed<T>
                where
                    T: $trait<T, Output = T> + Copy,
                {
                    type Output = $rhs;

                    fn [<$trait:lower>](self, rhs: &$rhs) -> Self::Output {
                        $trait::[<$trait:lower>](self, *rhs)
                    }
                }

                impl<T> $trait<$rhs> for &$packed<T>
                where
                    T: $trait<T, Output = T> + Copy,
                {
                    type Output = $rhs;
                    fn [<$trait:lower>](self, rhs: $rhs) -> Self::Output {
                        $trait::[<$trait:lower>](*self, rhs)
                    }
                }

                impl<T> $trait<&$rhs> for &$packed<T>
                where
                    T: $trait<T, Output = T> + Copy,
                {
                    type Output = $rhs;
                    fn [<$trait:lower>](self, rhs: &$rhs) -> Self::Output {
                        $trait::[<$trait:lower>](*self, *rhs)
                    }
                }
            }
        )*
    };
}

macro_rules! arithmetic_impls {
    ($packed:ident: $($trait:ident $op:tt)*) => {
        paste::paste! {
            $(
                impl<T> [<$trait Assign>]<T> for $packed<T>
                where
                    T: $trait<T, Output = T> + Copy,
                {
                    fn [<$trait:lower _assign>](&mut self, rhs: T) {
                        self.0 = self.0 $op rhs;
                    }
                }


                impl<T> [<$trait Assign>]<$packed<T>> for $packed<T>
                where
                    T: $trait<T, Output = T> + Copy,
                {
                    fn [<$trait:lower _assign>](&mut self, rhs: $packed<T>) {
                        self.0 = self.0 $op rhs.0;
                    }
                }

                impl<T> $trait<T> for $packed<T>
                where
                    T: $trait<T, Output = T> + Copy,
                {
                    type Output = T;
                    fn [<$trait:lower>](self, rhs: T) -> Self::Output {
                        self.0 $op rhs
                    }
                }

                impl<T> $trait<$packed<T>> for $packed<T>
                where
                    T: $trait<T, Output = T> + Copy,
                {
                    type Output = $packed<T>;
                    fn [<$trait:lower>](self, rhs: $packed<T>) -> Self::Output {
                        $packed(self.0 $op rhs.0)
                    }
                }

                ref_arithmetic_impls!($packed $trait: T $packed<T>);
            )*
        }
    };
}

arithmetic_impls!(PackedValue: Add + Sub - Mul * Div / Rem %);
arithmetic_impls!(PackedValueChecked: Add + Sub - Mul * Div / Rem %);

macro_rules! primitive_impls {
    ($packed:ident) => {
        impl<T> FromPrimitive for $packed<T>
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
        impl<T> ToPrimitive for $packed<T>
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
    };
}

primitive_impls!(PackedValue);
primitive_impls!(PackedValueChecked);

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use super::*;
    use crate::{idl::TypeToIdl, Result};
    use star_frame_idl::{ty::IdlTypeDef, IdlDefinition};

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
