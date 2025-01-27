use bytemuck::{Pod, Zeroable};
use num_traits::{AsPrimitive, Num};
use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};

#[derive(Copy, Clone)]
pub struct PodVec<T, const CAPACITY: usize, Size = usize>
where
    T: Copy,
    Size: Num + AsPrimitive<usize>,
{
    size: Size,
    data: [MaybeUninit<T>; CAPACITY],
}
impl<T, const CAPACITY: usize, Size> PodVec<T, CAPACITY, Size>
where
    T: Copy,
    Size: Num + AsPrimitive<usize>,
{
    pub fn new() -> Self {
        Self {
            size: Size::zero(),
            data: unsafe { MaybeUninit::uninit().assume_init() },
        }
    }

    pub fn push(&mut self, value: T) {
        assert!(self.size.as_() < CAPACITY);
        self.data[self.size.as_()] = MaybeUninit::new(value);
        self.size = self.size + Size::one();
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index < self.size.as_() {
            let value = unsafe { self.data[index].assume_init() };
            for i in index..self.size.as_() - 1 {
                self.data[i] = self.data[i + 1];
            }
            self.size = self.size - Size::one();
            Some(value)
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.size = Size::zero();
    }
}
unsafe impl<T, const CAPACITY: usize, Size> Zeroable for PodVec<T, CAPACITY, Size>
where
    T: Copy,
    Size: Zeroable + Num + AsPrimitive<usize>,
{
}
unsafe impl<T, const CAPACITY: usize, Size> Pod for PodVec<T, CAPACITY, Size>
where
    T: Pod,
    Size: Pod + Num + AsPrimitive<usize>,
{
}
impl<T, const CAPACITY: usize, Size> Deref for PodVec<T, CAPACITY, Size>
where
    T: Copy,
    Size: Num + AsPrimitive<usize>,
{
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { std::slice::from_raw_parts(self.data.as_ptr() as *const T, self.size.as_()) }
    }
}
impl<T, const CAPACITY: usize, Size> DerefMut for PodVec<T, CAPACITY, Size>
where
    T: Copy,
    Size: Num + AsPrimitive<usize>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::slice::from_raw_parts_mut(self.data.as_mut_ptr() as *mut T, self.size.as_()) }
    }
}
impl<T, const CAPACITY: usize, Size> AsRef<[T]> for PodVec<T, CAPACITY, Size>
where
    T: Copy,
    Size: Num + AsPrimitive<usize>,
{
    fn as_ref(&self) -> &[T] {
        self.deref()
    }
}
impl<T, const CAPACITY: usize, Size> AsMut<[T]> for PodVec<T, CAPACITY, Size>
where
    T: Copy,
    Size: Num + AsPrimitive<usize>,
{
    fn as_mut(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.data.as_mut_ptr() as *mut T, self.size.as_()) }
    }
}

impl<T, const CAPACITY: usize, Size> Default for PodVec<T, CAPACITY, Size>
where
    T: Copy,
    Size: Num + AsPrimitive<usize>,
{
    fn default() -> Self {
        Self::new()
    }
}
