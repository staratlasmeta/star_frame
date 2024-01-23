use crate::serialize::unsized_type::UnsizedType;
use crate::serialize::{FrameworkFromBytes, FrameworkFromBytesMut, FrameworkInit};
use crate::Result;
use solana_program::program_memory::sol_memset;
use std::marker::PhantomData;
use std::ptr::NonNull;

#[derive(Debug)]
pub struct TestByteSet<T: ?Sized> {
    pub bytes: Vec<u8>,
    pub phantom_t: PhantomData<T>,
}

impl<T: ?Sized> TestByteSet<T> {
    pub fn new(len: usize) -> Self {
        Self {
            bytes: vec![0; len],
            phantom_t: PhantomData,
        }
    }
}

impl<T> TestByteSet<T>
where
    T: ?Sized + UnsizedType,
{
    pub fn init<'a, A>(&'a mut self, arg: A) -> Result<T::RefMut<'a>>
    where
        T::RefMut<'a>: FrameworkInit<'a, A>,
    {
        let vec_ptr = &mut self.bytes as *mut Vec<u8>;
        let bytes = &mut self.bytes[..<T::RefMut<'a> as FrameworkInit<'a, A>>::INIT_LENGTH];
        sol_memset(bytes, 0, bytes.len());
        unsafe {
            <T::RefMut<'a> as FrameworkInit<'a, A>>::init(bytes, arg, move |len, _| {
                let bytes = &mut *vec_ptr;
                bytes.resize(len, 0);
                Ok(NonNull::<[u8]>::from(bytes.as_slice()).cast())
            })
        }
    }

    pub fn immut(&self) -> Result<T::Ref<'_>> {
        T::Ref::from_bytes(&mut &self.bytes[..])
    }

    pub fn mutable(&mut self) -> Result<T::RefMut<'_>> {
        let bytes = &mut self.bytes as *mut Vec<u8>;
        T::RefMut::from_bytes_mut(&mut &mut self.bytes[..], move |len, _| {
            let bytes = unsafe { &mut *bytes };
            bytes.resize(len, 0);
            Ok(NonNull::<[u8]>::from(bytes.as_slice()).cast())
        })
    }
}
