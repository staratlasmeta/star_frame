#![allow(clippy::extra_unused_type_parameters)]
use crate::prelude::*;
use derive_where::derive_where;
use num_traits::{real::Real, Pow};
use serde::{Deserialize, Serialize};
use std::{
    marker::PhantomData,
    ops::{Add, AddAssign, Div, Mul, Rem, Sub, SubAssign},
};
use typenum::{IsEqual, Mod, True, Unsigned, P2, Z0};

/// A value within a unit system.
#[derive(Serialize, Deserialize, Align1)]
#[derive_where(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default; T)]
#[serde(bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>"))]
#[serde(transparent)]
#[repr(transparent)]
pub struct UnitVal<T, Unit> {
    val: T,
    _unit: PhantomData<Unit>,
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
impl<T1, Unit1> UnitVal<T1, Unit1> {
    /// Puts this unit to the power of provided generic.
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
}

/// Marks that a given unit can be converted to a different unit system's unit.
pub trait Convert<Rhs> {}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl {
    use super::*;
    use crate::idl::TypeToIdl;
    use star_frame_idl::{ty::IdlTypeDef, IdlDefinition};
    impl<T: TypeToIdl, Unit> TypeToIdl for UnitVal<T, Unit> {
        type AssociatedProgram = T::AssociatedProgram;

        fn type_to_idl(idl_definition: &mut IdlDefinition) -> anyhow::Result<IdlTypeDef> {
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

/// Creates a new unit system type.
///
/// # Example
/// ```
/// # fn main() {} // This is needed to make the doctest work so modules aren't generated inside a function
/// use star_frame::create_unit_system;
/// use typenum::Z0;
/// // Creates a unit system with 3 axis.
/// create_unit_system!(struct CreatedUnitSystem<Seconds, Meters, Kilograms>);
///
/// // Creates a unit system with 2 axis and makes it convertable to `CreatedUnitSystem`.
/// // `==` can be replaced with `to` or `from` if unidirectional conversion is desired.
/// create_unit_system!(struct OtherUnitSystem<Seconds, Meters>{
///     impl<Seconds, Meters>: <Seconds, Meters> == CreatedUnitSystem<Seconds, Meters, Z0>,
/// });
///
/// // Creates a unit system with 2 axis and makes it convertable to `CreatedUnitSystem` and `OtherUnitSystem`.
/// create_unit_system!(struct ThirdUnitSystem<Seconds, Kilograms>{
///     impl<Seconds, Kilograms>: <Seconds, Kilograms> == CreatedUnitSystem<Seconds, Z0, Kilograms>,
///     impl<S>: <S, Z0> == OtherUnitSystem<S, Z0>,
/// });
/// ```
// TODO: Replace with proc macro for proper `IsEqual` impl
#[macro_export]
macro_rules! create_unit_system {
    (
        $vis:vis struct $ident:ident<$($unit:ident),+ $(,)?>{
            $(
                impl<$($gen:ident),* $(,)?>: <$($from:ty),* $(,)?> $op:tt $conv_ident:ident<$($to:ty),* $(,)?>
            ),* $(,)?
        }
    ) => {
        $crate::create_unit_system!($vis struct $ident<$($unit),+>);
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

    ($vis:vis struct $ident:ident<$($unit:ident),* $(,)?>) => {
        $crate::paste::paste!{
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
            $vis struct $ident<$($unit,)*>(::std::marker::PhantomData<($($unit,)*)>);

            $vis mod [<$ident:snake _units>] {
                use super::*;
                $crate::__unit_type_aliases!($vis $ident | $($unit)*);
            }

            unsafe impl<$($unit,)*> $crate::bytemuck::Zeroable for $ident<$($unit,)*>
            where
                $($unit: $crate::typenum::Integer,)*
            {}
            unsafe impl<$($unit,)*> $crate::bytemuck::Pod for $ident<$($unit,)*>
            where
                $($unit: $crate::typenum::Integer,)*
            {}
            #[automatically_derived]
            impl<$([<$unit 1>], [<$unit 2>],)*> ::std::ops::Add<$ident<$([<$unit 2>],)*>>
                for $ident<$([<$unit 1>],)*>
            where
                $(
                    [<$unit 1>]: $crate::typenum::Integer + ::std::ops::Add<[<$unit 2>]>,
                    [<$unit 2>]: $crate::typenum::Integer,
                    [<$unit 1>]::Output: $crate::typenum::Integer,
                )*
            {
                type Output = $ident<$([<$unit 1>]::Output,)*>;
                /// This trait implementation is solely used as trait bounds in `UnitVal` and the method isn't actually called
                fn add(
                    self,
                    _rhs: $ident<$([<$unit 2>],)*>,
                ) -> Self::Output {
                    ::std::panic!("Not implemented")
                }
            }
            #[automatically_derived]
            impl<$([<$unit 1>], [<$unit 2>],)*> ::std::ops::Sub<$ident<$([<$unit 2>],)*>>
                for $ident<$([<$unit 1>],)*>
            where
                $(
                    [<$unit 1>]: $crate::typenum::Integer + ::std::ops::Sub<[<$unit 2>]>,
                    [<$unit 2>]: $crate::typenum::Integer,
                    [<$unit 1>]::Output: $crate::typenum::Integer,
                )*
            {
                type Output = $ident<$([<$unit 1>]::Output,)*>;
                /// This trait implementation is solely used as trait bounds in `UnitVal` and the method isn't actually called
                fn sub(
                    self,
                    _rhs: $ident<$([<$unit 2>],)*>,
                ) -> Self::Output {
                    ::std::panic!("Not implemented")
                }
            }
            #[automatically_derived]
            impl<$($unit,)* Value> ::std::ops::Mul<Value> for $ident<$($unit,)*>
            where
                $(
                    $unit: $crate::typenum::Integer + ::std::ops::Mul<Value>,
                    $unit::Output: $crate::typenum::Integer,
                )*
            {
                type Output = $ident<$($unit::Output,)*>;
                /// This trait implementation is solely used as trait bounds in `UnitVal` and the method isn't actually called
                fn mul(self, _rhs: Value) -> Self::Output {
                    ::std::panic!("Not implemented")
                }
            }
            #[automatically_derived]
            impl<$($unit,)* Value> ::std::ops::Div<Value> for $ident<$($unit,)*>
            where
                $(
                    $unit: $crate::typenum::Integer + ::std::ops::Div<Value>,
                    $unit::Output: $crate::typenum::Integer,
                )*
            {
                type Output = $ident<$($unit::Output,)*>;
                /// This trait implementation is solely used as trait bounds in `UnitVal` and the method isn't actually called
                fn div(self, _rhs: Value) -> Self::Output {
                    ::std::panic!("Not implemented")
                }
            }
            #[automatically_derived]
            impl<$($unit,)* Value> ::std::ops::Rem<Value> for $ident<$($unit,)*>
            where
                $(
                    $unit: $crate::typenum::Integer + ::std::ops::Rem<Value>,
                    $unit::Output: $crate::typenum::Integer,
                )*
            {
                type Output = $ident<$($unit::Output,)*>;
                /// This trait implementation is solely used as trait bounds in `UnitVal` and the method isn't actually called
                fn rem(self, _rhs: Value) -> Self::Output {
                    ::std::panic!("Not implemented")
                }
            }
            #[automatically_derived]
            impl<$([<$unit 1>], [<$unit 2>],)*> $crate::typenum::IsEqual<$ident<$([<$unit 2>],)*>>
                for $ident<$([<$unit 1>],)*>
            where
                $(
                    [<$unit 1>]: $crate::typenum::Integer + $crate::typenum::IsEqual<[<$unit 2>], Output=$crate::typenum::True>,
                    [<$unit 2>]: $crate::typenum::Integer,
                )*

            {
                type Output = $crate::typenum::True;

                /// This trait implementation is solely used as trait bounds in `UnitVal` and the method isn't actually called
                fn is_equal(self, _rhs: $ident<$([<$unit 2>],)*>) -> Self::Output {
                    panic!("Not implemented")
                }
            }
        }
    };
}

#[cfg(test)]
mod test {
    use crate::data_types::UnitVal;
    use fixed::types::I53F11;
    use typenum::{Diff, Sum, N2, P1, Z0};

    create_unit_system!(struct CreatedUnitSystem<Seconds, Meters, Kilograms>);

    create_unit_system!(struct OtherUnitSystem<Seconds, Meters>{
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

    #[test]
    fn test_unit_system() {
        let value: UnitVal<_, Unitless> = UnitVal::new(10.0);
        let seconds: UnitVal<_, Seconds> = UnitVal::new(2.0);
        let meters: UnitVal<_, Meters> = UnitVal::new(4.0);
        let kilograms: UnitVal<_, Kilograms> = UnitVal::new(8.0);

        let speed = meters / seconds;
        assert_eq!(speed, UnitVal::<_, MetersPerSecond>::new(2.0));
        let acceleration = speed / seconds;
        assert_eq!(acceleration, UnitVal::<_, MetersPerSecondSquared>::new(1.0));
        let force: UnitVal<_, Newtons> = acceleration * kilograms;
        assert_eq!(force, UnitVal::<_, Newtons>::new(8.0));
        let multiplied = force * value;
        assert_eq!(multiplied, UnitVal::<_, Newtons>::new(80.0));

        let added = force + force;
        assert_eq!(added, UnitVal::<_, Newtons>::new(16.0));
        // let other = force + speed; // This does not compile

        let acceleration2 = added / kilograms;
        assert_eq!(
            acceleration2,
            UnitVal::<_, MetersPerSecondSquared>::new(2.0)
        );
        let converted = acceleration2.convert::<OtherUnitSystem<_, _>>();
        assert_eq!(converted, UnitVal::<_, OtherUnitSystem<N2, P1>>::new(2.0));
    }

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
    }
}
