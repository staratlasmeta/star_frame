#[cfg(test)]
#[allow(dead_code)]
mod test {
    use crate::prelude::*;
    use crate::serialize::pointer_breakup::{BuildPointer, BuildPointerMut};
    use crate::serialize::unsized_type::UnsizedTypeToOwned;
    use crate::serialize::ResizeFn;
    use star_frame::serialize::pointer_breakup::PointerBreakup;
    use std::ptr::NonNull;

    #[derive(Align1)]
    #[repr(transparent)]
    struct Data1<T: ?Sized + UnsizedType> {
        data: T,
    }

    #[repr(transparent)]
    struct Data1Ref<'a, T: ?Sized + UnsizedType>(T::Ref<'a>);
    impl<'a, T: ?Sized + UnsizedType> Data1Ref<'a, T> {
        pub fn data(self) -> T::Ref<'a> {
            self.0
        }
    }
    impl<'a, T: ?Sized + UnsizedType> Clone for Data1Ref<'a, T> {
        fn clone(&self) -> Self {
            *self
        }
    }
    impl<'a, T: ?Sized + UnsizedType> Copy for Data1Ref<'a, T> {}
    impl<'a, T: ?Sized + UnsizedType> FrameworkSerialize for Data1Ref<'a, T> {
        fn to_bytes(&self, output: &mut &mut [u8]) -> Result<()> {
            <<T as UnsizedType>::Ref<'a> as FrameworkSerialize>::to_bytes(&self.0, output)
        }
    }
    unsafe impl<'a, T: ?Sized + UnsizedType> FrameworkFromBytes<'a> for Data1Ref<'a, T> {
        fn from_bytes(bytes: &mut &'a [u8]) -> Result<Self> {
            Ok(Self(<<T as UnsizedType>::Ref<'a> as FrameworkFromBytes<
                'a,
            >>::from_bytes(bytes)?))
        }
    }
    impl<'a, T: ?Sized + UnsizedType> PointerBreakup for Data1Ref<'a, T> {
        type Metadata = T::RefMeta;

        fn break_pointer(&self) -> (NonNull<()>, Self::Metadata) {
            self.0.break_pointer()
        }
    }
    impl<'a, T: ?Sized + UnsizedType> BuildPointer for Data1Ref<'a, T> {
        unsafe fn build_pointer(pointee: NonNull<()>, metadata: Self::Metadata) -> Self {
            unsafe { Self(T::Ref::build_pointer(pointee, metadata)) }
        }
    }

    #[repr(transparent)]
    struct Data1RefMut<'a, T: ?Sized + UnsizedType>(T::RefMut<'a>);
    impl<'a, T: ?Sized + UnsizedType> Data1RefMut<'a, T> {
        pub fn data(&self) -> &T::RefMut<'a> {
            &self.0
        }

        pub fn data_mut(&mut self) -> &mut T::RefMut<'a> {
            &mut self.0
        }
    }
    impl<'a, T: ?Sized + UnsizedType> FrameworkSerialize for Data1RefMut<'a, T> {
        fn to_bytes(&self, output: &mut &mut [u8]) -> Result<()> {
            <<T as UnsizedType>::RefMut<'a> as FrameworkSerialize>::to_bytes(&self.0, output)
        }
    }
    unsafe impl<'a, T: ?Sized + UnsizedType> FrameworkFromBytesMut<'a> for Data1RefMut<'a, T> {
        fn from_bytes_mut(
            bytes: &mut &'a mut [u8],
            resize: impl ResizeFn<'a, Self::Metadata>,
        ) -> Result<Self> {
            Ok(Self(
                <<T as UnsizedType>::RefMut<'a> as FrameworkFromBytesMut<'a>>::from_bytes_mut(
                    bytes, resize,
                )?,
            ))
        }
    }
    impl<'a, T: ?Sized + UnsizedType> PointerBreakup for Data1RefMut<'a, T> {
        type Metadata = T::RefMeta;

        fn break_pointer(&self) -> (NonNull<()>, Self::Metadata) {
            self.0.break_pointer()
        }
    }
    impl<'a, T: ?Sized + UnsizedType> BuildPointerMut<'a> for Data1RefMut<'a, T> {
        unsafe fn build_pointer_mut(
            pointee: NonNull<()>,
            metadata: Self::Metadata,
            resize: impl ResizeFn<'a, Self::Metadata>,
        ) -> Self {
            unsafe {
                Self(
                    <<T as UnsizedType>::RefMut<'a> as BuildPointerMut<'a>>::build_pointer_mut(
                        pointee, metadata, resize,
                    ),
                )
            }
        }
    }

    impl<T: ?Sized + UnsizedType> UnsizedType for Data1<T> {
        type RefMeta = T::RefMeta;
        type Ref<'a> = Data1Ref<'a, T>;
        type RefMut<'a> = Data1RefMut<'a, T>;
    }

    struct Data1Owned<T: ?Sized + UnsizedTypeToOwned> {
        data: T::Owned,
    }
    impl<T: ?Sized + UnsizedType> UnsizedTypeToOwned for Data1<T>
    where
        T: UnsizedTypeToOwned,
    {
        type Owned = Data1Owned<T>;
        fn owned_from_ref(r: Self::Ref<'_>) -> Self::Owned {
            Data1Owned {
                data: T::owned_from_ref(r.0),
            }
        }
        fn owned_from_ref_mut(r: Self::RefMut<'_>) -> Self::Owned {
            Data1Owned {
                data: T::owned_from_ref_mut(r.0),
            }
        }
    }

    #[derive(Align1)]
    struct Data2(pub(crate) u8, pub(super) List<PackedValue<i32>>);

    // struct Data3 {
    //     list1: List<PackedValue<u16>>,
    //     pub list2: List<PackedValue<u32>>,
    //     pub(crate) list3: List<PackedValue<u64>>,
    //     list4: List<PackedValue<u128>>,
    // }

    type Data3Storage = CombinedUnsized<
        CombinedUnsized<List<PackedValue<u16>>, List<PackedValue<u32>>>,
        CombinedUnsized<List<PackedValue<u64>>, List<PackedValue<u128>>>,
    >;
    #[repr(transparent)]
    struct Data3(Data3Storage);

    #[repr(transparent)]
    struct Data3Ref<'a>(<Data3Storage as UnsizedType>::Ref<'a>);
    impl<'a> Data3Ref<'a> {
        fn list1(&self) -> <List<PackedValue<u16>> as UnsizedType>::Ref<'_> {
            self.0.t().t()
        }
        fn list2(&self) -> <List<PackedValue<u32>> as UnsizedType>::Ref<'_> {
            self.0.t().u()
        }
        fn list3(&self) -> <List<PackedValue<u64>> as UnsizedType>::Ref<'_> {
            self.0.u().t()
        }
        fn list4(&self) -> <List<PackedValue<u128>> as UnsizedType>::Ref<'_> {
            self.0.u().u()
        }
    }

    #[repr(transparent)]
    struct Data3RefMut<'a>(<Data3Storage as UnsizedType>::RefMut<'a>);
    impl<'a> Data3RefMut<'a> {
        fn list1(&self) -> <List<PackedValue<u16>> as UnsizedType>::Ref<'_> {
            self.0.t().t()
        }
        fn list2(&self) -> <List<PackedValue<u32>> as UnsizedType>::Ref<'_> {
            self.0.t().u()
        }
        fn list3(&self) -> <List<PackedValue<u64>> as UnsizedType>::Ref<'_> {
            self.0.u().t()
        }
        fn list4(&self) -> <List<PackedValue<u128>> as UnsizedType>::Ref<'_> {
            self.0.u().u()
        }

        fn list1_mut(&mut self) -> <List<PackedValue<u16>> as UnsizedType>::RefMut<'a> {
            self.0.t_mut().t_mut()
        }
        fn list2_mut(&mut self) -> <List<PackedValue<u32>> as UnsizedType>::RefMut<'a> {
            self.0.t_mut().u_mut()
        }
        fn list3_mut(&mut self) -> <List<PackedValue<u64>> as UnsizedType>::RefMut<'a> {
            self.0.u_mut().t_mut()
        }
        fn list4_mut(&mut self) -> <List<PackedValue<u128>> as UnsizedType>::RefMut<'a> {
            self.0.u_mut().u_mut()
        }
    }
}
