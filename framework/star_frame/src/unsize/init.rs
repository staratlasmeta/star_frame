use super::UnsizedType;
use crate::Result;
use bytemuck::Zeroable;

/// An [`UnsizedType`] that can be initialized with an `InitArg`. Must have a statically known size
/// (for arg type) at initialization.
pub trait UnsizedInit<InitArg>: UnsizedType {
    /// Amount of zeroed bytes this type takes to initialize.
    const INIT_BYTES: usize;

    /// # Safety
    /// todo... The trait should probably be marked as unsafe, but the method is probably fine
    unsafe fn init(bytes: &mut &mut [u8], arg: InitArg) -> Result<()>;
}

pub trait DefaultInitable {
    fn default_init() -> Self;
}

impl<T> DefaultInitable for T
where
    T: Zeroable,
{
    fn default_init() -> Self {
        T::zeroed()
    }
}

/// Argument for initializing a type to a default value
#[derive(Debug, Copy, Clone)]
pub struct DefaultInit;
