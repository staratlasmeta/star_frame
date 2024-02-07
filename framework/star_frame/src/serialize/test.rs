use crate::serialize::unsized_type::UnsizedType;
use crate::serialize::{FrameworkFromBytes, FrameworkFromBytesMut, FrameworkInit};
use crate::Result;
use solana_program::program_memory::sol_memset;
use std::marker::PhantomData;
use std::ptr::{addr_of_mut, NonNull};

#[derive(Debug)]
pub struct TestByteSet<T: ?Sized> {
    pub bytes: Vec<u8>,
    pub phantom_t: PhantomData<T>,
}

impl<T: ?Sized> TestByteSet<T> {
    pub fn new<A>(arg: A) -> Result<Self>
    where
        T: FrameworkInit<A>,
    {
        let mut bytes = vec![0; T::INIT_LENGTH];
        unsafe {
            T::init(&mut bytes, arg, |_, _| {
                panic!("Cannot resize during `init`")
            })?;
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
    pub fn re_init<A>(&mut self, arg: A) -> Result<T::RefMut<'_>>
    where
        T: FrameworkInit<A>,
    {
        self.bytes.resize(T::INIT_LENGTH, 0);
        sol_memset(&mut self.bytes, 0, T::INIT_LENGTH);
        unsafe {
            T::init(&mut self.bytes, arg, |_, _| {
                panic!("Cannot resize during `init`")
            })
        }
    }

    pub fn immut(&self) -> Result<T::Ref<'_>> {
        T::Ref::from_bytes(&mut &self.bytes[..])
    }

    pub fn mutable(&mut self) -> Result<T::RefMut<'_>> {
        let bytes = addr_of_mut!(self.bytes);
        T::RefMut::from_bytes_mut(&mut &mut self.bytes[..], move |len, _| {
            let bytes = unsafe { &mut *bytes };
            bytes.resize(len, 0);
            Ok(NonNull::<[u8]>::from(bytes.as_slice()).cast())
        })
    }
}
