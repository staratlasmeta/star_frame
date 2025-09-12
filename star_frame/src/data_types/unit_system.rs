#![allow(clippy::extra_unused_type_parameters)]
use crate::prelude::*;
use derive_where::derive_where;
use fixed::traits::{Fixed, FromFixed, ToFixed};
use num_traits::{
    real::Real, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, ConstZero, Pow, SaturatingAdd,
    SaturatingMul, SaturatingSub, Zero,
};
use pinocchio::sysvars::clock::Clock;
use serde::{Deserialize, Serialize};
use std::{
    marker::PhantomData,
    ops::{Add, AddAssign, Div, Mul, Neg, Rem, Sub, SubAssign},
};
use typenum::{IsEqual, Mod, True, Unsigned, P2, Z0};

/// Strongly typed values with units from [`create_unit_system`].
///
/// # Example
/// ```
/// use star_frame::{create_unit_system, data_types::UnitVal};
/// use typenum::{Diff, Sum, N2, P1, Z0};
///
/// // First create a unit system
/// create_unit_system!(struct CreatedUnitSystem<Seconds, Meters, Kilograms>);
/// // (and a couple more)
/// create_unit_system!(struct OtherUnitSystem<Seconds, Meters>{
///     impl<Seconds, Meters>: <Seconds, Meters> == CreatedUnitSystem<Seconds, Meters, Z0>
/// });
/// create_unit_system!(struct ThirdUnitSystem<Seconds, Kilograms>{
///     impl<Seconds, Kilograms>: <Seconds, Kilograms> == CreatedUnitSystem<Seconds, Z0, Kilograms>,
///     impl<S>: <S, Z0> to OtherUnitSystem<S, Z0>
/// });
///
/// use created_unit_system_units::{Kilograms, Meters, Seconds, Unitless};
/// type MetersPerSecond = Diff<Meters, Seconds>;
/// type MetersPerSecondSquared = Diff<MetersPerSecond, Seconds>;
/// type Newtons = Sum<MetersPerSecondSquared, Kilograms>;
///
/// # fn main() {
/// // Use `UnitVal` to create values with units
/// let value: UnitVal<_, Unitless> = UnitVal::new(10.0);
/// let seconds: UnitVal<_, Seconds> = UnitVal::new(2.0);
/// let meters: UnitVal<_, Meters> = UnitVal::new(4.0);
/// let kilograms: UnitVal<_, Kilograms> = UnitVal::new(8.0);
///
/// let speed = meters / seconds;
/// assert_eq!(speed, UnitVal::<_, MetersPerSecond>::new(2.0));
///
/// let acceleration = speed / seconds;
/// assert_eq!(acceleration, UnitVal::<_, MetersPerSecondSquared>::new(1.0));
///
/// let force: UnitVal<_, Newtons> = acceleration * kilograms;
/// assert_eq!(force, UnitVal::<_, Newtons>::new(8.0));
///
/// let multiplied = force * value;
/// assert_eq!(multiplied, UnitVal::<_, Newtons>::new(80.0));
///
/// let added = force + force;
/// assert_eq!(added, UnitVal::<_, Newtons>::new(16.0));
///
/// // let other = force + speed; // This does not compile
///
/// let acceleration2 = added / kilograms;
/// assert_eq!(
///     acceleration2,
///     UnitVal::<_, MetersPerSecondSquared>::new(2.0)
/// );
///
/// // You can convert between unit systems, too.
/// let converted = acceleration2.convert::<OtherUnitSystem<_, _>>();
/// assert_eq!(converted, UnitVal::<_, OtherUnitSystem<N2, P1>>::new(2.0));
/// # }
/// ```
///
/// # An invalid operation will not compile
///
/// ```compile_fail
/// # use star_frame::{create_unit_system, data_types::UnitVal};
/// # use typenum::{Diff, Sum, N2, P1, Z0};
/// create_unit_system!(struct CreatedUnitSystem<Florps, Glorps>);
/// use created_unit_system_units::{Florps, Glorps, Unitless};
/// type FlorpsPerGloop = Diff<Florps, Glorps>;
/// # fn main() {
/// let florps: UnitVal<_, Florps> = UnitVal::new(10.0);
/// let glorps: UnitVal<_, Glorps> = UnitVal::new(2.0);
///
/// let florps_per_glorp = florps / glorps;
/// assert_eq!(florps_per_glorp, UnitVal::<_, FlorpsPerGloop>::new(5.0));
/// // Compile error
/// let invalid = florps_per_glorp + glorps;
/// # }
#[derive(Serialize, Deserialize, Align1, BorshSerialize, BorshDeserialize)]
#[derive_where(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default; T)]
#[serde(bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>"))]
#[serde(transparent)]
#[repr(transparent)]
pub struct UnitVal<T, Unit> {
    val: T,
    _unit: PhantomData<Unit>,
}
impl<T, Unit> From<T> for UnitVal<T, Unit> {
    fn from(val: T) -> Self {
        Self::new(val)
    }
}
unsafe impl<T, Unit> Zeroable for UnitVal<T, Unit> where T: Zeroable {}
unsafe impl<T, Unit> Pod for UnitVal<T, Unit>
where
    T: Pod,
    Unit: 'static,
{
}
impl<T, Unit> UnitVal<T, Unit> {
    /// Creates a new [`UnitVal`] from a given value.
    /// Note: This will make any unit unless generics are specified.
    pub const fn new(val: T) -> Self {
        Self {
            val,
            _unit: PhantomData,
        }
    }
    /// Extracts the inner value
    pub fn val(self) -> T {
        self.val
    }
    /// Gets a reference to the value.
    pub fn val_ref(&self) -> &T {
        &self.val
    }
    /// Gets a mutable reference to the value.
    pub fn val_mut(&mut self) -> &mut T {
        &mut self.val
    }
}
impl<T, Unit> Neg for UnitVal<T, Unit>
where
    T: Neg,
{
    type Output = UnitVal<T::Output, Unit>;

    fn neg(self) -> Self::Output {
        UnitVal::new(-self.val)
    }
}
impl<T1, T2, Unit1, Unit2> Add<UnitVal<T2, Unit2>> for UnitVal<T1, Unit1>
where
    T1: Add<T2>,
    Unit1: IsEqual<Unit2, Output = True>,
{
    type Output = UnitVal<T1::Output, Unit1>;

    fn add(self, rhs: UnitVal<T2, Unit2>) -> Self::Output {
        UnitVal::new(self.val + rhs.val)
    }
}
impl<T1, T2, Unit1, Unit2> AddAssign<UnitVal<T2, Unit2>> for UnitVal<T1, Unit1>
where
    T1: AddAssign<T2>,
    Unit1: IsEqual<Unit2, Output = True>,
{
    fn add_assign(&mut self, rhs: UnitVal<T2, Unit2>) {
        self.val += rhs.val;
    }
}
impl<T1, T2, Unit1, Unit2> Sub<UnitVal<T2, Unit2>> for UnitVal<T1, Unit1>
where
    T1: Sub<T2>,
    Unit1: IsEqual<Unit2, Output = True>,
{
    type Output = UnitVal<T1::Output, Unit1>;

    fn sub(self, rhs: UnitVal<T2, Unit2>) -> Self::Output {
        UnitVal::new(self.val - rhs.val)
    }
}
impl<T1, T2, Unit1, Unit2> SubAssign<UnitVal<T2, Unit2>> for UnitVal<T1, Unit1>
where
    T1: SubAssign<T2>,
    Unit1: IsEqual<Unit2, Output = True>,
{
    fn sub_assign(&mut self, rhs: UnitVal<T2, Unit2>) {
        self.val -= rhs.val;
    }
}
impl<T1, T2, Unit1, Unit2> Mul<UnitVal<T2, Unit2>> for UnitVal<T1, Unit1>
where
    T1: Mul<T2>,
    Unit1: Add<Unit2>,
{
    type Output = UnitVal<T1::Output, Unit1::Output>;

    fn mul(self, rhs: UnitVal<T2, Unit2>) -> Self::Output {
        UnitVal::new(self.val * rhs.val)
    }
}
impl<T1, T2, Unit1, Unit2> Div<UnitVal<T2, Unit2>> for UnitVal<T1, Unit1>
where
    T1: Div<T2>,
    Unit1: Sub<Unit2>,
{
    type Output = UnitVal<T1::Output, Unit1::Output>;

    fn div(self, rhs: UnitVal<T2, Unit2>) -> Self::Output {
        UnitVal::new(self.val / rhs.val)
    }
}
impl<T1, T2, Unit1, Unit2> Rem<UnitVal<T2, Unit2>> for UnitVal<T1, Unit1>
where
    T1: Rem<T2>,
{
    type Output = UnitVal<T1::Output, Unit1>;

    fn rem(self, rhs: UnitVal<T2, Unit2>) -> Self::Output {
        UnitVal::new(self.val % rhs.val)
    }
}
impl<T, Unit> Zero for UnitVal<T, Unit>
where
    T: Zero,
    Self: Add<Self, Output = Self>,
{
    fn zero() -> Self {
        Self::new(T::zero())
    }

    fn set_zero(&mut self) {
        self.val.set_zero();
    }

    fn is_zero(&self) -> bool {
        self.val.is_zero()
    }
}
impl<T, Unit> ConstZero for UnitVal<T, Unit>
where
    T: ConstZero,
    Self: Add<Self, Output = Self>,
{
    const ZERO: Self = Self::new(T::ZERO);
}
impl<T1, Unit1> UnitVal<T1, Unit1> {
    /// Puts this unit to the power of the provided generic.
    pub fn pow<Value>(self) -> UnitVal<T1::Output, Unit1::Output>
    where
        Value: Unsigned,
        T1: Pow<u32>,
        Unit1: Mul<Value>,
    {
        UnitVal::new(self.val.pow(Value::U32))
    }

    /// Gets the square root of this unit val. Only works for units that are a power of 2.
    pub fn sqrt(self) -> UnitVal<T1, <Unit1 as Div<P2>>::Output>
    where
        T1: Real,
        Z0: IsEqual<Mod<Unit1, P2>, Output = True>,
        Unit1: Rem<P2> + Div<P2>,
    {
        UnitVal::new(self.val.sqrt())
    }

    /// Converts between two unit systems.
    pub fn convert<Unit2>(self) -> UnitVal<T1, Unit2>
    where
        Unit1: Convert<Unit2>,
    {
        UnitVal::new(self.val)
    }

    /// Checked addition. Returns `None` if overflow occurred.
    /// Only works with values of the same unit.
    #[must_use]
    pub fn checked_add<Unit2>(self, rhs: &UnitVal<T1, Unit2>) -> Option<Self>
    where
        T1: CheckedAdd,
        Unit1: IsEqual<Unit2, Output = True>,
    {
        self.val.checked_add(&rhs.val).map(UnitVal::new)
    }

    /// Saturating addition. Clamps the result to the type's bounds instead of overflowing.
    /// Only works with values of the same unit.
    #[must_use]
    pub fn saturating_add<Unit2>(self, rhs: &UnitVal<T1, Unit2>) -> Self
    where
        T1: SaturatingAdd,
        Unit1: IsEqual<Unit2, Output = True>,
    {
        UnitVal::new(self.val.saturating_add(&rhs.val))
    }

    /// Checked subtraction. Returns `None` if overflow occurred.
    /// Only works with values of the same unit.
    #[must_use]
    pub fn checked_sub<Unit2>(self, rhs: &UnitVal<T1, Unit2>) -> Option<Self>
    where
        T1: CheckedSub,
        Unit1: IsEqual<Unit2, Output = True>,
    {
        self.val.checked_sub(&rhs.val).map(UnitVal::new)
    }

    /// Saturating subtraction. Clamps the result to the type's bounds instead of overflowing.
    /// Only works with values of the same unit.
    #[must_use]
    pub fn saturating_sub<Unit2>(self, rhs: &UnitVal<T1, Unit2>) -> Self
    where
        T1: SaturatingSub,
        Unit1: IsEqual<Unit2, Output = True>,
    {
        UnitVal::new(self.val.saturating_sub(&rhs.val))
    }

    /// Checked multiplication. Returns `None` if overflow occurred.
    /// The resulting unit is the sum of the input units.
    #[must_use]
    pub fn checked_mul<Unit2>(
        self,
        rhs: &UnitVal<T1, Unit2>,
    ) -> Option<UnitVal<T1, <Unit1 as Add<Unit2>>::Output>>
    where
        T1: CheckedMul,
        Unit1: Add<Unit2>,
    {
        self.val.checked_mul(&rhs.val).map(UnitVal::new)
    }

    /// Saturating multiplication. Clamps the result to the type's bounds instead of overflowing.
    /// The resulting unit is the sum of the input units.
    #[must_use]
    pub fn saturating_mul<Unit2>(
        self,
        rhs: &UnitVal<T1, Unit2>,
    ) -> UnitVal<T1, <Unit1 as Add<Unit2>>::Output>
    where
        T1: SaturatingMul,
        Unit1: Add<Unit2>,
    {
        UnitVal::new(self.val.saturating_mul(&rhs.val))
    }

    /// Checked division. Returns `None` if the divisor is zero or if overflow occurred.
    /// The resulting unit is the difference of the input units.
    #[must_use]
    pub fn checked_div<Unit2>(
        self,
        rhs: &UnitVal<T1, Unit2>,
    ) -> Option<UnitVal<T1, <Unit1 as Sub<Unit2>>::Output>>
    where
        T1: CheckedDiv,
        Unit1: Sub<Unit2>,
    {
        self.val.checked_div(&rhs.val).map(UnitVal::new)
    }

    /// Applies a function to the inner value, preserving the unit.
    #[must_use]
    pub fn map<O>(self, f: impl FnOnce(T1) -> O) -> UnitVal<O, Unit1> {
        UnitVal::new(f(self.val))
    }

    /// Applies a function that returns a tuple to the inner value,
    /// returning a tuple of `UnitVal`s with the same unit.
    #[must_use]
    pub fn map_tuple<O1, O2>(
        self,
        f: impl FnOnce(T1) -> (O1, O2),
    ) -> (UnitVal<O1, Unit1>, UnitVal<O2, Unit1>) {
        let (o1, o2) = f(self.val);
        (UnitVal::new(o1), UnitVal::new(o2))
    }

    /// Applies a function that returns an `Option` to the inner value,
    /// preserving the unit if the result is `Some`.
    #[must_use]
    pub fn map_optional<O>(self, f: impl FnOnce(T1) -> Option<O>) -> Option<UnitVal<O, Unit1>> {
        f(self.val).map(UnitVal::new)
    }

    /// Applies a fallible function to the inner value, preserving the unit on success.
    pub fn try_map<E, O>(self, f: impl FnOnce(T1) -> Result<O, E>) -> Result<UnitVal<O, Unit1>, E> {
        f(self.val).map(UnitVal::new)
    }

    /// Creates a `UnitVal` containing a reference to the inner value.
    pub fn as_ref(&self) -> UnitVal<&T1, Unit1> {
        UnitVal::new(&self.val)
    }

    /// Creates a `UnitVal` containing a mutable reference to the inner value.
    pub fn as_mut(&mut self) -> UnitVal<&mut T1, Unit1> {
        UnitVal::new(&mut self.val)
    }
}
impl<F, Unit1> UnitVal<F, Unit1>
where
    F: Fixed,
{
    /// See [`Fixed::from_num`]
    pub fn from_num<Src: ToFixed>(src: UnitVal<Src, Unit1>) -> Self {
        src.map(F::from_num)
    }

    /// See [`Fixed::to_num`]
    pub fn to_num<Dst: FromFixed>(self) -> UnitVal<Dst, Unit1> {
        self.map(F::to_num)
    }

    /// See [`Fixed::checked_from_num`]
    pub fn checked_from_num<Src: ToFixed>(src: UnitVal<Src, Unit1>) -> Option<Self> {
        src.map_optional(F::checked_from_num)
    }

    /// See [`Fixed::checked_to_num`]
    pub fn checked_to_num<Dst: FromFixed>(self) -> Option<UnitVal<Dst, Unit1>> {
        self.map_optional(F::checked_to_num)
    }

    /// See [`Fixed::saturating_from_num`]
    pub fn saturating_from_num<Src: ToFixed>(src: UnitVal<Src, Unit1>) -> Self {
        src.map(F::saturating_from_num)
    }

    /// See [`Fixed::saturating_to_num`]
    pub fn saturating_to_num<Dst: FromFixed>(self) -> UnitVal<Dst, Unit1> {
        self.map(F::saturating_to_num)
    }

    /// See [`Fixed::wrapping_from_num`]
    pub fn wrapping_from_num<Src: ToFixed>(src: UnitVal<Src, Unit1>) -> Self {
        src.map(F::wrapping_from_num)
    }

    /// See [`Fixed::wrapping_to_num`]
    pub fn wrapping_to_num<Dst: FromFixed>(self) -> UnitVal<Dst, Unit1> {
        self.map(F::wrapping_to_num)
    }

    /// See [`Fixed::unwrapped_from_num`]
    pub fn unwrapped_from_num<Src: ToFixed>(src: UnitVal<Src, Unit1>) -> Self {
        src.map(F::unwrapped_from_num)
    }

    /// See [`Fixed::unwrapped_to_num`]
    pub fn unwrapped_to_num<Dst: FromFixed>(self) -> UnitVal<Dst, Unit1> {
        self.map(F::unwrapped_to_num)
    }

    /// See [`Fixed::overflowing_from_num`]. Returns a tuple of the converted value and a boolean indicating overflow.
    pub fn overflowing_from_num<Src: ToFixed>(src: UnitVal<Src, Unit1>) -> (Self, bool) {
        let (val, overflow) = src.map_tuple(F::overflowing_from_num);
        (val, overflow.val)
    }

    /// See [`Fixed::overflowing_to_num`]. Returns a tuple of the converted value and a boolean indicating overflow.
    pub fn overflowing_to_num<Dst: FromFixed>(self) -> (UnitVal<Dst, Unit1>, bool) {
        let (val, overflow) = self.map_tuple(F::overflowing_to_num);
        (val, overflow.val)
    }

    /// See [`Fixed::ceil`]
    #[must_use]
    pub fn ceil(self) -> Self {
        self.map(F::ceil)
    }

    /// See [`Fixed::floor`]
    #[must_use]
    pub fn floor(self) -> Self {
        self.map(F::floor)
    }

    /// See [`Fixed::round`]
    #[must_use]
    pub fn round(self) -> Self {
        self.map(F::round)
    }

    /// See [`Fixed::round_ties_even`]
    #[must_use]
    pub fn round_ties_even(self) -> Self {
        self.map(F::round_ties_even)
    }

    /// See [`Fixed::checked_ceil`]
    #[must_use]
    pub fn checked_ceil(self) -> Option<Self> {
        self.map_optional(F::checked_ceil)
    }

    /// See [`Fixed::checked_floor`]
    #[must_use]
    pub fn checked_floor(self) -> Option<Self> {
        self.map_optional(F::checked_floor)
    }

    /// See [`Fixed::checked_round`]
    #[must_use]
    pub fn checked_round(self) -> Option<Self> {
        self.map_optional(F::checked_round)
    }

    /// See [`Fixed::checked_round_ties_even`]
    #[must_use]
    pub fn checked_round_ties_even(self) -> Option<Self> {
        self.map_optional(F::checked_round_ties_even)
    }

    /// See [`Fixed::saturating_ceil`]
    #[must_use]
    pub fn saturating_ceil(self) -> Self {
        self.map(F::saturating_ceil)
    }

    /// See [`Fixed::saturating_floor`]
    #[must_use]
    pub fn saturating_floor(self) -> Self {
        self.map(F::saturating_floor)
    }

    /// See [`Fixed::saturating_round`]
    #[must_use]
    pub fn saturating_round(self) -> Self {
        self.map(F::saturating_round)
    }

    /// See [`Fixed::saturating_round_ties_even`]
    #[must_use]
    pub fn saturating_round_ties_even(self) -> Self {
        self.map(F::saturating_round_ties_even)
    }

    /// See [`Fixed::wrapping_ceil`]
    #[must_use]
    pub fn wrapping_ceil(self) -> Self {
        self.map(F::wrapping_ceil)
    }

    /// See [`Fixed::wrapping_floor`]
    #[must_use]
    pub fn wrapping_floor(self) -> Self {
        self.map(F::wrapping_floor)
    }

    /// See [`Fixed::wrapping_round`]
    #[must_use]
    pub fn wrapping_round(self) -> Self {
        self.map(F::wrapping_round)
    }

    /// See [`Fixed::wrapping_round_ties_even`]
    #[must_use]
    pub fn wrapping_round_ties_even(self) -> Self {
        self.map(F::wrapping_round_ties_even)
    }

    /// See [`Fixed::unwrapped_ceil`]
    #[must_use]
    pub fn unwrapped_ceil(self) -> Self {
        self.map(F::unwrapped_ceil)
    }

    /// See [`Fixed::unwrapped_floor`]
    #[must_use]
    pub fn unwrapped_floor(self) -> Self {
        self.map(F::unwrapped_floor)
    }

    /// See [`Fixed::unwrapped_round`]
    #[must_use]
    pub fn unwrapped_round(self) -> Self {
        self.map(F::unwrapped_round)
    }

    /// See [`Fixed::unwrapped_round_ties_even`]
    #[must_use]
    pub fn unwrapped_round_ties_even(self) -> Self {
        self.map(F::unwrapped_round_ties_even)
    }

    /// See [`Fixed::overflowing_ceil`]. Returns a tuple of the ceiling value and a boolean indicating overflow.
    pub fn overflowing_ceil(self) -> (Self, bool) {
        let (val, overflow) = self.map_tuple(F::overflowing_ceil);
        (val, overflow.val)
    }

    /// See [`Fixed::overflowing_floor`]. Returns a tuple of the floor value and a boolean indicating overflow.
    pub fn overflowing_floor(self) -> (Self, bool) {
        let (val, overflow) = self.map_tuple(F::overflowing_floor);
        (val, overflow.val)
    }

    /// See [`Fixed::overflowing_round`]. Returns a tuple of the rounded value and a boolean indicating overflow.
    pub fn overflowing_round(self) -> (Self, bool) {
        let (val, overflow) = self.map_tuple(F::overflowing_round);
        (val, overflow.val)
    }

    /// See [`Fixed::overflowing_round_ties_even`]. Returns a tuple of the rounded value and a boolean indicating overflow.
    pub fn overflowing_round_ties_even(self) -> (Self, bool) {
        let (val, overflow) = self.map_tuple(F::overflowing_round_ties_even);
        (val, overflow.val)
    }
}

/// Marks that a given unit can be converted to a different unit system's unit.
#[doc(hidden)]
pub trait Convert<Rhs> {}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl {
    use super::*;
    use crate::idl::TypeToIdl;
    use star_frame_idl::{ty::IdlTypeDef, IdlDefinition};
    impl<T: TypeToIdl, Unit> TypeToIdl for UnitVal<T, Unit> {
        type AssociatedProgram = T::AssociatedProgram;

        fn type_to_idl(idl_definition: &mut IdlDefinition) -> eyre::Result<IdlTypeDef> {
            T::type_to_idl(idl_definition)
        }
    }
}

/// A helper macro to create unit type aliases
#[doc(hidden)]
#[macro_export]
macro_rules! __unit_type_aliases {
    (@ty $ident:ident $($unit:ident <$ty:ty>)*) => {
        $ident<$($ty),*>
    };
    // Base case
    ($vis:vis $ident:ident $($zeros:ident)* |) => {
        #[allow(unused)]
        pub type Unitless = $crate::__unit_type_aliases!(@ty $ident $($zeros <$crate::typenum::Z0>)*);
    };
    ($vis:vis $ident:ident $($zeros:ident)* | $p1:ident $($end_zeros:ident)*) => {
        #[allow(unused)]
        pub type $p1 = $crate::__unit_type_aliases!(@ty $ident $($zeros <$crate::typenum::Z0>)* $p1 <$crate::typenum::P1> $($end_zeros <$crate::typenum::Z0>)*);
        $crate::__unit_type_aliases!($vis $ident $($zeros)* $p1 | $($end_zeros)*);
    };

}

pub trait ClockExt {
    fn unix_timestamp_unit<Seconds: IsSeconds>(&self) -> UnitVal<i64, Seconds>;
}
impl ClockExt for Clock {
    fn unix_timestamp_unit<Seconds: IsSeconds>(&self) -> UnitVal<i64, Seconds> {
        UnitVal::new(self.unix_timestamp)
    }
}

pub trait IsSeconds {}

/// Creates a new unit system type.
///
/// # Example
/// ```
/// # fn main() {} // This is needed to make the doctest work so modules aren't generated inside a function
/// use star_frame::create_unit_system;
/// use typenum::Z0;
/// // Creates a unit system with 3 axis.
/// create_unit_system!(struct CreatedUnitSystem<@seconds Seconds, Meters, Kilograms>);
///
/// // Creates a unit system with 2 axis and makes it convertable to `CreatedUnitSystem`.
/// // `==` can be replaced with `to` or `from` if unidirectional conversion is desired.
/// create_unit_system!(struct OtherUnitSystem<@seconds Seconds, Meters>{
///     impl<Seconds, Meters>: <Seconds, Meters> == CreatedUnitSystem<Seconds, Meters, Z0>,
/// });
///
/// // Creates a unit system with 2 axis and makes it convertable to `CreatedUnitSystem` and `OtherUnitSystem`.
/// create_unit_system!(struct ThirdUnitSystem<@seconds Seconds, Kilograms>{
///     impl<Seconds, Kilograms>: <Seconds, Kilograms> == CreatedUnitSystem<Seconds, Z0, Kilograms>,
///     impl<S>: <S, Z0> == OtherUnitSystem<S, Z0>,
/// });
/// ```
// TODO: Replace with proc macro for proper `IsEqual` impl
#[macro_export]
macro_rules! create_unit_system {
    (
        $vis:vis struct $ident:ident<$($(@$seconds:ident)? $unit:ident),+ $(,)?>{
            $(
                impl<$($gen:ident),* $(,)?>: <$($from:ty),* $(,)?> $op:tt $conv_ident:ident<$($to:ty),* $(,)?>
            ),* $(,)?
        }
    ) => {
        $crate::create_unit_system!($vis struct $ident<$($(@$seconds)? $unit),+>);
        $(
            $crate::create_unit_system!(@convert $ident <$($gen,)*>: <$($from,)*> $op $conv_ident<$($to,)*>);
        )*
    };

    (@convert $ident:ident <$($gen:ident),* $(,)?>: <$($from:ty),* $(,)?> == $conv_ident:ident<$($to:ty),* $(,)?>) => {
        $crate::create_unit_system!(@convert $ident <$($gen,)*>: <$($from,)*> to $conv_ident<$($to,)*>);
        $crate::create_unit_system!(@convert $ident <$($gen,)*>: <$($from,)*> from $conv_ident<$($to,)*>);
    };
    (@convert $ident:ident <$($gen:ident),* $(,)?>: <$($from:ty),* $(,)?> to $conv_ident:ident<$($to:ty),* $(,)?>) => {
        impl<$($gen,)*> $crate::data_types::Convert<$conv_ident<$($to,)*>> for $ident<$($from,)*>{}
    };
    (@convert $ident:ident <$($gen:ident),* $(,)?>: <$($from:ty),* $(,)?> from $conv_ident:ident<$($to:ty),* $(,)?>) => {
        impl<$($gen,)*> $crate::data_types::Convert<$ident<$($from,)*>> for $conv_ident<$($to,)*>{}
    };

    (@impl @seconds $unit:ident) => {
        impl $crate::data_types::IsSeconds for $unit {}
    };

    ($vis:vis struct $ident:ident<$($(@$seconds:ident)? $unit:ident),+ $(,)?>) => {$crate::paste::paste!{
        #[allow(unused_imports)]
        mod [<_serde_ $ident:snake>] {
            pub(super) use $crate::serde as _serde_unit_system;
        }
        #[allow(unused_imports)]
        use [<_serde_ $ident:snake>]::*;
        #[derive(
            $crate::serde::Serialize,
            $crate::serde::Deserialize,
            $crate::align1::Align1,
            $crate::derive_where::DeriveWhere,
        )]
        #[serde(bound = "", crate = "_serde_unit_system")]
        #[derive_where(Copy, Clone, Default, Debug, PartialEq, Eq)]
        #[repr(transparent)]
        $vis struct $ident<$($unit,)+>(::std::marker::PhantomData<($($unit,)+)>);

        $vis mod [<$ident:snake _units>] {
            use super::*;
            $crate::__unit_type_aliases!($vis $ident | $($unit)+);

            $($($crate::create_unit_system!(@impl @$seconds $unit);)?)+
        }

        unsafe impl<$($unit,)+> $crate::bytemuck::Zeroable for $ident<$($unit,)+>
        where
            $($unit: $crate::typenum::Integer,)+
        {}
        unsafe impl<$($unit,)+> $crate::bytemuck::Pod for $ident<$($unit,)+>
        where
            $($unit: $crate::typenum::Integer,)+
        {}
        #[automatically_derived]
        impl<$([<$unit 1>], [<$unit 2>],)+> ::std::ops::Add<$ident<$([<$unit 2>],)+>>
            for $ident<$([<$unit 1>],)+>
        where
            $(
                [<$unit 1>]: $crate::typenum::Integer + ::std::ops::Add<[<$unit 2>]>,
                [<$unit 2>]: $crate::typenum::Integer,
                [<$unit 1>]::Output: $crate::typenum::Integer,
            )+
        {
            type Output = $ident<$([<$unit 1>]::Output,)+>;
            /// This trait implementation is solely used as trait bounds in `UnitVal` and the method isn't actually called
            fn add(
                self,
                _rhs: $ident<$([<$unit 2>],)+>,
            ) -> Self::Output {
                ::std::panic!("Not implemented")
            }
        }
        #[automatically_derived]
        impl<$([<$unit 1>], [<$unit 2>],)+> ::std::ops::Sub<$ident<$([<$unit 2>],)+>>
            for $ident<$([<$unit 1>],)+>
        where
            $(
                [<$unit 1>]: $crate::typenum::Integer + ::std::ops::Sub<[<$unit 2>]>,
                [<$unit 2>]: $crate::typenum::Integer,
                [<$unit 1>]::Output: $crate::typenum::Integer,
            )+
        {
            type Output = $ident<$([<$unit 1>]::Output,)+>;
            /// This trait implementation is solely used as trait bounds in `UnitVal` and the method isn't actually called
            fn sub(
                self,
                _rhs: $ident<$([<$unit 2>],)+>,
            ) -> Self::Output {
                ::std::panic!("Not implemented")
            }
        }
        #[automatically_derived]
        impl<$($unit,)+ Value> ::std::ops::Mul<Value> for $ident<$($unit,)+>
        where
            $(
                $unit: $crate::typenum::Integer + ::std::ops::Mul<Value>,
                $unit::Output: $crate::typenum::Integer,
            )+
        {
            type Output = $ident<$($unit::Output,)+>;
            /// This trait implementation is solely used as trait bounds in `UnitVal` and the method isn't actually called
            fn mul(self, _rhs: Value) -> Self::Output {
                ::std::panic!("Not implemented")
            }
        }
        #[automatically_derived]
        impl<$($unit,)+ Value> ::std::ops::Div<Value> for $ident<$($unit,)+>
        where
            $(
                $unit: $crate::typenum::Integer + ::std::ops::Div<Value>,
                $unit::Output: $crate::typenum::Integer,
            )+
        {
            type Output = $ident<$($unit::Output,)+>;
            /// This trait implementation is solely used as trait bounds in `UnitVal` and the method isn't actually called
            fn div(self, _rhs: Value) -> Self::Output {
                ::std::panic!("Not implemented")
            }
        }
        #[automatically_derived]
        impl<$($unit,)+ Value> ::std::ops::Rem<Value> for $ident<$($unit,)+>
        where
            $(
                $unit: $crate::typenum::Integer + ::std::ops::Rem<Value>,
                $unit::Output: $crate::typenum::Integer,
            )+
        {
            type Output = $ident<$($unit::Output,)+>;
            /// This trait implementation is solely used as trait bounds in `UnitVal` and the method isn't actually called
            fn rem(self, _rhs: Value) -> Self::Output {
                ::std::panic!("Not implemented")
            }
        }
        #[automatically_derived]
        impl<$([<$unit 1>], [<$unit 2>],)+> $crate::typenum::IsEqual<$ident<$([<$unit 2>],)+>>
            for $ident<$([<$unit 1>],)+>
        where
            $(
                [<$unit 1>]: $crate::typenum::Integer + $crate::typenum::IsEqual<[<$unit 2>], Output=$crate::typenum::True>,
                [<$unit 2>]: $crate::typenum::Integer,
            )+

        {
            type Output = $crate::typenum::True;

            /// This trait implementation is solely used as trait bounds in `UnitVal` and the method isn't actually called
            fn is_equal(self, _rhs: $ident<$([<$unit 2>],)+>) -> Self::Output {
                panic!("Not implemented")
            }
        }
    }};
}

#[cfg(test)]
mod test {
    use crate::data_types::{ClockExt, UnitVal};
    use fixed::types::I53F11;
    use pinocchio::sysvars::clock::Clock;
    use typenum::{Diff, Sum, Z0};

    create_unit_system!(struct CreatedUnitSystem<@seconds Seconds, Meters, Kilograms>);

    create_unit_system!(struct OtherUnitSystem<@seconds Seconds, Meters>{
        impl<Seconds, Meters>: <Seconds, Meters> == CreatedUnitSystem<Seconds, Meters, Z0>
    });

    create_unit_system!(struct ThirdUnitSystem<Seconds, Kilograms>{
        impl<Seconds, Kilograms>: <Seconds, Kilograms> == CreatedUnitSystem<Seconds, Z0, Kilograms>,
        impl<S>: <S, Z0> to OtherUnitSystem<S, Z0>
    });

    use created_unit_system_units::{Kilograms, Meters, Seconds, Unitless};

    type MetersPerSecond = Diff<Meters, Seconds>;
    type MetersPerSecondSquared = Diff<MetersPerSecond, Seconds>;
    type Newtons = Sum<MetersPerSecondSquared, Kilograms>;

    type Fixed = I53F11;

    #[test]
    fn test_fixed_point() {
        let value: UnitVal<_, Unitless> = UnitVal::new(Fixed::from(10));
        let seconds: UnitVal<_, Seconds> = UnitVal::new(Fixed::from(2));
        let meters: UnitVal<_, Meters> = UnitVal::new(Fixed::from(4));
        let kilograms: UnitVal<_, Kilograms> = UnitVal::new(Fixed::from(8));

        let speed = meters / seconds;
        assert_eq!(speed, UnitVal::<_, MetersPerSecond>::new(Fixed::from(2)));
        let acceleration = speed / seconds;
        assert_eq!(
            acceleration,
            UnitVal::<_, MetersPerSecondSquared>::new(Fixed::from(1))
        );
        let force: UnitVal<_, Newtons> = acceleration * kilograms;
        assert_eq!(force, UnitVal::<_, Newtons>::new(Fixed::from(8)));
        let multiplied = force * value;
        assert_eq!(multiplied, UnitVal::<_, Newtons>::new(Fixed::from(80)));

        let added = force + force;
        assert_eq!(added, UnitVal::<_, Newtons>::new(Fixed::from(16)));
        // let other = force + speed; // This does not compile

        let clock = Clock {
            slot: 0,
            epoch_start_timestamp: 0,
            epoch: 0,
            leader_schedule_epoch: 0,
            unix_timestamp: 145,
        };
        let timestamp = clock.unix_timestamp_unit();
        assert_eq!(timestamp, UnitVal::<_, Seconds>::new(145));
    }
}
