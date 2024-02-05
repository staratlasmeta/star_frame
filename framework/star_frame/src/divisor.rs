use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{Pod, Zeroable};
use num_traits::{FromPrimitive, ToPrimitive};
use star_frame_proc::Align1;
use std::io::Read;
use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

#[derive(
    Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Pod, Zeroable, Align1,
)]
#[repr(transparent)]
pub struct Divisor<T, const DIV: u64>(T);
impl<T, const DIV: u64> Divisor<T, DIV> {
    pub fn new(val: f64) -> Self
    where
        T: FromPrimitive,
    {
        Self(T::from_f64(val * DIV as f64).unwrap())
    }

    pub fn to_f64(&self) -> f64
    where
        T: ToPrimitive,
    {
        f64::from_f64(self.0.to_f64().unwrap() / DIV as f64).unwrap()
    }

    pub fn from_raw(t: T) -> Self {
        Self(t)
    }

    pub fn into_raw(self) -> T {
        self.0
    }
}
impl<T1, T2, const DIV: u64> Add<Divisor<T2, DIV>> for Divisor<T1, DIV>
where
    T1: Add<T2>,
{
    type Output = Divisor<<T1 as Add<T2>>::Output, DIV>;

    fn add(self, rhs: Divisor<T2, DIV>) -> Self::Output {
        Divisor(self.0 + rhs.0)
    }
}
impl<T1, T2, const DIV: u64> AddAssign<Divisor<T2, DIV>> for Divisor<T1, DIV>
where
    T1: AddAssign<T2>,
{
    fn add_assign(&mut self, rhs: Divisor<T2, DIV>) {
        self.0 += rhs.0;
    }
}
impl<T1, T2, const DIV: u64> Sub<Divisor<T2, DIV>> for Divisor<T1, DIV>
where
    T1: Sub<T2>,
{
    type Output = Divisor<<T1 as Sub<T2>>::Output, DIV>;

    fn sub(self, rhs: Divisor<T2, DIV>) -> Self::Output {
        Divisor(self.0 - rhs.0)
    }
}
impl<T1, T2, const DIV: u64> SubAssign<Divisor<T2, DIV>> for Divisor<T1, DIV>
where
    T1: SubAssign<T2>,
{
    fn sub_assign(&mut self, rhs: Divisor<T2, DIV>) {
        self.0 -= rhs.0;
    }
}
impl<T1, T2, const DIV: u64> Mul<T2> for Divisor<T1, DIV>
where
    T1: Mul<T2>,
{
    type Output = Divisor<<T1 as Mul<T2>>::Output, DIV>;

    fn mul(self, rhs: T2) -> Self::Output {
        Divisor(self.0 * rhs)
    }
}
impl<T1, T2, const DIV: u64> MulAssign<T2> for Divisor<T1, DIV>
where
    T1: MulAssign<T2>,
{
    fn mul_assign(&mut self, rhs: T2) {
        self.0 *= rhs;
    }
}
impl<T, const DIV: u64> BorshSerialize for Divisor<T, DIV>
where
    T: BorshSerialize,
{
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        self.0.serialize(writer)
    }
}
impl<T, const DIV: u64> BorshDeserialize for Divisor<T, DIV>
where
    T: BorshDeserialize,
{
    fn deserialize_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        Ok(Self(T::deserialize_reader(reader)?))
    }
}
