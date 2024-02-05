use borsh;
use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{Pod, Zeroable};

/// Wrapper around booleans for u8 types
#[repr(transparent)]
#[derive(Copy, Clone, Pod, Zeroable, Debug, BorshDeserialize, BorshSerialize)]
pub struct PodBool(u8);

impl PodBool {
    /// Constructor function
    #[must_use]
    pub fn new(val: bool) -> Self {
        Self(u8::from(val))
    }

    /// Getter function
    #[must_use]
    pub fn get(&self) -> bool {
        self.0 > 0
    }

    /// Setter function
    pub fn set(&mut self, val: bool) {
        self.0 = u8::from(val);
    }
}

impl PartialEq for PodBool {
    fn eq(&self, other: &Self) -> bool {
        self.get().eq(&other.get())
    }
}
impl Eq for PodBool {}

impl From<PodBool> for u8 {
    fn from(val: PodBool) -> Self {
        val.0
    }
}

/// Trait for sealing Boolable trait implementations for types other than u8
pub trait Boolable: sealed::Sealed {}
impl Boolable for u8 {}

mod sealed {
    pub trait Sealed {}
    impl Sealed for u8 {}
}

impl From<bool> for PodBool {
    fn from(val: bool) -> Self {
        Self::new(val)
    }
}

impl From<PodBool> for bool {
    fn from(val: PodBool) -> Self {
        val.get()
    }
}
