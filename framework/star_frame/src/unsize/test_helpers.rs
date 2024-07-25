use crate::prelude::*;
use crate::unsize::ref_wrapper::RefWrapper;
use solana_program::program_memory::sol_memset;
use std::marker::PhantomData;

/// A way to test [`UnsizedType`] types. Uses a [`Vec<u8>`] internally.
#[derive(Debug)]
pub struct TestByteSet<T: ?Sized> {
    /// The data bytes.
    pub bytes: Vec<u8>,
    phantom_t: PhantomData<T>,
}

impl<T: ?Sized> TestByteSet<T> {
    /// Creates a new [`TestByteSet`] from a given set of bytes. These are not validated.
    #[must_use]
    pub const fn from_bytes(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            phantom_t: PhantomData,
        }
    }

    /// Creates a new [`TestByteSet`] by initializing the type with an arg from [`UnsizedInit`].
    pub fn new<A>(arg: A) -> Result<Self>
    where
        T: UnsizedInit<A>,
    {
        let mut bytes = vec![0; T::INIT_BYTES];
        unsafe {
            T::init(&mut bytes, arg)?;
        }
        Ok(Self::from_bytes(bytes))
    }
}

impl<T> TestByteSet<T>
where
    T: ?Sized + UnsizedType,
{
    /// Resets the test byte set by setting length to [`UnsizedInit::INIT_BYTES`], zeroing the
    /// bytes, then calling [`UnsizedInit::init`] with `arg`.
    pub fn re_init<A>(&mut self, arg: A) -> Result<RefWrapper<&mut Vec<u8>, T::RefData>>
    where
        T: UnsizedInit<A>,
    {
        self.bytes.resize(T::INIT_BYTES, 0);
        sol_memset(&mut self.bytes, 0, T::INIT_BYTES);
        unsafe { T::init(&mut self.bytes, arg).map(|r| r.0) }
    }

    /// Gets an immutable [`RefWrapper`].
    pub fn immut(&self) -> Result<RefWrapper<&Vec<u8>, T::RefData>> {
        T::from_bytes(&self.bytes).map(|r| r.ref_wrapper)
    }

    /// Gets a mutable [`RefWrapper`].
    pub fn mutable(&mut self) -> Result<RefWrapper<&mut Vec<u8>, T::RefData>> {
        T::from_bytes(&mut self.bytes).map(|r| r.ref_wrapper)
    }
}
