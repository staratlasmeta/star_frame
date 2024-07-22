use solana_program::pubkey::Pubkey;

#[macro_export]
macro_rules! star_frame_serialize_borsh {
    (@impl $ty:ty) => {
        impl $crate::serialize::StarFrameSerialize for $ty {
            fn to_bytes(&self, output: &mut &mut [u8]) -> $crate::Result<()> {
                <$ty as $crate::borsh::BorshSerialize>::serialize(self, output).map_err(Into::into)
            }
        }
        unsafe impl<'a> $crate::serialize::StarFrameFromBytes<'a> for $ty {
            fn from_bytes(bytes: &mut &'a [u8]) -> $crate::Result<Self> {
                <$ty as $crate::borsh::BorshDeserialize>::deserialize(bytes).map_err(Into::into)
            }
        }
    };
    ($($ty:ty),* $(,)?) => {
        $($crate::star_frame_serialize_borsh!(@impl $ty);)*
    };
}

star_frame_serialize_borsh!(
    (),
    u8,
    u16,
    u32,
    u64,
    u128,
    i8,
    i16,
    i32,
    i64,
    i128,
    bool,
    Pubkey,
    String
);
