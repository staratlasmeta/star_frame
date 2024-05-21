use crate::prelude::*;
use crate::serialize::ref_wrapper::RefWrapper;
use solana_program::program_memory::sol_memset;
use star_frame::prelude::UnsizedInit;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct TestByteSet<T: ?Sized> {
    pub bytes: Vec<u8>,
    pub phantom_t: PhantomData<T>,
}

impl<T: ?Sized> TestByteSet<T> {
    pub fn new<A>(arg: A) -> Result<Self>
    where
        T: UnsizedInit<A>,
    {
        let mut bytes = vec![0; T::INIT_BYTES];
        unsafe {
            T::init(&mut bytes, arg)?;
        }
        Ok(Self {
            bytes,
            phantom_t: PhantomData,
        })
    }
}

impl<T> TestByteSet<T>
where
    T: ?Sized + UnsizedType,
{
    pub fn re_init<A>(&mut self, arg: A) -> Result<RefWrapper<&mut Vec<u8>, T::RefData>>
    where
        T: UnsizedInit<A>,
    {
        self.bytes.resize(T::INIT_BYTES, 0);
        sol_memset(&mut self.bytes, 0, T::INIT_BYTES);
        unsafe { T::init(&mut self.bytes, arg).map(|r| r.0) }
    }

    pub fn immut(&self) -> Result<RefWrapper<&Vec<u8>, T::RefData>> {
        unsafe { T::from_bytes(&self.bytes).map(|r| r.ref_wrapper) }
    }

    pub fn mutable(&mut self) -> Result<RefWrapper<&mut Vec<u8>, T::RefData>> {
        unsafe { T::from_bytes(&mut self.bytes).map(|r| r.ref_wrapper) }
    }
}
