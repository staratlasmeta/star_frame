use crate::serialize::ResizeFn;
use std::ptr;
use std::ptr::Pointee;

pub trait PointerBreakup {
    type Metadata: 'static + Copy;

    fn break_pointer(&self) -> (*const (), Self::Metadata);
}
pub trait PointerBreakupMut: PointerBreakup {
    fn break_pointer_mut(&mut self) -> (*mut (), Self::Metadata);
}

pub trait BuildPointer: PointerBreakup {
    /// # Safety
    /// Pointer must come from [`break_pointer`](PointerBreakup::break_pointer)
    unsafe fn build_pointer(pointee: *const (), metadata: Self::Metadata) -> Self;
}
pub trait BuildPointerMut<'a>: PointerBreakupMut {
    /// # Safety
    /// Pointer must come from [`break_pointer_mut`](PointerBreakupMut::break_pointer_mut).
    /// `resize` may only be called when self is not aliased.
    unsafe fn build_pointer_mut(
        pointee: *mut (),
        metadata: Self::Metadata,
        resize: impl ResizeFn<'a, Self::Metadata>,
    ) -> Self;
}

impl<'a, T> PointerBreakup for &'a T {
    type Metadata = <T as Pointee>::Metadata;

    fn break_pointer(&self) -> (*const (), Self::Metadata) {
        ((*self as *const T).cast(), ptr::metadata(self))
    }
}
impl<'a, T> BuildPointer for &'a T {
    unsafe fn build_pointer(pointee: *const (), metadata: Self::Metadata) -> Self {
        &*ptr::from_raw_parts(pointee, metadata)
    }
}

impl<'a, T> PointerBreakup for &'a mut T {
    type Metadata = <T as Pointee>::Metadata;

    fn break_pointer(&self) -> (*const (), Self::Metadata) {
        ((*self as *const T).cast(), ptr::metadata(self))
    }
}
impl<'a, T> PointerBreakupMut for &'a mut T {
    fn break_pointer_mut(&mut self) -> (*mut (), Self::Metadata) {
        ((*self as *mut T).cast(), ptr::metadata(self))
    }
}
impl<'a, T> BuildPointerMut<'a> for &'a mut T {
    unsafe fn build_pointer_mut(
        pointee: *mut (),
        metadata: Self::Metadata,
        _resize: impl ResizeFn<'a, Self::Metadata>,
    ) -> Self {
        &mut *ptr::from_raw_parts_mut(pointee, metadata)
    }
}
