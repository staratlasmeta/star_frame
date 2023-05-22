use common_utils::prelude::*;

/// An account's data size.
pub trait DataSize {
    /// The minimum account data size.
    const MIN_DATA_SIZE: usize;
}

macro_rules! impl_data_size {
    ($ty:ty, $size:expr) => {
        impl DataSize for $ty {
            const MIN_DATA_SIZE: usize = $size;
        }
    };
}
impl_data_size!((), 0);
impl_data_size!(u8, 1);
impl_data_size!(u16, 2);
impl_data_size!(u32, 4);
impl_data_size!(u64, 8);
impl_data_size!(u128, 16);
impl_data_size!(i8, 1);
impl_data_size!(i16, 2);
impl_data_size!(i32, 4);
impl_data_size!(i64, 8);
impl_data_size!(i128, 16);
impl_data_size!(bool, 1);
impl_data_size!(Pubkey, 32);

impl<T, const N: usize> DataSize for [T; N]
where
    T: DataSize,
{
    const MIN_DATA_SIZE: usize = N * T::MIN_DATA_SIZE;
}
