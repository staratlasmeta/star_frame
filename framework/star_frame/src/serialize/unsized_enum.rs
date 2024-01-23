pub use star_frame_proc::unsized_enum;

use crate::align1::Align1;
use crate::packed_value::{PackedValue, PackedValueChecked};
use crate::serialize::combined_unsized::CombinedUnsized;
use crate::serialize::list::List;
use crate::serialize::pointer_breakup::{BuildPointer, BuildPointerMut, PointerBreakup};
use crate::serialize::unsized_type::UnsizedType;
use crate::serialize::{
    FrameworkFromBytes, FrameworkFromBytesMut, FrameworkInit, FrameworkSerialize,
};
use crate::Result;
use advance::Advance;
use bytemuck::{CheckedBitPattern, Pod, Zeroable};
use derivative::Derivative;
use solana_program::program_error::ProgramError;
use solana_program::program_memory::sol_memset;
use star_frame::serialize::ResizeFn;
use std::any::type_name_of_val;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::{Deref, DerefMut};
use std::ptr;
use std::ptr::{slice_from_raw_parts_mut, NonNull};

pub trait UnsizedEnum:
    for<'a> UnsizedType<Ref<'a> = Self::EnumRefWrapper<'a>, RefMut<'a> = Self::EnumRefMutWrapper<'a>>
{
    type Discriminant: 'static + Copy + Debug + CheckedBitPattern;
    type EnumRefWrapper<'a>: EnumRefWrapper;
    type EnumRefMutWrapper<'a>: EnumRefMutWrapper;

    fn discriminant(&self) -> Self::Discriminant;
}

pub trait EnumRefWrapper {
    type Ref<'a>
    where
        Self: 'a;

    fn value(&self) -> Self::Ref<'_>;
}
pub trait EnumRefMutWrapper: EnumRefWrapper {
    type RefMut<'a>
    where
        Self: 'a;

    fn value_mut(&mut self) -> Self::RefMut<'_>;
}

// ---------------------------- Test Stuff ----------------------------

#[cfg(test)]
mod test {
    use super::*;
    use crate::serialize::test::TestByteSet;

    #[derive(Pod, Zeroable, Copy, Clone, Align1, Debug, PartialEq, Eq)]
    #[repr(C, packed)]
    pub struct TestStruct {
        val1: u32,
        val2: u64,
    }

    #[unsized_enum]
    enum TestEnum<T>
    where
        T: UnsizedType,
    {
        #[variant_type(T)]
        A,
        #[variant_type(CombinedUnsized<TestStruct, List<u8, u8>>)]
        B = 4,
        #[variant_type(List<PackedValue<u32>>)]
        C,
    }

    // #[unsized_enum]
    // enum TestEnum2 {
    //     #[variant_type(u8)]
    //     A,
    //     #[variant_type(CombinedUnsized<TestStruct, List<u8, u8>>)]
    //     B = 4,
    //     #[variant_type(List<PackedValue<u32>>)]
    //     C,
    // }

    #[test]
    fn test_enum() -> Result<()> {
        type EnumToTest = TestEnum<TestStruct>;
        let mut test_byte_set = TestByteSet::<EnumToTest>::new(
            <<EnumToTest as UnsizedType>::RefMut<'_> as FrameworkInit<(
                test_enum::A,
                (TestStruct,),
            )>>::INIT_LENGTH,
        );

        test_byte_set.init((
            test_enum::A,
            (TestStruct {
                val1: 100,
                val2: 200,
            },),
        ))?;

        match test_byte_set.immut()?.value() {
            TestEnumRef::A(val) => assert_eq!(
                val,
                &TestStruct {
                    val1: 100,
                    val2: 200
                }
            ),
            x => panic!("Invalid variant: {:?}", x),
        };

        // test_byte_set.mutable()?.set_c();

        Ok(())
    }

    // ---------------------------- Generated ----------------------------

    #[allow(clippy::unit_arg)]
    unsafe impl<'a, T> FrameworkFromBytes<'a> for TestEnumRefWrapper<'a, T>
    where
        T: UnsizedType,
    {
        fn from_bytes(bytes: &mut &'a [u8]) -> crate::Result<Self> {
            let bytes_len = bytes.len();
            let discriminant =
                bytemuck::checked::try_from_bytes::<PackedValueChecked<TestEnumDiscriminant>>(
                    &bytes[..size_of::<PackedValueChecked<TestEnumDiscriminant>>()],
                )
                .map_err(|_| ProgramError::InvalidAccountData)?;
            let ptr = NonNull::from(
                bytes.try_advance(size_of::<PackedValueChecked<TestEnumDiscriminant>>())?,
            )
            .cast();
            match discriminant.0 {
                TestEnumDiscriminant::A => {
                    let sub_ptr =
                        <<T as UnsizedType>::Ref<'_> as FrameworkFromBytes>::from_bytes(bytes)?;
                    Ok(Self {
                        phantom_ref: PhantomData,
                        ptr,
                        meta: TestEnumMeta {
                            inner: TestEnumMetaInner::A(sub_ptr.break_pointer().1),
                            byte_len: bytes_len - bytes.len(),
                        },
                    })
                }
                TestEnumDiscriminant::B => {
                    let sub_ptr = <<CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedType>::Ref<
                        '_,
                    > as FrameworkFromBytes>::from_bytes(bytes)?;
                    Ok(Self {
                        phantom_ref: PhantomData,
                        ptr,
                        meta: TestEnumMeta {
                            inner: TestEnumMetaInner::B(sub_ptr.break_pointer().1),
                            byte_len: bytes_len - bytes.len(),
                        },
                    })
                }
                TestEnumDiscriminant::C => {
                    let sub_ptr = <<List<PackedValue<u32>> as UnsizedType>::Ref<'_> as FrameworkFromBytes>::from_bytes(bytes)?;
                    Ok(Self {
                        phantom_ref: PhantomData,
                        ptr,
                        meta: TestEnumMeta {
                            inner: TestEnumMetaInner::C(sub_ptr.break_pointer().1),
                            byte_len: bytes_len - bytes.len(),
                        },
                    })
                }
            }
        }
    }
    impl<T> PointerBreakup for TestEnumRefWrapper<'_, T>
    where
        T: UnsizedType,
    {
        type Metadata = TestEnumMeta<T>;

        #[allow(clippy::unit_arg)]
        fn break_pointer(&self) -> (NonNull<()>, Self::Metadata) {
            (self.ptr, self.meta)
        }
    }
    impl<T> BuildPointer for TestEnumRefWrapper<'_, T>
    where
        T: UnsizedType,
    {
        unsafe fn build_pointer(pointee: NonNull<()>, metadata: Self::Metadata) -> Self {
            Self {
                ptr: pointee,
                meta: metadata,
                phantom_ref: PhantomData,
            }
        }
    }

    #[derive(Derivative)]
    #[derivative(
        Debug(
            bound = "<T as UnsizedType>::RefMut<'a>: Debug, <CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedType>::RefMut<'a>: Debug, <List<PackedValue<u32>> as UnsizedType>::RefMut<'a>: Debug"
        ),
        Clone(
            bound = "<T as UnsizedType>::RefMut<'a>: Clone, <CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedType>::RefMut<'a>: Clone, <List<PackedValue<u32>> as UnsizedType>::RefMut<'a>: Clone"
        ),
        Copy(
            bound = "<T as UnsizedType>::RefMut<'a>: Copy, <CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedType>::RefMut<'a>: Copy, <List<PackedValue<u32>> as UnsizedType>::RefMut<'a>: Copy"
        )
    )]
    pub enum TestEnumRefMut<'a, T>
    where
        T: UnsizedType,
    {
        A(<T as UnsizedType>::RefMut<'a>),
        B(<CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedType>::RefMut<'a>),
        C(<List<PackedValue<u32>> as UnsizedType>::RefMut<'a>),
    }
    #[derive(Derivative)]
    #[derivative(Debug(bound = "TestEnumMeta<T>: Debug"))]
    pub struct TestEnumRefMutWrapper<'a, T>
    where
        T: UnsizedType,
    {
        ptr: NonNull<()>,
        meta: TestEnumMeta<T>,
        #[derivative(Debug = "ignore")]
        resize: Box<dyn ResizeFn<'a, TestEnumMeta<T>>>,
    }
    impl<'a, T> TestEnumRefMutWrapper<'a, T>
    where
        T: UnsizedType,
    {
        unsafe fn data_ptr(&self) -> NonNull<()> {
            unsafe {
                NonNull::new_unchecked(
                    self.ptr
                        .as_ptr()
                        .byte_add(size_of::<PackedValueChecked<TestEnumDiscriminant>>()),
                )
            }
        }

        #[allow(clippy::unit_arg)]
        pub fn set_a<A>(&mut self, arg: A) -> Result<()>
        where
            for<'b> <T as UnsizedType>::RefMut<'b>: FrameworkInit<'b, A>,
        {
            let new_len =
                size_of::<u8>() + <<T as UnsizedType>::RefMut<'_> as FrameworkInit<A>>::INIT_LENGTH;
            (self.resize)(new_len, self.meta)?;
            sol_memset(
                unsafe {
                    &mut *slice_from_raw_parts_mut(
                        self.ptr.as_ptr().byte_add(size_of::<u8>()).cast(),
                        <<T as UnsizedType>::RefMut<'_> as FrameworkInit<A>>::INIT_LENGTH,
                    )
                },
                0,
                self.meta.byte_len - size_of::<u8>(),
            );
            let init = unsafe {
                <<T as UnsizedType>::RefMut<'_> as FrameworkInit<A>>::init(
                    &mut self.bytes[size_of::<u8>()..],
                    arg,
                    |_, _| panic!("Cannot resize during `set`"),
                )?
            }
            .break_pointer()
            .1;
            let new_meta = TestEnumMetaInner::A(
                // Safety: This is effectively moving `init`. The compiler is bugging out on the type.
                unsafe {
                    (&init as *const <<T as UnsizedType>::RefMut<'_> as PointerBreakup>::Metadata)
                        .cast::<<T as UnsizedType>::RefMeta>()
                        .read()
                },
            );
            self.meta.inner = new_meta;
            self.meta.byte_len = new_len;
            (self.resize)(new_len, self.meta)?;
            Ok(())
        }

        #[allow(clippy::unit_arg)]
        pub fn set_b<A>(&mut self, arg: A) -> Result<()>
        where
            for<'b> <CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedType>::RefMut<'b>:
                FrameworkInit<'b, A>,
        {
            let new_len =
                size_of::<u8>() + <<CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedType>::RefMut<'_> as FrameworkInit<A>>::INIT_LENGTH;
            (self.resize)(new_len, self.meta)?;
            sol_memset(
                unsafe {
                    &mut *slice_from_raw_parts_mut(
                        self.ptr.as_ptr().byte_add(size_of::<u8>()).cast(),
                        <<CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedType>::RefMut<'_> as FrameworkInit<A>>::INIT_LENGTH,
                    )
                },
                0,
                self.meta.byte_len - size_of::<u8>(),
            );
            let init = unsafe {
                <<CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedType>::RefMut<'_> as FrameworkInit<A>>::init(
                    &mut self.bytes[size_of::<u8>()..],
                    arg,
                    |_, _| panic!("Cannot resize during `set`"),
                )?
            }
                .break_pointer()
                .1;
            let new_meta = TestEnumMetaInner::B(
                // Safety: This is effectively moving `init`. The compiler is bugging out on the type.
                unsafe {
                    (&init as *const <<CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedType>::RefMut<'_> as PointerBreakup>::Metadata)
                        .cast::<<CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedType>::RefMeta>()
                        .read()
                },
            );
            self.meta.inner = new_meta;
            self.meta.byte_len = new_len;
            (self.resize)(new_len, self.meta)?;
            Ok(())
        }

        #[allow(clippy::unit_arg)]
        pub fn set_c<A>(&mut self, arg: A) -> Result<()>
        where
            for<'b> <List<PackedValue<u32>> as UnsizedType>::RefMut<'b>: FrameworkInit<'b, A>,
        {
            let new_len =
                size_of::<u8>() + <<List<PackedValue<u32>> as UnsizedType>::RefMut<'_> as FrameworkInit<A>>::INIT_LENGTH;
            (self.resize)(new_len, self.meta)?;
            sol_memset(
                unsafe {
                    &mut *slice_from_raw_parts_mut(
                        self.ptr.as_ptr().byte_add(size_of::<u8>()).cast(),
                        <<List<PackedValue<u32>> as UnsizedType>::RefMut<'_> as FrameworkInit<A>>::INIT_LENGTH,
                    )
                },
                0,
                self.meta.byte_len - size_of::<u8>(),
            );
            let init = unsafe {
                <<List<PackedValue<u32>> as UnsizedType>::RefMut<'_> as FrameworkInit<A>>::init(
                    &mut self.bytes[size_of::<u8>()..],
                    arg,
                    |_, _| panic!("Cannot resize during `set`"),
                )?
            }
            .break_pointer()
            .1;
            let new_meta = TestEnumMetaInner::C(
                // Safety: This is effectively moving `init`. The compiler is bugging out on the type.
                unsafe {
                    (&init as *const <<List<PackedValue<u32>> as UnsizedType>::RefMut<'_> as PointerBreakup>::Metadata)
                        .cast::<<List<PackedValue<u32>> as UnsizedType>::RefMeta>()
                        .read()
                },
            );
            self.meta.inner = new_meta;
            self.meta.byte_len = new_len;
            (self.resize)(new_len, self.meta)?;
            Ok(())
        }
    }
    impl<T> EnumRefWrapper for TestEnumRefMutWrapper<'_, T>
    where
        T: UnsizedType,
    {
        type Ref<'a> = TestEnumRef<'a, T> where Self: 'a;

        fn value<'b>(&'b self) -> Self::Ref<'b> {
            unsafe {
                let data_ptr = self.data_ptr();

                match self.meta.inner {
                TestEnumMetaInner::A(meta) => TestEnumRef::<'b, T>::A(
                    <<T as UnsizedType>::Ref<'b> as BuildPointer>::build_pointer(
                        data_ptr, meta,
                    ),
                ),
                TestEnumMetaInner::B(meta) => TestEnumRef::B(<<CombinedUnsized<
                    TestStruct,
                    List<u8, u8>,
                > as UnsizedType>::Ref<'b> as BuildPointer>::build_pointer(
                    data_ptr, meta
                )),
                TestEnumMetaInner::C(meta) => TestEnumRef::C(
                    <<List<PackedValue<u32>> as UnsizedType>::Ref<'b> as BuildPointer>::build_pointer(
                        data_ptr, meta,
                    ),
                ),
            }
            }
        }
    }
    impl<T> EnumRefMutWrapper for TestEnumRefMutWrapper<'_, T>
    where
        T: UnsizedType,
    {
        type RefMut<'a> = TestEnumRefMut<'a, T> where Self: 'a;

        fn value_mut<'b>(&'b mut self) -> Self::RefMut<'b> {
            unsafe {
                let data_ptr = self.data_ptr();
                let Self { ptr, meta, resize } = self;
                match meta.inner {
                TestEnumMetaInner::A(inner_meta) => TestEnumRefMut::A(
                    <<T as UnsizedType>::RefMut<'b> as BuildPointerMut>::
                    build_pointer_mut(
                        data_ptr, inner_meta, move |new_len, new_meta| {
                            meta.inner = TestEnumMetaInner::A(new_meta);
                            meta.byte_len = new_len + size_of::<PackedValueChecked<TestEnumDiscriminant>>();
                            *ptr = resize(meta.byte_len, *meta)?;
                            Ok(NonNull::new(ptr.as_ptr().byte_add(size_of::<PackedValueChecked<TestEnumDiscriminant>>())).unwrap())
                        }
                    ),
                ),
                TestEnumMetaInner::B(inner_meta) => TestEnumRefMut::B(
                    <<CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedType>::RefMut<'b>
                    as BuildPointerMut>::build_pointer_mut(
                        data_ptr, inner_meta, move |new_len, new_meta| {
                            meta.inner = TestEnumMetaInner::B(new_meta);
                            meta.byte_len = new_len + size_of::<PackedValueChecked<TestEnumDiscriminant>>();
                            *ptr = resize(meta.byte_len, *meta)?;
                            Ok(NonNull::new(ptr.as_ptr().byte_add(size_of::<PackedValueChecked<TestEnumDiscriminant>>())).unwrap())
                        }
                    ),
                ),
                TestEnumMetaInner::C(inner_meta) => TestEnumRefMut::C(
                    <<List<PackedValue<u32>> as UnsizedType>::RefMut<'b> as BuildPointerMut>::
                    build_pointer_mut(
                        data_ptr, inner_meta, move |new_len, new_meta| {
                            meta.inner = TestEnumMetaInner::C(new_meta);
                            meta.byte_len = new_len + size_of::<PackedValueChecked<TestEnumDiscriminant>>();
                            *ptr = resize(meta.byte_len, *meta)?;
                            Ok(NonNull::new(ptr.as_ptr().byte_add(size_of::<PackedValueChecked<TestEnumDiscriminant>>())).unwrap())
                        }
                    ),
                ),
            }
            }
        }
    }
    impl<T> Deref for TestEnumRefMutWrapper<'_, T>
    where
        T: UnsizedType,
    {
        type Target = TestEnum<T>;

        fn deref(&self) -> &Self::Target {
            unsafe { &*ptr::from_raw_parts(self.ptr.as_ptr(), self.meta.byte_len) }
        }
    }
    impl<T> DerefMut for TestEnumRefMutWrapper<'_, T>
    where
        T: UnsizedType,
    {
        fn deref_mut(&mut self) -> &mut Self::Target {
            unsafe { &mut *ptr::from_raw_parts_mut(self.ptr.as_ptr(), self.meta.byte_len) }
        }
    }
    impl<T> FrameworkSerialize for TestEnumRefMutWrapper<'_, T>
    where
        T: UnsizedType,
    {
        fn to_bytes(&self, output: &mut &mut [u8]) -> crate::Result<()> {
            unsafe {
                let data_ptr = self.data_ptr();
                match self.meta.inner {
                    TestEnumMetaInner::A(a) => {
                        0u8.to_bytes(output)?;
                        <<T as UnsizedType>::Ref<'_> as FrameworkSerialize>::to_bytes(
                            &<<T as UnsizedType>::Ref<'_> as BuildPointer>::build_pointer(
                                data_ptr, a,
                            ),
                            output,
                        )
                    }
                    TestEnumMetaInner::B(b) => {
                        4u8.to_bytes(output)?;
                        <<CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedType>::Ref<'_>
                    as FrameworkSerialize>::to_bytes(
                        &<<CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedType>::Ref<'_>
                        as BuildPointer>::build_pointer(data_ptr, b),
                        output,
                    )
                    }
                    TestEnumMetaInner::C(c) => {
                        5u8.to_bytes(output)?;
                        <<List<PackedValue<u32>> as UnsizedType>::Ref<'_> as FrameworkSerialize>::to_bytes(
                        &<<List<PackedValue<u32>> as UnsizedType>::Ref<'_> as BuildPointer>::
                        build_pointer(data_ptr, c),
                        output,
                    )
                    }
                }
            }
        }
    }
    impl<T> PointerBreakup for TestEnumRefMutWrapper<'_, T>
    where
        T: UnsizedType,
    {
        type Metadata = TestEnumMeta<T>;

        #[allow(clippy::unit_arg)]
        fn break_pointer(&self) -> (NonNull<()>, Self::Metadata) {
            (self.ptr, self.meta)
        }
    }
    #[allow(clippy::unit_arg)]
    unsafe impl<'a, T> FrameworkFromBytesMut<'a> for TestEnumRefMutWrapper<'a, T>
    where
        T: UnsizedType,
    {
        fn from_bytes_mut(
            bytes: &mut &'a mut [u8],
            resize: impl ResizeFn<'a, Self::Metadata>,
        ) -> crate::Result<Self> {
            let bytes_len = bytes.len();
            let discriminant =
                bytemuck::checked::try_from_bytes::<PackedValueChecked<TestEnumDiscriminant>>(
                    &bytes[..size_of::<PackedValueChecked<TestEnumDiscriminant>>()],
                )
                .map_err(|_| ProgramError::InvalidAccountData)?
                .0;
            let ptr = NonNull::from(
                bytes.try_advance(size_of::<PackedValueChecked<TestEnumDiscriminant>>())?,
            )
            .cast();
            match discriminant {
                TestEnumDiscriminant::A => {
                    let sub_ptr =
                        <<T as UnsizedType>::RefMut<'_> as FrameworkFromBytesMut>::from_bytes_mut(
                            bytes,
                            |_, _| panic!("Cannot resize during `from_bytes`"),
                        )?;
                    let broken = sub_ptr.break_pointer();
                    Ok(Self {
                        ptr,
                        meta: TestEnumMeta {
                            inner: TestEnumMetaInner::A(broken.1),
                            byte_len: bytes_len - bytes.len(),
                        },
                        resize: Box::new(resize),
                    })
                }
                TestEnumDiscriminant::B => {
                    let sub_ptr = <<CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedType>::RefMut<
                    '_,
                > as FrameworkFromBytesMut>::from_bytes_mut(
                    bytes,
                    |_, _| panic!("Cannot resize during `from_bytes`"),
                )?;
                    Ok(Self {
                        ptr,
                        meta: TestEnumMeta {
                            inner: TestEnumMetaInner::B(sub_ptr.break_pointer().1),
                            byte_len: bytes_len - bytes.len(),
                        },
                        resize: Box::new(resize),
                    })
                }
                TestEnumDiscriminant::C => {
                    let sub_ptr = <<List<PackedValue<u32>> as UnsizedType>::RefMut<'_> as FrameworkFromBytesMut>::from_bytes_mut(bytes, |_, _| panic!("Cannot resize during `from_bytes`"))?;
                    Ok(Self {
                        ptr,
                        meta: TestEnumMeta {
                            inner: TestEnumMetaInner::C(sub_ptr.break_pointer().1),
                            byte_len: bytes_len - bytes.len(),
                        },
                        resize: Box::new(resize),
                    })
                }
            }
        }
    }
    impl<'a, T> BuildPointerMut<'a> for TestEnumRefMutWrapper<'a, T>
    where
        T: UnsizedType,
    {
        unsafe fn build_pointer_mut(
            pointee: NonNull<()>,
            metadata: Self::Metadata,
            resize: impl ResizeFn<'a, Self::Metadata>,
        ) -> Self {
            Self {
                ptr: pointee,
                meta: metadata,
                resize: Box::new(resize),
            }
        }
    }

    pub mod test_enum {
        #[derive(Copy, Clone, Debug)]
        pub struct A;
        #[derive(Copy, Clone, Debug)]
        pub struct B;
        #[derive(Copy, Clone, Debug)]
        pub struct C;
    }

    #[allow(clippy::unit_arg)]
    unsafe impl<'a, A, T> FrameworkInit<'a, (test_enum::A, A)> for TestEnumRefMutWrapper<'a, T>
    where
        T: UnsizedType,
        <T as UnsizedType>::RefMut<'a>: FrameworkInit<'a, A>,
    {
        const INIT_LENGTH: usize =
            size_of::<u8>() + <<T as UnsizedType>::RefMut<'a> as FrameworkInit<'a, A>>::INIT_LENGTH;

        unsafe fn init(
            bytes: &'a mut [u8],
            (_, arg): (test_enum::A, A),
            resize: impl ResizeFn<'a, Self::Metadata>,
        ) -> crate::Result<Self> {
            debug_assert_eq!(
                bytes.len(),
                <Self as FrameworkInit<(test_enum::A, A)>>::INIT_LENGTH
            );
            let ptr = NonNull::from(&*bytes).cast();
            bytes[0] = TestEnumDiscriminant::A as u8;
            let sub_ptr = unsafe {
                <<T as UnsizedType>::RefMut<'_> as FrameworkInit<A>>::init(
                    &mut bytes[size_of::<u8>()..],
                    arg,
                    |_, _| panic!("Cannot resize during `init`"),
                )?
            };

            // // Just try uncommenting this if you don't like manual transmute (cast and read)
            // let broken: (NonNull<()>, ()) = { <&mut T as PointerBreakup>::break_pointer(&sub_ptr) };
            let broken =
                { <<T as UnsizedType>::RefMut<'_> as PointerBreakup>::break_pointer(&sub_ptr) };
            Ok(Self {
                ptr,
                meta: TestEnumMeta {
                    // // Just try uncommenting this if you don't like manual transmute (cast and read)
                    // inner: TestEnumMetaInner::A(broken.1),
                    inner: TestEnumMetaInner::A(unsafe {
                        (&broken.1
                            as *const <<T as UnsizedType>::RefMut<'_> as PointerBreakup>::Metadata)
                            .cast::<<T as UnsizedType>::RefMeta>()
                            .read()
                    }),
                    byte_len: <Self as FrameworkInit<'a, (test_enum::A, A)>>::INIT_LENGTH,
                },
                resize: Box::new(resize),
            })
        }
    }

    #[allow(clippy::unit_arg)]
    unsafe impl<'a, A, T> FrameworkInit<'a, (test_enum::B, A)> for TestEnumRefMutWrapper<'a, T>
    where
        T: UnsizedType,
        <CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedType>::RefMut<'a>:
            FrameworkInit<'a, A>,
    {
        const INIT_LENGTH: usize = size_of::<u8>()
        + <<CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedType>::RefMut<'a> as FrameworkInit<'a, A>>::INIT_LENGTH;

        unsafe fn init(
            bytes: &'a mut [u8],
            (_, arg): (test_enum::B, A),
            resize: impl ResizeFn<'a, Self::Metadata>,
        ) -> crate::Result<Self> {
            debug_assert_eq!(
                bytes.len(),
                <Self as FrameworkInit<(test_enum::B, A)>>::INIT_LENGTH
            );
            let ptr = NonNull::from(&*bytes).cast();
            bytes[0] = TestEnumDiscriminant::A as u8;
            let sub_ptr = unsafe {
                <<CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedType>::RefMut<'_> as FrameworkInit<A>>::init(
                &mut bytes[size_of::<u8>()..],
                arg,
                |_, _| panic!("Cannot resize during `init`"),
            )?
            };

            // // Just try uncommenting this if you don't like manual transmute (cast and read)
            // let broken: (NonNull<()>, ()) = { <&mut T as PointerBreakup>::break_pointer(&sub_ptr) };
            let broken = {
                <<CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedType>::RefMut<'_> as PointerBreakup>::break_pointer(
                &sub_ptr,
            )
            };
            println!("init: {}", type_name_of_val(&broken));
            Ok(Self {
                ptr,
                meta: TestEnumMeta {
                    // // Just try uncommenting this if you don't like manual transmute (cast and read)
                    // inner: TestEnumMetaInner::A(broken.1),
                    inner: TestEnumMetaInner::B(unsafe {
                        (&broken.1 as *const <<CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedType>::RefMut<'_> as PointerBreakup>::Metadata)
                        .cast::<<CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedType>::RefMeta>()
                        .read()
                    }),
                    byte_len: <Self as FrameworkInit<'a, (test_enum::B, A)>>::INIT_LENGTH,
                },
                resize: Box::new(resize),
            })
        }
    }

    #[allow(clippy::unit_arg)]
    unsafe impl<'a, A, T> FrameworkInit<'a, (test_enum::C, A)> for TestEnumRefMutWrapper<'a, T>
    where
        T: UnsizedType,
        <List<PackedValue<u32>> as UnsizedType>::RefMut<'a>: FrameworkInit<'a, A>,
    {
        const INIT_LENGTH: usize = size_of::<u8>()
        + <<List<PackedValue<u32>> as UnsizedType>::RefMut<'a> as FrameworkInit<'a, A>>::INIT_LENGTH;

        unsafe fn init(
            bytes: &'a mut [u8],
            (_, arg): (test_enum::C, A),
            resize: impl ResizeFn<'a, Self::Metadata>,
        ) -> crate::Result<Self> {
            debug_assert_eq!(
                bytes.len(),
                <Self as FrameworkInit<(test_enum::C, A)>>::INIT_LENGTH
            );
            let ptr = NonNull::from(&*bytes).cast();
            bytes[0] = TestEnumDiscriminant::A as u8;
            let sub_ptr = unsafe {
                <<List<PackedValue<u32>> as UnsizedType>::RefMut<'_> as FrameworkInit<A>>::init(
                    &mut bytes[size_of::<u8>()..],
                    arg,
                    |_, _| panic!("Cannot resize during `init`"),
                )?
            };

            // // Just try uncommenting this if you don't like manual transmute (cast and read)
            // let broken: (NonNull<()>, ()) = { <&mut T as PointerBreakup>::break_pointer(&sub_ptr) };
            let broken = {
                <<List<PackedValue<u32>> as UnsizedType>::RefMut<'_> as PointerBreakup>::break_pointer(
                &sub_ptr,
            )
            };
            println!("init: {}", type_name_of_val(&broken));
            Ok(Self {
                ptr,
                meta: TestEnumMeta {
                    // // Just try uncommenting this if you don't like manual transmute (cast and read)
                    // inner: TestEnumMetaInner::A(broken.1),
                    inner: TestEnumMetaInner::B(unsafe {
                        (&broken.1 as *const <<List<PackedValue<u32>> as UnsizedType>::RefMut<'_> as PointerBreakup>::Metadata)
                        .cast::<<CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedType>::RefMeta>()
                        .read()
                    }),
                    byte_len: <Self as FrameworkInit<'a, (test_enum::C, A)>>::INIT_LENGTH,
                },
                resize: Box::new(resize),
            })
        }
    }
}
