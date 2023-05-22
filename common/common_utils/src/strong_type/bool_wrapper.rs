use bytemuck::{Pod, Zeroable};
#[repr(transparent)]
#[derive(Copy, Clone, Pod, Zeroable, Debug)]
/// Wrapper around booleans for u8 types
pub struct BoolWrapper(u8);

impl BoolWrapper {
    #[must_use]
    /// Constructor function
    pub fn new(val: bool) -> Self {
        Self(u8::from(val))
    }

    #[must_use]
    /// Getter function
    pub fn get(&self) -> bool {
        self.0 > 0
    }

    /// Setter function
    pub fn set(&mut self, val: bool) {
        self.0 = u8::from(val);
    }
}

impl PartialEq for BoolWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.get().eq(&other.get())
    }
}
impl Eq for BoolWrapper {}

impl From<BoolWrapper> for u8 {
    fn from(val: BoolWrapper) -> Self {
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

impl From<bool> for BoolWrapper {
    fn from(val: bool) -> Self {
        Self::new(val)
    }
}

impl From<BoolWrapper> for bool {
    fn from(val: BoolWrapper) -> Self {
        val.get()
    }
}
