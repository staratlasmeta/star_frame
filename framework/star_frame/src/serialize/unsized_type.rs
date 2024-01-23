use crate::align1::Align1;
use crate::serialize::pointer_breakup::{BuildPointer, BuildPointerMut};
use crate::serialize::{FrameworkFromBytes, FrameworkFromBytesMut};
use bytemuck::Pod;
use std::ops::{Deref, DerefMut};

pub trait UnsizedType: 'static {
    type RefMeta: 'static + Copy;
    type Ref<'a>: FrameworkFromBytes<'a>
        + Deref<Target = Self>
        + BuildPointer<Metadata = Self::RefMeta>
        + Copy;
    type RefMut<'a>: FrameworkFromBytesMut<'a>
        + DerefMut<Target = Self>
        + BuildPointerMut<'a, Metadata = Self::RefMeta>;
}
impl<T> UnsizedType for T
where
    T: Align1 + Pod,
{
    type RefMeta = ();
    type Ref<'a> = &'a T;
    type RefMut<'a> = &'a mut T;
}
