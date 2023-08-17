use array_init::map_array_init;
use bytemuck::{Pod, Zeroable};
use num_traits::ToPrimitive;
use solana_program::clock::Clock;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

/// A unit for a fixed point value
pub trait UnitType: Copy + Debug + Eq + Ord + Hash + 'static {
    /// Helper function to display unit information in the Display trait
    fn display() -> String;
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Default unit representing unitless-ness
pub struct Unitless;
impl UnitType for Unitless {
    fn display() -> String {
        String::default()
    }
}

/// Defines a custom unit type
#[macro_export]
macro_rules! define_unit {
    {$($(#[$meta:meta])* $name:ident;)*} => {
        $(
            $(#[$meta])*
            #[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
            pub struct $name;
            impl $crate::UnitType for $name {
                fn display() -> String {
                    stringify!($name).to_string()
                }
            }
        )*
    };
}

macro_rules! fixed_point_value {
    ($($(#[$struct_meta:meta])* $struct_name:ident: $value_ty:ident;)*) => {
        $(
            $(#[$struct_meta])*
            #[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
            #[repr(transparent)]
            pub struct $struct_name<Unit: UnitType, const DIV: $value_ty>($value_ty, PhantomData<fn() -> Unit>);

            impl<Unit: UnitType, const DIV: $value_ty> std::fmt::Display for $struct_name<Unit, DIV> {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    let unit_display = Unit::display();
                    if unit_display.is_empty() {
                        write!(f, "{}", self.0 as f64 / DIV as f64)
                    } else {
                        write!(f, "{} {}", self.0 as f64 / DIV as f64, Unit::display())
                    }
                }
            }

            #[allow(
                clippy::cast_precision_loss,
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss
            )]
            impl<Unit: UnitType, const DIV: $value_ty> $struct_name<Unit, DIV> {
                #[doc = "Converts a value to fixed point representation"]
                #[must_use]
                pub fn new(val: $value_ty) -> Self {
                    Self(val * DIV, PhantomData)
                }

                #[doc = "Converts a value to fixed point representation from the raw value"]
                #[must_use]
                pub fn from_raw(val: $value_ty) -> Self {
                    Self(val, PhantomData)
                }

                #[doc = "Converts this to the raw value"]
                #[must_use]
                pub fn to_raw(self) -> $value_ty {
                    self.0
                }

                #[doc = "Converts this to the float value"]
                #[must_use]
                pub fn to_float(self) -> FloatWithUnit<Unit> {
                    FloatWithUnit::from_raw(self.0 as f64 / DIV as f64)
                }

                #[doc = "Converts a float value to the fixed point representation"]
                #[must_use]
                pub fn from_float(val: FloatWithUnit<Unit>) -> Self {
                    Self((val.raw_value() * DIV as f64) as $value_ty, PhantomData)
                }

                #[doc = "Converts this to another fixed point representation using an [`f64`] as the intermediate value"]
                #[must_use]
                pub fn convert_to_div<const DIV_TO: $value_ty>(self) -> $struct_name<Unit, DIV_TO> {
                    $struct_name::from_float(FloatWithUnit::from_raw(self.0 as f64 * DIV_TO as f64 / DIV as f64))
                }
            }
            impl<Unit: UnitType, const DIV: $value_ty> Add for $struct_name<Unit, DIV> {
                type Output = $struct_name<Unit, DIV>;

                fn add(self, rhs: $struct_name<Unit, DIV>) -> Self::Output {
                    Self(self.0.checked_add(rhs.0).unwrap(), PhantomData)
                }
            }
            impl<Unit: UnitType, const DIV: $value_ty> AddAssign for $struct_name<Unit, DIV> {
                fn add_assign(&mut self, rhs: $struct_name<Unit, DIV>) {
                    self.0 = self.0.checked_add(rhs.0).unwrap();
                }
            }
            impl<Unit: UnitType, const DIV: $value_ty> Sub for $struct_name<Unit, DIV> {
                type Output = $struct_name<Unit, DIV>;

                fn sub(self, rhs: $struct_name<Unit, DIV>) -> Self::Output {
                    Self(self.0.checked_sub(rhs.0).unwrap(), PhantomData)
                }
            }
            impl<Unit: UnitType, const DIV: $value_ty> SubAssign for $struct_name<Unit, DIV> {
                fn sub_assign(&mut self, rhs: $struct_name<Unit, DIV>) {
                    self.0 = self.0.checked_sub(rhs.0).unwrap();
                }
            }
            impl<Unit: UnitType, const DIV: $value_ty> Mul<$value_ty> for $struct_name<Unit, DIV> {
                type Output = $struct_name<Unit, DIV>;

                fn mul(self, rhs: $value_ty) -> Self::Output {
                    Self(self.0.checked_mul(rhs).unwrap(), PhantomData)
                }
            }
            impl<Unit: UnitType, const DIV: $value_ty> MulAssign<$value_ty> for $struct_name<Unit, DIV> {
                fn mul_assign(&mut self, rhs: $value_ty) {
                    self.0 = self.0.checked_mul(rhs).unwrap();
                }
            }
            impl<Unit: UnitType, const DIV: $value_ty> Div<$value_ty> for $struct_name<Unit, DIV> {
                type Output = $struct_name<Unit, DIV>;

                fn div(self, rhs: $value_ty) -> Self::Output {
                    Self(self.0.checked_div(rhs).unwrap(), PhantomData)
                }
            }
            impl<Unit: UnitType, const DIV: $value_ty> DivAssign<$value_ty> for $struct_name<Unit, DIV> {
                fn div_assign(&mut self, rhs: $value_ty) {
                    self.0 = self.0.checked_div(rhs).unwrap();
                }
            }
            // Safety: Safe because $value_ty impls `Zeroable` and fixed point is transparent to it.
            unsafe impl<Unit: UnitType, const DIV: $value_ty> Zeroable for $struct_name<Unit, DIV> {
                fn zeroed() -> Self {
                    Self($value_ty::zeroed(), PhantomData)
                }
            }
            // Safety: Safe because $value_ty impls `Pod` and fixed point is transparent to it.
            unsafe impl<Unit: UnitType, const DIV: $value_ty> Pod for $struct_name<Unit, DIV> {}

            impl<Unit: UnitType, const DIV: $value_ty, const N: usize> FixedPointArrayToFloat<N> for [$struct_name<Unit, DIV>; N] {
                type Unit = Unit;

                fn fixed_point_array_to_float(self) -> [FloatWithUnit<Self::Unit>; N] {
                    map_array_init(&self, |v| v.to_float())
                }
            }
        )*
    };
}

fixed_point_value! {
    /// Fixed point [`u8`] value.
    FixedPointU8: u8;
    /// Fixed point [`u16`] value.
    FixedPointU16: u16;
    /// Fixed point [`u32`] value.
    FixedPointU32: u32;
    /// Fixed point [`u64`] value.
    FixedPointU64: u64;
    /// Fixed point [`u128`] value.
    FixedPointU128: u128;
    /// Fixed point [`i8`] value.
    FixedPointI8: i8;
    /// Fixed point [`i16`] value.
    FixedPointI16: i16;
    /// Fixed point [`i32`] value.
    FixedPointI32: i32;
    /// Fixed point [`i64`] value.
    FixedPointI64: i64;
    /// Fixed point [`i128`] value.
    FixedPointI128: i128;
}

/// Converts a fixed point array to a float array
pub trait FixedPointArrayToFloat<const N: usize> {
    /// The unit of the fixed point value
    type Unit: UnitType;
    /// Converts this array to a float
    fn fixed_point_array_to_float(self) -> [FloatWithUnit<Self::Unit>; N];
}

macro_rules! to_fixed {
    {$unit:ident: {$($func_name:ident<$fixed_ty:ty> -> $fixed_ident:ident;)*}} => {
        $(
            /// Converts this to a fixed point value
            #[must_use]
            pub fn $func_name<const DIV: $fixed_ty>(self) -> $fixed_ident<$unit, DIV> {
                $fixed_ident::from_float(self)
            }
        )*
    };
}

define_unit! {
    /// Seconds
    Second;
}

/// Fixed point value from a clock
pub trait FromClock {
    /// Gets the unix timestamp of the clock in seconds
    fn timestamp_unit(&self) -> FixedPointI64<Second, 1>;
    /// Gets the unix timestamp of the clock in seconds in y64
    fn timestamp_unit_u64(&self) -> FixedPointU64<Second, 1>;
}
impl FromClock for Clock {
    fn timestamp_unit(&self) -> FixedPointI64<Second, 1> {
        FixedPointI64::from_raw(self.unix_timestamp)
    }
    fn timestamp_unit_u64(&self) -> FixedPointU64<Second, 1> {
        FixedPointU64::from_raw(self.unix_timestamp.to_u64().unwrap())
    }
}

/// A floating point number with associated unit.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct FloatWithUnit<Unit: UnitType>(f64, PhantomData<fn() -> Unit>);
impl<Unit: UnitType> Display for FloatWithUnit<Unit> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let unit_display = Unit::display();
        if unit_display.is_empty() {
            write!(f, "{}", self.0)
        } else {
            write!(f, "{} {}", self.0, unit_display)
        }
    }
}
impl<Unit: UnitType> FloatWithUnit<Unit> {
    /// Creates a new [`FloatWithUnit`] from the given value.
    #[must_use]
    pub fn from_raw(val: f64) -> Self {
        Self(val, PhantomData)
    }

    /// Gets the raw contained float
    #[must_use]
    pub fn raw_value(self) -> f64 {
        self.0
    }

    to_fixed! { Unit: {
        to_fixed_u8<u8> -> FixedPointU8;
        to_fixed_u16<u16> -> FixedPointU16;
        to_fixed_u32<u32> -> FixedPointU32;
        to_fixed_u64<u64> -> FixedPointU64;
        to_fixed_u128<u128> -> FixedPointU128;
        to_fixed_i8<i8> -> FixedPointI8;
        to_fixed_i16<i16> -> FixedPointI16;
        to_fixed_i32<i32> -> FixedPointI32;
        to_fixed_i64<i64> -> FixedPointI64;
        to_fixed_i128<i128> -> FixedPointI128;
    }}

    /// Divides this by the same unit to give a unitless value.
    #[must_use]
    pub fn div_by_self(self, other: FloatWithUnit<Unit>) -> FloatWithUnit<Unitless> {
        FloatWithUnit::from_raw(self.0 / other.0)
    }

    /// Multiplies this by the a unitless value to give the same unit.
    #[must_use]
    pub fn mul_by_unitless(self, other: FloatWithUnit<Unitless>) -> FloatWithUnit<Unit> {
        FloatWithUnit::from_raw(self.0 * other.0)
    }

    /// Divides by a unitless value to give the same unit.
    #[must_use]
    pub fn div_by_unitless(self, other: FloatWithUnit<Unitless>) -> FloatWithUnit<Unit> {
        FloatWithUnit::from_raw(self.0 / other.0)
    }

    /// Divides by a unit with the same unit the numerator
    #[must_use]
    pub fn div_and_cancel<U2: UnitType>(
        self,
        other: FloatWithUnit<DivUnit<Unit, U2>>,
    ) -> FloatWithUnit<U2> {
        FloatWithUnit::from_raw(self.0 / other.0)
    }
}
impl<U1: UnitType, U2: UnitType> FloatWithUnit<MulUnit<U1, U2>> {
    /// Runs an operation on the first index of a multiplied unit. Treats the other value as 1
    #[must_use]
    pub fn op_unit1<UO: UnitType>(
        self,
        operation: impl FnOnce(FloatWithUnit<U1>) -> FloatWithUnit<UO>,
    ) -> FloatWithUnit<MulUnit<UO, U2>> {
        FloatWithUnit::from_raw(operation(FloatWithUnit::from_raw(self.0)).0)
    }

    /// Runs an operation on the second index of a multiplied unit. Treats the other value as 1
    #[must_use]
    pub fn op_unit2<UO: UnitType>(
        self,
        operation: impl FnOnce(FloatWithUnit<U2>) -> FloatWithUnit<UO>,
    ) -> FloatWithUnit<MulUnit<U1, UO>> {
        FloatWithUnit::from_raw(operation(FloatWithUnit::from_raw(self.0)).0)
    }

    /// Flips the order of the units using the commutative property of multiplication
    #[must_use]
    pub fn flip_mul(self) -> FloatWithUnit<MulUnit<U2, U1>> {
        FloatWithUnit::from_raw(self.0)
    }

    /// Divides this by the first unit index
    #[must_use]
    pub fn div_by_u1(self, other: FloatWithUnit<U1>) -> FloatWithUnit<U2> {
        FloatWithUnit(self.0 / other.0, PhantomData)
    }

    /// Divides this by the second unit index
    #[must_use]
    pub fn div_by_u2(self, other: FloatWithUnit<U2>) -> FloatWithUnit<U1> {
        FloatWithUnit(self.0 / other.0, PhantomData)
    }
}
impl<U1: UnitType, U2: UnitType> FloatWithUnit<DivUnit<U1, U2>> {
    /// Multiplies by a strong typed unit with the same unit as the denominator
    #[must_use]
    pub fn mul_by_u2(self, other: FloatWithUnit<U2>) -> FloatWithUnit<U1> {
        FloatWithUnit(self.0 * other.0, PhantomData)
    }

    /// Multiplies by a strong typed unit with the same unit as the numerator
    #[must_use]
    pub fn mul_by_u1(
        self,
        other: FloatWithUnit<U1>,
    ) -> FloatWithUnit<DivUnit<MulUnit<U1, U1>, U2>> {
        FloatWithUnit(self.0 * other.0, PhantomData)
    }

    /// Divide by a strong typed unit with the same unit as the numerator
    #[must_use]
    pub fn div_by_u1(self, other: FloatWithUnit<U1>) -> FloatWithUnit<DivUnit<Unitless, U2>> {
        FloatWithUnit(self.0 / other.0, PhantomData)
    }

    /// Divide by a strong typed unit with the same unit as the denominator
    #[must_use]
    pub fn div_by_u2(
        self,
        other: FloatWithUnit<U2>,
    ) -> FloatWithUnit<DivUnit<U1, MulUnit<U2, U2>>> {
        FloatWithUnit(self.0 / other.0, PhantomData)
    }

    /// Runs an operation on the numerator of a divided unit. Treats the other value as 1
    #[must_use]
    pub fn op_numerator<UO: UnitType>(
        self,
        operation: impl FnOnce(FloatWithUnit<U1>) -> FloatWithUnit<UO>,
    ) -> FloatWithUnit<DivUnit<UO, U2>> {
        FloatWithUnit::from_raw(operation(FloatWithUnit::from_raw(self.0)).0)
    }

    /// Runs an operation on the denominator of a divided unit. Treats the other value as 1
    #[must_use]
    pub fn op_denominator<UO: UnitType>(
        self,
        operation: impl FnOnce(FloatWithUnit<U2>) -> FloatWithUnit<UO>,
    ) -> FloatWithUnit<DivUnit<U1, UO>> {
        FloatWithUnit::from_raw(operation(FloatWithUnit::from_raw(self.0.recip())).0.recip())
    }

    /// Flips the order of the units by taking the reciprocal
    #[must_use]
    pub fn reciprocal_of_div(self) -> FloatWithUnit<DivUnit<U2, U1>> {
        FloatWithUnit::from_raw(self.0.recip())
    }

    /// Multiplies by the denominator
    #[must_use]
    pub fn mul_by_denominator(self, other: FloatWithUnit<U2>) -> FloatWithUnit<U1> {
        FloatWithUnit(self.0 * other.0, PhantomData)
    }
}
impl<Unit: UnitType> FloatWithUnit<MulUnit<Unit, Unit>> {
    /// Takes the square root of a self multiplied unit
    #[must_use]
    pub fn sqrt(self) -> FloatWithUnit<Unit> {
        FloatWithUnit::from_raw(self.0.sqrt())
    }
}
impl<Unit: UnitType> FloatWithUnit<DivUnit<Unit, Unit>> {
    /// Simplifies a div of itself to a unitless value
    #[must_use]
    pub fn simplify_div(self) -> FloatWithUnit<Unitless> {
        FloatWithUnit(self.0, PhantomData)
    }
}
impl<Unit: UnitType> FloatWithUnit<MulUnit<Unitless, Unit>> {
    /// Simplifies a mul of unitless to the same unit
    #[must_use]
    pub fn simplify_unitless_mul1(self) -> FloatWithUnit<Unit> {
        FloatWithUnit(self.0, PhantomData)
    }
}
impl<Unit: UnitType> FloatWithUnit<MulUnit<Unit, Unitless>> {
    /// Simplifies a mul of unitless to the same unit
    #[must_use]
    pub fn simplify_unitless_mul2(self) -> FloatWithUnit<Unit> {
        FloatWithUnit(self.0, PhantomData)
    }
}
impl<Unit: UnitType> FloatWithUnit<DivUnit<Unit, Unitless>> {
    /// Simplifies a div of unitless to the same unit
    #[must_use]
    pub fn simplify_unitless_div(self) -> FloatWithUnit<Unit> {
        FloatWithUnit(self.0, PhantomData)
    }
}
impl<U1: UnitType, U2: UnitType> FloatWithUnit<MulUnit<DivUnit<U1, U2>, U2>> {
    /// Simplifies a multiply of a divide with the same units
    #[must_use]
    pub fn simplify_mul_of_div1(self) -> FloatWithUnit<U1> {
        FloatWithUnit(self.0, PhantomData)
    }
}
impl<U1: UnitType, U2: UnitType> FloatWithUnit<MulUnit<U2, DivUnit<U1, U2>>> {
    /// Simplifies a multiply of a divide with the same units
    #[must_use]
    pub fn simplify_mul_of_div2(self) -> FloatWithUnit<U1> {
        FloatWithUnit(self.0, PhantomData)
    }
}
impl<U1: UnitType, U2: UnitType> FloatWithUnit<DivUnit<MulUnit<U1, U2>, U1>> {
    /// Simplifies a divide of a multiply with the same units
    #[must_use]
    pub fn simplify_div_of_mul1(self) -> FloatWithUnit<U2> {
        FloatWithUnit(self.0, PhantomData)
    }
}
impl<U1: UnitType, U2: UnitType> FloatWithUnit<DivUnit<MulUnit<U1, U2>, U2>> {
    /// Simplifies a divide of a multiply with the same units
    #[must_use]
    pub fn simplify_div_of_mul2(self) -> FloatWithUnit<U1> {
        FloatWithUnit(self.0, PhantomData)
    }
}
impl<U1: UnitType, U2: UnitType, U3: UnitType> FloatWithUnit<DivUnit<DivUnit<U1, U2>, U3>> {
    /// Simplifies a divide of a divide
    #[must_use]
    pub fn simplify_div_of_numerator_div(self) -> FloatWithUnit<DivUnit<U1, MulUnit<U2, U3>>> {
        FloatWithUnit(self.0, PhantomData)
    }
}
impl<U1: UnitType, U2: UnitType, U3: UnitType> FloatWithUnit<DivUnit<U1, DivUnit<U2, U3>>> {
    /// Simplifies a divide of a divide
    #[must_use]
    pub fn simplify_div_of_denominator_div(self) -> FloatWithUnit<DivUnit<MulUnit<U1, U3>, U2>> {
        FloatWithUnit(self.0, PhantomData)
    }
}
impl<U1: UnitType, U2: UnitType, U3: UnitType> FloatWithUnit<MulUnit<MulUnit<U1, U2>, U3>> {
    /// Associates multiplication right
    #[must_use]
    pub fn assoc_right(self) -> FloatWithUnit<MulUnit<U1, MulUnit<U2, U3>>> {
        FloatWithUnit(self.0, PhantomData)
    }
}
impl<U1: UnitType, U2: UnitType, U3: UnitType> FloatWithUnit<MulUnit<U1, MulUnit<U2, U3>>> {
    /// Associates multiplication left
    #[must_use]
    pub fn assoc_left(self) -> FloatWithUnit<MulUnit<MulUnit<U1, U2>, U3>> {
        FloatWithUnit(self.0, PhantomData)
    }
}

impl<Unit: UnitType> Add for FloatWithUnit<Unit> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0, PhantomData)
    }
}
impl<Unit: UnitType> AddAssign for FloatWithUnit<Unit> {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}
impl<Unit: UnitType> Sub for FloatWithUnit<Unit> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0, PhantomData)
    }
}
impl<Unit: UnitType> SubAssign for FloatWithUnit<Unit> {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}
impl<U1: UnitType, U2: UnitType> Mul<FloatWithUnit<U2>> for FloatWithUnit<U1> {
    type Output = FloatWithUnit<MulUnit<U1, U2>>;

    fn mul(self, rhs: FloatWithUnit<U2>) -> Self::Output {
        FloatWithUnit::from_raw(self.0 * rhs.0)
    }
}
impl<U1: UnitType, U2: UnitType> Div<FloatWithUnit<U2>> for FloatWithUnit<U1> {
    type Output = FloatWithUnit<DivUnit<U1, U2>>;

    fn div(self, rhs: FloatWithUnit<U2>) -> Self::Output {
        FloatWithUnit::from_raw(self.0 / rhs.0)
    }
}

/// Two units multiplied together
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct MulUnit<U1: UnitType, U2: UnitType>(PhantomData<fn() -> (U1, U2)>);
impl<U1: UnitType, U2: UnitType> UnitType for MulUnit<U1, U2> {
    fn display() -> String {
        let u1 = U1::display();
        let u2 = U2::display();
        if u2.is_empty() || u1.is_empty() {
            format!("{u1}{u2}")
        } else {
            format!("({u1} * {u2})")
        }
    }
}

/// Two units divided together
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct DivUnit<U1: UnitType, U2: UnitType>(PhantomData<fn() -> (U1, U2)>);
impl<U1: UnitType, U2: UnitType> UnitType for DivUnit<U1, U2> {
    fn display() -> String {
        let u1 = U1::display();
        let u2 = U2::display();
        if u2.is_empty() {
            u1
        } else if u1.is_empty() {
            format!("/ {u2}")
        } else {
            format!("({u1} / {u2})")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_display() {
        define_unit! {
            Smoot;
        }
        let ten_smoot_per_second_squared =
            FloatWithUnit::<DivUnit<Smoot, MulUnit<Second, Second>>>::from_raw(10.0);
        assert_eq!(
            "10 (Smoot / (Second * Second))",
            ten_smoot_per_second_squared.to_string()
        );

        let ten_per_smoot = FloatWithUnit::<DivUnit<Unitless, Smoot>>::from_raw(10.0);
        assert_eq!("10 / Smoot", ten_per_smoot.to_string());

        let ten_smoot = FixedPointU32::<MulUnit<Unitless, Smoot>, 1>::from_raw(10);
        assert_eq!("10 Smoot", ten_smoot.to_string());

        let ten_useless_unitless = FloatWithUnit::<DivUnit<Unitless, Unitless>>::from_raw(10.0);
        assert_eq!("10", ten_useless_unitless.to_string());
    }

    #[test]
    fn fixed_point_raw() {
        let fixed_point = FixedPointU32::<Second, 100>::new(2);
        assert_eq!(200, fixed_point.to_raw());

        let fixed_point = FixedPointI64::<Second, 100>::new(-2);
        assert_eq!(-200, fixed_point.to_raw());

        let fixed_point = FixedPointI128::<Second, 100>::from_raw(2);

        assert_eq!(2, fixed_point.to_raw());
    }

    #[test]
    fn fixed_zeroed() {
        let fixed_point = FixedPointU32::<Second, 100>::zeroed();
        assert_eq!(0, fixed_point.to_raw());

        let fixed_point = FixedPointI64::<Second, 100>::zeroed();
        assert_eq!(0, fixed_point.to_raw());

        let fixed_point = FixedPointI128::<Second, 100>::zeroed();
        assert_eq!(0, fixed_point.to_raw());
    }

    #[test]
    fn fixed_point_add() {
        let mut addend = FixedPointU32::<Second, 100>::new(2);
        let augend = FixedPointU32::<Second, 100>::new(3);
        let expected_sum = FixedPointU32::<Second, 100>::new(5);
        assert_eq!(expected_sum, addend + augend);
        assert_eq!(expected_sum, {
            addend += augend;
            addend
        });

        let mut addend = FixedPointI64::<Second, 100>::new(2);
        let augend = FixedPointI64::<Second, 100>::new(3);
        let expected_sum = FixedPointI64::<Second, 100>::new(5);
        assert_eq!(expected_sum, addend + augend);
        assert_eq!(expected_sum, {
            addend += augend;
            addend
        });
    }

    #[test]
    fn fixed_point_sub() {
        let mut minuend = FixedPointU32::<Second, 100>::new(5);
        let subtrahend = FixedPointU32::<Second, 100>::new(2);
        let expected_sum = FixedPointU32::<Second, 100>::new(3);
        assert_eq!(expected_sum, minuend - subtrahend);
        assert_eq!(expected_sum, {
            minuend -= subtrahend;
            minuend
        });

        let mut minuend = FixedPointI64::<Second, 100>::new(5);
        let subtrahend = FixedPointI64::<Second, 100>::new(2);
        let expected_sum = FixedPointI64::<Second, 100>::new(3);
        assert_eq!(expected_sum, minuend - subtrahend);
        assert_eq!(expected_sum, {
            minuend -= subtrahend;
            minuend
        });
    }

    #[test]
    fn fixed_point_mul() {
        let mut multiplicand = FixedPointU32::<Second, 100>::new(2);
        let multiplier = 3;
        let expected_product = FixedPointU32::<Second, 100>::new(6);
        assert_eq!(expected_product, multiplicand * multiplier);
        assert_eq!(expected_product, {
            multiplicand *= multiplier;
            multiplicand
        });

        let mut multiplicand = FixedPointI64::<Second, 100>::new(2);
        let multiplier = 3;
        let expected_product = FixedPointI64::<Second, 100>::new(6);
        assert_eq!(expected_product, multiplicand * multiplier);
        assert_eq!(expected_product, {
            multiplicand *= multiplier;
            multiplicand
        });
    }

    #[test]
    fn fixed_point_div() {
        let mut dividend = FixedPointU32::<Second, 100>::new(8);
        let divisor = 4;
        let expected_quotient = FixedPointU32::<Second, 100>::new(2);
        assert_eq!(expected_quotient, dividend / divisor);
        assert_eq!(expected_quotient, {
            dividend /= divisor;
            dividend
        });

        let mut dividend = FixedPointI64::<Second, 100>::new(8);
        let divisor = 4;
        let expected_quotient = FixedPointI64::<Second, 100>::new(2);
        assert_eq!(expected_quotient, dividend / divisor);
        assert_eq!(expected_quotient, {
            dividend /= divisor;
            dividend
        });
    }

    #[test]
    fn fixed_mul_by_denominator() {
        let multiplicand = FixedPointU32::<DivUnit<Unitless, Second>, 100>::new(8);
        assert_eq!(800, multiplicand.to_raw());
        let multiplier = FixedPointU32::<Second, 100>::new(4);
        assert_eq!(400, multiplier.to_raw());
        let expected_product = FixedPointU32::<Unitless, 100>::new(32);
        assert_eq!(3200, expected_product.to_raw());
        assert_eq!(
            expected_product.to_float(),
            multiplicand
                .to_float()
                .mul_by_denominator(multiplier.to_float())
        );
    }

    #[test]
    fn fixed_div_and_cancel() {
        let dividend = FixedPointU32::<Second, 10>::new(16);
        assert_eq!(160, dividend.to_raw());
        let divisor = FixedPointU32::<DivUnit<Second, Unitless>, 5>::new(2);
        assert_eq!(10, divisor.to_raw());
        let expected_quotient = FixedPointU32::<Unitless, 20>::new(8);

        assert_eq!(
            expected_quotient.to_float(),
            dividend.to_float().div_and_cancel(divisor.to_float())
        );
    }
}
