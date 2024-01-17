use crate::align1::Align1;
use crate::serialize::pointer_breakup::{BuildPointer, BuildPointerMut, PointerBreakup};
use crate::serialize::{FrameworkFromBytes, FrameworkFromBytesMut};
use bytemuck::Pod;
use std::ops::{Deref, DerefMut};

pub trait SerializeWith {
    type RefMeta: 'static + Copy;
    type Ref<'a>: FrameworkFromBytes<'a>
        + Deref<Target = Self>
        + BuildPointer<Metadata = Self::RefMeta>
    where
        Self: 'a;
    type RefMut<'a>: FrameworkFromBytesMut<'a>
        + DerefMut<Target = Self>
        + BuildPointerMut<'a, Metadata = Self::RefMeta>
    where
        Self: 'a;
}
impl<T> SerializeWith for T
where
    T: Align1 + Pod,
{
    type RefMeta = <Self::Ref<'static> as PointerBreakup>::Metadata;
    type Ref<'a> = &'a T;
    type RefMut<'a> = &'a mut T;
}
