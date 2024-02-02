#![allow(clippy::extra_unused_type_parameters)]
use bytemuck::{Pod, Zeroable};
use num_traits::real::Real;
use num_traits::Pow;
use serde::{Deserialize, Serialize};
use star_frame_proc::Align1;
use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Div, Mul, Rem, Sub, SubAssign};
use typenum::{IsEqual, Mod, True, Unsigned, P2, Z0};

#[derive(
    Copy,
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Pod,
    Zeroable,
    Serialize,
    Deserialize,
    Align1,
)]
#[repr(transparent)]
pub struct UnitVal<T, Unit> {
    val: T,
    _unit: PhantomData<Unit>,
}
impl<T, Unit> UnitVal<T, Unit> {
    pub const fn new(val: T) -> Self {
        Self {
            val,
            _unit: PhantomData,
        }
    }
    pub fn val(self) -> T
    where
        T: Copy,
    {
        self.val
    }
    pub fn val_ref(&self) -> &T {
        &self.val
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
    pub fn pow<Value>(self) -> UnitVal<T1::Output, Unit1::Output>
    where
        Value: Unsigned,
        T1: Pow<u32>,
        Unit1: Mul<Value>,
    {
        UnitVal::new(self.val.pow(Value::U32))
    }

    pub fn sqrt(self) -> UnitVal<T1, <Unit1 as Div<P2>>::Output>
    where
        T1: Real,
        Z0: IsEqual<Mod<Unit1, P2>, Output = True>,
        Unit1: Rem<P2> + Div<P2>,
    {
        UnitVal::new(self.val.sqrt())
    }
}

// TODO: Replace with proc macro for proper `IsEqual` impl
#[macro_export]
macro_rules! create_unit_system {
    ($vis:vis struct $ident:ident<$($unit:ident),+ $(,)?>) => {
        #[derive(
            Debug,
            Copy,
            Clone,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            $crate::bytemuck::Pod,
            $crate::bytemuck::Zeroable,
            $crate::serde::Serialize,
            $crate::serde::Deserialize,
            $crate::align1::Align1,
        )]
        #[repr(transparent)]
        $vis struct $ident<$($unit,)*>(::std::marker::PhantomData<($($unit,)*)>)
        where
            $($unit: $crate::typenum::Integer,)*;

        $crate::paste::paste!{
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
                fn add(
                    self,
                    _rhs: $ident<$([<$unit 2>],)*>,
                ) -> Self::Output {
                    ::std::panic!("Not implemented")
                }
            }
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
                fn sub(
                    self,
                    _rhs: $ident<$([<$unit 2>],)*>,
                ) -> Self::Output {
                    ::std::panic!("Not implemented")
                }
            }
            impl<$($unit,)* Value> ::std::ops::Mul<Value> for $ident<$($unit,)*>
            where
                $(
                    $unit: $crate::typenum::Integer + ::std::ops::Mul<Value>,
                    $unit::Output: $crate::typenum::Integer,
                )*
            {
                type Output = $ident<$($unit::Output,)*>;
                fn mul(self, _rhs: Value) -> Self::Output {
                    ::std::panic!("Not implemented")
                }
            }
            impl<$($unit,)* Value> ::std::ops::Div<Value> for $ident<$($unit,)*>
            where
                $(
                    $unit: $crate::typenum::Integer + ::std::ops::Div<Value>,
                    $unit::Output: $crate::typenum::Integer,
                )*
            {
                type Output = $ident<$($unit::Output,)*>;
                fn div(self, _rhs: Value) -> Self::Output {
                    ::std::panic!("Not implemented")
                }
            }
            impl<$($unit,)* Value> ::std::ops::Rem<Value> for $ident<$($unit,)*>
            where
                $(
                    $unit: $crate::typenum::Integer + ::std::ops::Rem<Value>,
                    $unit::Output: $crate::typenum::Integer,
                )*
            {
                type Output = $ident<$($unit::Output,)*>;
                fn rem(self, _rhs: Value) -> Self::Output {
                    ::std::panic!("Not implemented")
                }
            }
            impl<$([<$unit 1>], [<$unit 2>],)*> $crate::typenum::IsEqual<$ident<$([<$unit 2>],)*>>
                for $ident<$([<$unit 1>],)*>
            where
                $(
                    [<$unit 1>]: $crate::typenum::Integer + $crate::typenum::IsEqual<[<$unit 2>], Output=$crate::typenum::True>,
                    [<$unit 2>]: $crate::typenum::Integer,
                )*

            {
                type Output = $crate::typenum::True;

                fn is_equal(self, _rhs: $ident<$([<$unit 2>],)*>) -> Self::Output {
                    panic!("Not implemented")
                }
            }
        }
    };
}

#[cfg(test)]
mod test {
    use crate::unit_val::UnitVal;
    use fixed::types::I53F11;
    use typenum::{Diff, Sum, P1, Z0};
    create_unit_system!(struct CreatedUnitSystem<Seconds, Meters, Kilograms>);

    type Unitless = CreatedUnitSystem<Z0, Z0, Z0>;
    type Seconds = CreatedUnitSystem<P1, Z0, Z0>;
    type Meters = CreatedUnitSystem<Z0, P1, Z0>;
    type Kilograms = CreatedUnitSystem<Z0, Z0, P1>;

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
