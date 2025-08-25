use super::UnsizedType;
use crate::Result;
use bytemuck::Zeroable;

/// An [`UnsizedType`] that can be initialized with a statically sized `InitArg`.
pub trait UnsizedInit<InitArg>: UnsizedType {
    /// Amount of bytes this type takes to initialize. The bytes are initialized, but may not be zeroed.
    const INIT_BYTES: usize;

    /// Initializes the [`UnsizedType`] from the `InitArg`.
    fn init(bytes: &mut &mut [u8], arg: InitArg) -> Result<()>;
}

/// Allows implementing `UnsizedInit<DefaultInit>` for [`bytemuck`] types.
///
/// We have a blanket implementation for types that implement [`Zeroable`], so this can only be implemented
/// for types that don't.
pub trait DefaultInitable {
    /// Returns a "default initialized" value of the type.
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

/// Argument for initializing a type to a default value.
#[derive(Debug, Copy, Clone)]
pub struct DefaultInit;
