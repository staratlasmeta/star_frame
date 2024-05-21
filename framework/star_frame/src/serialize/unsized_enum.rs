use crate::prelude::*;
use crate::serialize::ref_wrapper::{AsBytes, RefWrapperTypes};

pub trait UnsizedEnum: UnsizedType {
    type Discriminant: 'static + Debug + CheckedBitPattern + NoUninit;
    type Enumerated<S: RefWrapperTypes<Ref = Self::RefData>>;

    fn discriminant<R>(r: R) -> Self::Discriminant
    where
        R: RefWrapperTypes<Ref = Self::RefData>,
        R::Super: AsBytes;
    fn enumerated<R>(r: R) -> Result<Self::Enumerated<R>>
    where
        R: RefWrapperTypes<Ref = Self::RefData>,
        R::Super: AsBytes;
}

// ---------------------------- Test Stuff ----------------------------

#[cfg(test)]
// TODO: Remove
#[allow(clippy::type_complexity)]
mod test {
    use crate::prelude::*;
    use crate::serialize::ref_wrapper::{
        AsBytes, AsMutBytes, RefBytes, RefBytesMut, RefResize, RefWrapper, RefWrapperMutExt,
        RefWrapperTypes,
    };
    use crate::serialize::test::TestByteSet;
    use crate::serialize::unsize::resize::Resize;
    use crate::serialize::unsize::FromBytesReturn;
    use crate::serialize::unsized_enum::UnsizedEnum;
    use crate::util::OffsetRef;
    use advance::Advance;
    use anyhow::bail;
    use bytemuck::checked::try_from_bytes;
    use bytemuck::{bytes_of, Pod, Zeroable};
    use derivative::Derivative;
    use derive_more::Constructor;
    use star_frame_proc::Align1;
    use std::marker::PhantomData;
    use std::mem::size_of;
    use typenum::Or;

    #[derive(Pod, Zeroable, Copy, Clone, Align1, Debug, PartialEq, Eq)]
    #[repr(C, packed)]
    pub struct TestStruct {
        val1: u32,
        val2: u64,
    }

    // /// Default repr is `u8`
    // enum TestEnum<T>
    // where
    //     T: ?Sized + UnsizedType,
    // {
    //     // A,
    //     A,
    //     // B(TestStruct, List<u8, u8>) = 4,
    //     B = 4,
    //     // C {
    //     //     list: List<PackedValue<u32>>,
    //     //     other: T,
    //     // },
    //     C,
    // }

    use test_enum_unsized::*;
    mod test_enum_unsized {
        use super::*;

        pub use discriminant::*;
        mod discriminant {
            use super::*;

            #[repr(u8)]
            #[derive(CheckedBitPattern, NoUninit, Copy, Clone, Debug, PartialEq, Eq)]
            pub enum TestEnumDiscriminant {
                A,
                B = 4,
                C,
            }
        }

        pub use meta::*;
        mod meta {
            use super::*;

            // TODO: Perfect derive macro
            #[derive(Derivative)]
            #[derivative(
                Clone(bound = ""),
                Copy(bound = ""),
                Debug(
                    bound = "<CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedType>::RefMeta: Debug, \
            <CombinedUnsized<List<PackedValue<u32>>, T> as UnsizedType>::RefMeta: Debug"
                )
            )]
            pub enum TestEnumMeta<T>
            where
                T: ?Sized + UnsizedType,
            {
                A,
                B(<CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedType>::RefMeta),
                C(<CombinedUnsized<List<PackedValue<u32>>, T> as UnsizedType>::RefMeta),
            }

            unsafe impl<S, T> RefBytes<S> for TestEnumMeta<T>
            where
                S: AsBytes,
                T: ?Sized + UnsizedType,
            {
                fn bytes(wrapper: &RefWrapper<S, Self>) -> Result<&[u8]> {
                    let mut bytes = wrapper.as_bytes()?;
                    bytes.try_advance(size_of::<PackedValueChecked<TestEnumDiscriminant>>())?;
                    Ok(bytes)
                }
            }

            unsafe impl<S, T> RefBytesMut<S> for TestEnumMeta<T>
            where
                S: AsMutBytes,
                T: ?Sized + UnsizedType,
            {
                fn bytes_mut(wrapper: &mut RefWrapper<S, Self>) -> Result<&mut [u8]> {
                    let mut bytes = wrapper.as_mut_bytes()?;
                    bytes.try_advance(size_of::<PackedValueChecked<TestEnumDiscriminant>>())?;
                    Ok(bytes)
                }
            }

            // A does not resize since it's empty
            unsafe impl<S, T> RefResize<S, TestEnumMeta<T>> for TestEnumMeta<T>
            where
                S: Resize<TestEnumMeta<T>>,
                T: ?Sized + UnsizedType,
            {
                unsafe fn resize(
                    wrapper: &mut RefWrapper<S, Self>,
                    new_byte_len: usize,
                    new_meta: TestEnumMeta<T>,
                ) -> Result<()> {
                    unsafe {
                        wrapper.sup_mut().resize(
                            new_byte_len + size_of::<PackedValueChecked<TestEnumDiscriminant>>(),
                            new_meta,
                        )?;
                        *wrapper.r_mut() = new_meta;
                    }
                    Ok(())
                }

                unsafe fn set_meta(
                    wrapper: &mut RefWrapper<S, Self>,
                    new_meta: TestEnumMeta<T>,
                ) -> Result<()> {
                    unsafe {
                        wrapper.sup_mut().set_meta(new_meta)?;
                        *wrapper.r_mut() = new_meta;
                    }
                    Ok(())
                }
            }
        }

        pub use ext::*;
        mod ext {
            use super::*;

            pub trait TestEnumRefExt<T>: Sized + RefWrapperTypes
            where
                T: ?Sized + UnsizedType,
            {
                fn discriminant(&self) -> TestEnumDiscriminant;

                fn enumerate(self) -> Result<TestEnumEnumerated<Self, T>>;
            }
            pub trait TestEnumMutExt<T>: RefWrapperMutExt + TestEnumRefExt<T>
            where
                T: ?Sized + UnsizedType,
            {
                fn set_a_owned(self) -> Result<RefWrapper<Self, TestEnumVariantA<T>>>;
                fn set_a(&mut self) -> Result<RefWrapper<&mut Self, TestEnumVariantA<T>>>;
                fn set_b_owned<A0, A1>(
                    self,
                    arg: TestEnumInitB<A0, A1>,
                ) -> Result<RefWrapper<Self, TestEnumVariantB<T>>>
                where
                    TestStruct: UnsizedInit<A0>,
                    List<u8, u8>: UnsizedInit<A1>;
                fn set_b<A0, A1>(
                    &mut self,
                    arg: TestEnumInitB<A0, A1>,
                ) -> Result<RefWrapper<&mut Self, TestEnumVariantB<T>>>
                where
                    TestStruct: UnsizedInit<A0>,
                    List<u8, u8>: UnsizedInit<A1>;
                fn set_c_owned<AList, AOther>(
                    self,
                    arg: TestEnumInitC<AList, AOther>,
                ) -> Result<RefWrapper<Self, TestEnumVariantC<T>>>
                where
                    List<PackedValue<u32>>: UnsizedInit<AList>,
                    T: UnsizedInit<AOther>;
                fn set_c<AList, AOther>(
                    &mut self,
                    arg: TestEnumInitC<AList, AOther>,
                ) -> Result<RefWrapper<&mut Self, TestEnumVariantC<T>>>
                where
                    List<PackedValue<u32>>: UnsizedInit<AList>,
                    T: UnsizedInit<AOther>;
            }

            impl<T, V> TestEnumRefExt<T> for V
            where
                V: RefWrapperTypes<Ref = TestEnumMeta<T>>,
                V::Super: AsBytes,
                T: ?Sized + UnsizedType,
            {
                fn discriminant(&self) -> TestEnumDiscriminant {
                    TestEnum::discriminant(self)
                }

                fn enumerate(self) -> Result<TestEnumEnumerated<Self, T>> {
                    TestEnum::enumerated(self)
                }
            }
            impl<T, V> TestEnumMutExt<T> for V
            where
                V: RefWrapperMutExt<Ref = TestEnumMeta<T>>,
                V::Super: Resize<TestEnumMeta<T>>,
                T: ?Sized + UnsizedType,
            {
                fn set_a_owned(mut self) -> Result<RefWrapper<Self, TestEnumVariantA<T>>> {
                    unsafe {
                        self.sup_mut().resize(
                            size_of::<PackedValueChecked<TestEnumDiscriminant>>(),
                            TestEnumMeta::A,
                        )?;
                        self.sup_mut()
                            .as_mut_bytes()?
                            .try_advance(size_of::<PackedValueChecked<TestEnumDiscriminant>>())?
                            .copy_from_slice(bytes_of(&PackedValueChecked(
                                TestEnumDiscriminant::A,
                            )));
                        Ok(RefWrapper::new(self, TestEnumVariantA::default()))
                    }
                }

                fn set_a(&mut self) -> Result<RefWrapper<&mut Self, TestEnumVariantA<T>>> {
                    self.set_a_owned()
                }

                fn set_b_owned<A0, A1>(
                    mut self,
                    arg: TestEnumInitB<A0, A1>,
                ) -> Result<RefWrapper<Self, TestEnumVariantB<T>>>
                where
                    TestStruct: UnsizedInit<A0>,
                    List<u8, u8>: UnsizedInit<A1>,
                {
                    unsafe {
                        let old_meta = *self.r();
                        self.sup_mut().resize(
                            size_of::<PackedValueChecked<TestEnumDiscriminant>>()
                                + <CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedInit<(
                                    A0,
                                    A1,
                                )>>::INIT_BYTES,
                            old_meta,
                        )?;
                        let mut bytes = self.sup_mut().as_mut_bytes()?;
                        bytes
                            .try_advance(size_of::<PackedValueChecked<TestEnumDiscriminant>>())?
                            .copy_from_slice(bytes_of(&PackedValueChecked(
                                TestEnumDiscriminant::B,
                            )));
                        let (_, meta) =
                            <CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedInit<
                                (A0, A1),
                            >>::init(
                                RefWrapper::new(
                                    bytes,
                                    OffsetRef(size_of::<PackedValueChecked<TestEnumDiscriminant>>()),
                                ),
                                (arg.0, arg.1),
                            )?;
                        self.sup_mut().set_meta(TestEnumMeta::B(meta))?;
                        Ok(RefWrapper::new(self, TestEnumVariantB::from(meta)))
                    }
                }

                fn set_b<A0, A1>(
                    &mut self,
                    arg: TestEnumInitB<A0, A1>,
                ) -> Result<RefWrapper<&mut Self, TestEnumVariantB<T>>>
                where
                    TestStruct: UnsizedInit<A0>,
                    List<u8, u8>: UnsizedInit<A1>,
                {
                    self.set_b_owned(arg)
                }

                fn set_c_owned<AList, AOther>(
                    mut self,
                    arg: TestEnumInitC<AList, AOther>,
                ) -> Result<RefWrapper<Self, TestEnumVariantC<T>>>
                where
                    List<PackedValue<u32>>: UnsizedInit<AList>,
                    T: UnsizedInit<AOther>,
                {
                    unsafe {
                        let old_meta = *self.r();
                        self.sup_mut().resize(
                            size_of::<PackedValueChecked<TestEnumDiscriminant>>()
                                + <CombinedUnsized<List<PackedValue<u32>>, T> as UnsizedInit<(
                                    AList,
                                    AOther,
                                )>>::INIT_BYTES,
                            old_meta,
                        )?;
                        let mut bytes = self.sup_mut().as_mut_bytes()?;
                        bytes
                            .try_advance(size_of::<PackedValueChecked<TestEnumDiscriminant>>())?
                            .copy_from_slice(bytes_of(&PackedValueChecked(
                                TestEnumDiscriminant::B,
                            )));
                        let (_, meta) =
                            <CombinedUnsized<List<PackedValue<u32>>, T> as UnsizedInit<(
                                AList,
                                AOther,
                            )>>::init(
                                RefWrapper::new(
                                    bytes,
                                    OffsetRef(size_of::<PackedValueChecked<TestEnumDiscriminant>>()),
                                ),
                                (arg.list, arg.other),
                            )?;
                        self.sup_mut().set_meta(TestEnumMeta::C(meta))?;
                        Ok(RefWrapper::new(self, TestEnumVariantC::from(meta)))
                    }
                }

                fn set_c<AList, AOther>(
                    &mut self,
                    arg: TestEnumInitC<AList, AOther>,
                ) -> Result<RefWrapper<&mut Self, TestEnumVariantC<T>>>
                where
                    List<PackedValue<u32>>: UnsizedInit<AList>,
                    T: UnsizedInit<AOther>,
                {
                    self.set_c_owned(arg)
                }
            }
        }

        pub use owned::*;
        mod owned {
            use super::*;

            // TODO: Perfect derive macro
            pub enum TestEnumOwned<T>
            where
                T: ?Sized + UnsizedType,
            {
                A,
                B(
                    <TestStruct as UnsizedType>::Owned,
                    <List<u8, u8> as UnsizedType>::Owned,
                ),
                C {
                    list: <List<PackedValue<u32>> as UnsizedType>::Owned,
                    other: <T as UnsizedType>::Owned,
                },
            }
        }

        pub use unsized_type::*;
        mod unsized_type {
            use super::*;

            pub struct TestEnum<T>(PhantomData<T>)
            where
                T: ?Sized + UnsizedType;

            pub type TestEnumIsUnsized<T> = Or<
                <CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedType>::IsUnsized,
                <CombinedUnsized<List<PackedValue<u32>>, T> as UnsizedType>::IsUnsized,
            >;

            unsafe impl<T> UnsizedType for TestEnum<T>
            where
                T: ?Sized + UnsizedType,
            {
                type RefMeta = TestEnumMeta<T>;
                type RefData = TestEnumMeta<T>;
                type Owned = TestEnumOwned<T>;
                type IsUnsized = TestEnumIsUnsized<T>;

                unsafe fn from_bytes<S: AsBytes>(
                    super_ref: S,
                ) -> Result<FromBytesReturn<S, Self::RefData, Self::RefMeta>> {
                    let mut bytes = super_ref.as_bytes()?;
                    let bytes_used = size_of::<PackedValueChecked<TestEnumDiscriminant>>();
                    let discriminant = try_from_bytes::<PackedValueChecked<TestEnumDiscriminant>>(
                        bytes.try_advance(bytes_used)?,
                    )?;

                    match discriminant.0 {
                        TestEnumDiscriminant::A => Ok(FromBytesReturn {
                            bytes_used,
                            meta: TestEnumMeta::A,
                            ref_wrapper: unsafe { RefWrapper::new(super_ref, TestEnumMeta::A) },
                        }),
                        TestEnumDiscriminant::B => {
                            let FromBytesReturn {
                                bytes_used: inner_bytes_used,
                                meta: inner_meta,
                                ..
                            } = unsafe {
                                CombinedUnsized::<TestStruct, List<u8, u8>>::from_bytes(bytes)?
                            };
                            Ok(FromBytesReturn {
                                bytes_used: bytes_used + inner_bytes_used,
                                meta: TestEnumMeta::B(inner_meta),
                                ref_wrapper: unsafe {
                                    RefWrapper::new(super_ref, TestEnumMeta::B(inner_meta))
                                },
                            })
                        }
                        TestEnumDiscriminant::C => {
                            let FromBytesReturn {
                                bytes_used: inner_bytes_used,
                                meta: inner_meta,
                                ..
                            } = unsafe {
                                CombinedUnsized::<List<PackedValue<u32>>, T>::from_bytes(bytes)?
                            };
                            Ok(FromBytesReturn {
                                bytes_used: bytes_used + inner_bytes_used,
                                meta: TestEnumMeta::C(inner_meta),
                                ref_wrapper: unsafe {
                                    RefWrapper::new(super_ref, TestEnumMeta::C(inner_meta))
                                },
                            })
                        }
                    }
                }

                unsafe fn from_bytes_and_meta<S: AsBytes>(
                    super_ref: S,
                    meta: Self::RefMeta,
                ) -> Result<FromBytesReturn<S, Self::RefData, Self::RefMeta>> {
                    match meta {
                        TestEnumMeta::A => Ok(FromBytesReturn {
                            bytes_used: size_of::<PackedValueChecked<TestEnumDiscriminant>>(),
                            meta: TestEnumMeta::A,
                            ref_wrapper: unsafe { RefWrapper::new(super_ref, TestEnumMeta::A) },
                        }),
                        TestEnumMeta::B(meta) => {
                            let FromBytesReturn {
                                bytes_used, meta, ..
                            } = unsafe {
                                CombinedUnsized::<TestStruct, List<u8, u8>>::from_bytes_and_meta(
                                    &super_ref, meta,
                                )?
                            };
                            Ok(FromBytesReturn {
                                bytes_used: size_of::<PackedValueChecked<TestEnumDiscriminant>>()
                                    + bytes_used,
                                meta: TestEnumMeta::B(meta),
                                ref_wrapper: unsafe {
                                    RefWrapper::new(super_ref, TestEnumMeta::B(meta))
                                },
                            })
                        }
                        TestEnumMeta::C(meta) => {
                            let FromBytesReturn {
                                bytes_used, meta, ..
                            } = unsafe {
                                CombinedUnsized::<List<PackedValue<u32>>, T>::from_bytes_and_meta(
                                    &super_ref, meta,
                                )?
                            };
                            Ok(FromBytesReturn {
                                bytes_used: size_of::<PackedValueChecked<TestEnumDiscriminant>>()
                                    + bytes_used,
                                meta: TestEnumMeta::C(meta),
                                ref_wrapper: unsafe {
                                    RefWrapper::new(super_ref, TestEnumMeta::C(meta))
                                },
                            })
                        }
                    }
                }

                fn owned<S: AsBytes>(r: RefWrapper<S, Self::RefData>) -> Result<Self::Owned> {
                    match r.r() {
                        TestEnumMeta::A => Ok(TestEnumOwned::A),
                        TestEnumMeta::B(meta) => {
                            let meta = *meta;
                            let r = unsafe {
                                CombinedUnsized::<TestStruct, List<u8, u8>>::from_bytes_and_meta(
                                    r, meta,
                                )?
                            };
                            let owned =
                                CombinedUnsized::<TestStruct, List<u8, u8>>::owned(r.ref_wrapper)?;
                            Ok(TestEnumOwned::B(owned.0, owned.1))
                        }
                        TestEnumMeta::C(meta) => {
                            let meta = *meta;
                            let r = unsafe {
                                CombinedUnsized::<List<PackedValue<u32>>, T>::from_bytes_and_meta(
                                    r, meta,
                                )?
                            };
                            let owned =
                                CombinedUnsized::<List<PackedValue<u32>>, T>::owned(r.ref_wrapper)?;
                            Ok(TestEnumOwned::C {
                                list: owned.0,
                                other: owned.1,
                            })
                        }
                    }
                }
            }
            impl<T> UnsizedEnum for TestEnum<T>
            where
                T: ?Sized + UnsizedType,
            {
                type Discriminant = TestEnumDiscriminant;
                type Enumerated<S: RefWrapperTypes<Ref = Self::RefData>> = TestEnumEnumerated<S, T>;

                fn discriminant<R>(r: R) -> Self::Discriminant
                where
                    R: RefWrapperTypes<Ref = Self::RefData>,
                    R::Super: AsBytes,
                {
                    match RefWrapperTypes::r(&r) {
                        TestEnumMeta::A => TestEnumDiscriminant::A,
                        TestEnumMeta::B(_) => TestEnumDiscriminant::B,
                        TestEnumMeta::C(_) => TestEnumDiscriminant::C,
                    }
                }

                fn enumerated<R>(r: R) -> Result<Self::Enumerated<R>>
                where
                    R: RefWrapperTypes<Ref = Self::RefData>,
                    R::Super: AsBytes,
                {
                    match RefWrapperTypes::r(&r) {
                        TestEnumMeta::A => Ok(TestEnumEnumerated::A),
                        TestEnumMeta::B(meta) => {
                            let meta = *meta;
                            Ok(TestEnumEnumerated::B(unsafe {
                                RefWrapper::new(r, <TestEnumVariantB<T>>::from(meta))
                            }))
                        }
                        TestEnumMeta::C(meta) => {
                            let meta = *meta;
                            Ok(TestEnumEnumerated::C(unsafe {
                                RefWrapper::new(r, <TestEnumVariantC<T>>::from(meta))
                            }))
                        }
                    }
                }
            }
        }

        pub use variant_a::*;
        mod variant_a {
            use super::*;

            #[derive(Constructor, Copy, Clone, Debug)]
            pub struct TestEnumInitA;

            impl<T> UnsizedInit<TestEnumInitA> for TestEnum<T>
            where
                T: ?Sized + UnsizedType,
            {
                const INIT_BYTES: usize = size_of::<PackedValueChecked<TestEnumDiscriminant>>();

                unsafe fn init<S: AsMutBytes>(
                    mut super_ref: S,
                    _arg: TestEnumInitA,
                ) -> Result<(RefWrapper<S, Self::RefData>, Self::RefMeta)> {
                    super_ref
                        .as_mut_bytes()?
                        .try_advance(size_of::<PackedValueChecked<TestEnumDiscriminant>>())?
                        .copy_from_slice(bytes_of(&PackedValueChecked(TestEnumDiscriminant::A)));
                    Ok((
                        unsafe { RefWrapper::new(super_ref, TestEnumMeta::A) },
                        TestEnumMeta::A,
                    ))
                }
            }

            #[derive(Derivative)]
            #[derivative(
                Copy(bound = ""),
                Clone(bound = ""),
                Debug(bound = ""),
                Default(bound = "")
            )]
            pub struct TestEnumVariantA<T>(PhantomData<T>)
            where
                T: ?Sized + UnsizedType;

            unsafe impl<S, T> RefBytes<S> for TestEnumVariantA<T>
            where
                S: AsBytes,
                T: ?Sized + UnsizedType,
            {
                fn bytes(wrapper: &RefWrapper<S, Self>) -> Result<&[u8]> {
                    wrapper.as_bytes()
                }
            }

            unsafe impl<S, T> RefBytesMut<S> for TestEnumVariantA<T>
            where
                S: AsMutBytes,
                T: ?Sized + UnsizedType,
            {
                fn bytes_mut(wrapper: &mut RefWrapper<S, Self>) -> Result<&mut [u8]> {
                    wrapper.as_mut_bytes()
                }
            }
            // `A` cannot resize since it's empty

            pub trait TestEnumVariantAExt<T>
            where
                T: ?Sized + UnsizedType,
            {
            }
        }

        pub use variant_b::*;
        mod variant_b {
            use super::*;

            #[derive(Constructor, Copy, Clone, Debug)]
            pub struct TestEnumInitB<A0, A1>(pub A0, pub A1);

            impl<T, A1, A0> UnsizedInit<TestEnumInitB<A0, A1>> for TestEnum<T>
            where
                TestStruct: UnsizedInit<A0>,
                List<u8, u8>: UnsizedInit<A1>,
                T: ?Sized + UnsizedType,
            {
                const INIT_BYTES: usize = size_of::<PackedValueChecked<TestEnumDiscriminant>>()
                    + <CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedInit<(A0, A1)>>::INIT_BYTES;

                unsafe fn init<S: AsMutBytes>(
                    mut super_ref: S,
                    arg: TestEnumInitB<A0, A1>,
                ) -> Result<(RefWrapper<S, Self::RefData>, Self::RefMeta)> {
                    let mut bytes = super_ref.as_mut_bytes()?;
                    bytes
                        .try_advance(size_of::<PackedValueChecked<TestEnumDiscriminant>>())?
                        .copy_from_slice(bytes_of(&PackedValueChecked(TestEnumDiscriminant::B)));
                    let meta = unsafe {
                        <CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedInit<(A0, A1)>>::init(
                            RefWrapper::new(
                                &mut super_ref,
                                OffsetRef(size_of::<PackedValueChecked<TestEnumDiscriminant>>()),
                            ),
                            (arg.0, arg.1),
                        )?
                        .1
                    };
                    Ok((
                        unsafe { RefWrapper::new(super_ref, TestEnumMeta::B(meta)) },
                        TestEnumMeta::B(meta),
                    ))
                }
            }

            #[derive(Derivative)]
            #[derivative(
                Copy(bound = ""),
                Clone(bound = ""),
                Debug(
                    bound = "<CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedType>::RefMeta: Debug"
                )
            )]
            pub struct TestEnumVariantB<T>(
                <CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedType>::RefMeta,
                PhantomData<T>,
            )
            where
                T: ?Sized + UnsizedType;

            impl<T> From<<CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedType>::RefMeta>
                for TestEnumVariantB<T>
            where
                T: ?Sized + UnsizedType,
            {
                fn from(
                    meta: <CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedType>::RefMeta,
                ) -> Self {
                    Self(meta, PhantomData)
                }
            }

            unsafe impl<S, T> RefBytes<S> for TestEnumVariantB<T>
            where
                S: AsBytes,
                T: ?Sized + UnsizedType,
            {
                fn bytes(wrapper: &RefWrapper<S, Self>) -> Result<&[u8]> {
                    wrapper.as_bytes()
                }
            }

            unsafe impl<S, T> RefBytesMut<S> for TestEnumVariantB<T>
            where
                S: AsMutBytes,
                T: ?Sized + UnsizedType,
            {
                fn bytes_mut(wrapper: &mut RefWrapper<S, Self>) -> Result<&mut [u8]> {
                    wrapper.as_mut_bytes()
                }
            }

            unsafe impl<S, T>
                RefResize<S, <CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedType>::RefMeta>
                for TestEnumVariantB<T>
            where
                S: Resize<TestEnumMeta<T>>,
                T: ?Sized + UnsizedType,
            {
                unsafe fn resize(
                    wrapper: &mut RefWrapper<S, Self>,
                    new_byte_len: usize,
                    new_meta: <CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedType>::RefMeta,
                ) -> Result<()> {
                    unsafe {
                        wrapper
                            .sup_mut()
                            .resize(new_byte_len, TestEnumMeta::B(new_meta))
                    }
                }

                unsafe fn set_meta(
                    wrapper: &mut RefWrapper<S, Self>,
                    new_meta: <CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedType>::RefMeta,
                ) -> Result<()> {
                    unsafe { wrapper.sup_mut().set_meta(TestEnumMeta::B(new_meta)) }
                }
            }
        }

        pub use variant_c::*;
        mod variant_c {
            use super::*;

            #[derive(Constructor, Copy, Clone, Debug)]
            pub struct TestEnumInitC<AList, AOther> {
                pub list: AList,
                pub other: AOther,
            }

            impl<T, AList, AOther> UnsizedInit<TestEnumInitC<AList, AOther>> for TestEnum<T>
            where
                List<PackedValue<u32>>: UnsizedInit<AList>,
                T: UnsizedInit<AOther>,
                T: ?Sized + UnsizedType,
            {
                const INIT_BYTES: usize = size_of::<PackedValueChecked<TestEnumDiscriminant>>()
                    + <CombinedUnsized<List<PackedValue<u32>>, T> as UnsizedInit<(
                        AList,
                        AOther,
                    )>>::INIT_BYTES;

                unsafe fn init<S: AsMutBytes>(
                    mut super_ref: S,
                    arg: TestEnumInitC<AList, AOther>,
                ) -> Result<(RefWrapper<S, Self::RefData>, Self::RefMeta)> {
                    let mut bytes = super_ref.as_mut_bytes()?;
                    bytes
                        .try_advance(size_of::<PackedValueChecked<TestEnumDiscriminant>>())?
                        .copy_from_slice(bytes_of(&PackedValueChecked(TestEnumDiscriminant::B)));
                    let meta = unsafe {
                        <CombinedUnsized<List<PackedValue<u32>>, T> as UnsizedInit<(
                            AList,
                            AOther,
                        )>>::init(
                            RefWrapper::new(
                                &mut super_ref,
                                OffsetRef(size_of::<PackedValueChecked<TestEnumDiscriminant>>()),
                            ),
                            (arg.list, arg.other),
                        )?
                        .1
                    };
                    Ok((
                        unsafe { RefWrapper::new(super_ref, TestEnumMeta::C(meta)) },
                        TestEnumMeta::C(meta),
                    ))
                }
            }

            #[derive(Derivative)]
            #[derivative(
                Copy(bound = ""),
                Clone(bound = ""),
                Debug(
                    bound = "<CombinedUnsized<List<PackedValue<u32>>, T> as UnsizedType>::RefMeta: Debug"
                )
            )]
            pub struct TestEnumVariantC<T>(
                <CombinedUnsized<List<PackedValue<u32>>, T> as UnsizedType>::RefMeta,
                PhantomData<T>,
            )
            where
                T: ?Sized + UnsizedType;

            impl<T> From<<CombinedUnsized<List<PackedValue<u32>>, T> as UnsizedType>::RefMeta>
                for TestEnumVariantC<T>
            where
                T: ?Sized + UnsizedType,
            {
                fn from(
                    meta: <CombinedUnsized<List<PackedValue<u32>>, T> as UnsizedType>::RefMeta,
                ) -> Self {
                    Self(meta, PhantomData)
                }
            }

            unsafe impl<S, T> RefBytes<S> for TestEnumVariantC<T>
            where
                S: AsBytes,
                T: ?Sized + UnsizedType,
            {
                fn bytes(wrapper: &RefWrapper<S, Self>) -> Result<&[u8]> {
                    wrapper.as_bytes()
                }
            }

            unsafe impl<S, T> RefBytesMut<S> for TestEnumVariantC<T>
            where
                S: AsMutBytes,
                T: ?Sized + UnsizedType,
            {
                fn bytes_mut(wrapper: &mut RefWrapper<S, Self>) -> Result<&mut [u8]> {
                    wrapper.as_mut_bytes()
                }
            }

            unsafe impl<S, T>
                RefResize<S, <CombinedUnsized<List<PackedValue<u32>>, T> as UnsizedType>::RefMeta>
                for TestEnumVariantB<T>
            where
                S: Resize<TestEnumMeta<T>>,
                T: ?Sized + UnsizedType,
            {
                unsafe fn resize(
                    wrapper: &mut RefWrapper<S, Self>,
                    new_byte_len: usize,
                    new_meta: <CombinedUnsized<List<PackedValue<u32>>, T> as UnsizedType>::RefMeta,
                ) -> Result<()> {
                    unsafe {
                        wrapper
                            .sup_mut()
                            .resize(new_byte_len, TestEnumMeta::C(new_meta))
                    }
                }

                unsafe fn set_meta(
                    wrapper: &mut RefWrapper<S, Self>,
                    new_meta: <CombinedUnsized<List<PackedValue<u32>>, T> as UnsizedType>::RefMeta,
                ) -> Result<()> {
                    unsafe { wrapper.sup_mut().set_meta(TestEnumMeta::C(new_meta)) }
                }
            }
        }

        pub use enumerated::*;
        mod enumerated {
            use super::*;

            #[derive(Derivative)]
            #[derivative(
                Copy(bound = "
                RefWrapper<S, TestEnumVariantB<T>>: Copy,
                RefWrapper<S, TestEnumVariantC<T>>: Copy,
            "),
                Clone(bound = "
                RefWrapper<S, TestEnumVariantB<T>>: Clone,
                RefWrapper<S, TestEnumVariantC<T>>: Clone,
            "),
                Debug(bound = "
                RefWrapper<S, TestEnumVariantB<T>>: Debug,
                RefWrapper<S, TestEnumVariantC<T>>: Debug,
            ")
            )]
            pub enum TestEnumEnumerated<S, T>
            where
                T: ?Sized + UnsizedType,
            {
                A,
                B(RefWrapper<S, TestEnumVariantB<T>>),
                C(RefWrapper<S, TestEnumVariantC<T>>),
            }
        }
    }

    #[test]
    fn test_enum() -> Result<()> {
        type TestEnumImpl = TestEnum<List<u8, u8>>;
        let mut test = TestByteSet::<TestEnumImpl>::new(TestEnumInitA)?;
        let test_mut = test.mutable()?;
        let discriminant = TestEnumImpl::discriminant(&test_mut);
        assert_eq!(discriminant, TestEnumDiscriminant::A);
        let enumerated = TestEnumImpl::enumerated(test_mut)?;
        assert!(matches!(enumerated, TestEnumEnumerated::A));

        let test_mut = test.re_init(TestEnumInitB(
            TestStruct {
                val1: 1000,
                val2: 200,
            },
            (),
        ))?;
        let discriminant = TestEnumImpl::discriminant(&test_mut);
        assert_eq!(discriminant, TestEnumDiscriminant::B);
        let enumerated = TestEnumImpl::enumerated(test_mut)?;
        let _b = match enumerated {
            TestEnumEnumerated::B(b) => b,
            x => bail!("Expected B, found {x:?}"),
        };

        let test_mut = test.re_init(TestEnumInitC {
            list: (),
            other: (),
        })?;
        let discriminant = TestEnumImpl::discriminant(&test_mut);
        assert_eq!(discriminant, TestEnumDiscriminant::C);
        let enumerated = TestEnumImpl::enumerated(test_mut)?;
        let _c = match enumerated {
            TestEnumEnumerated::C(c) => c,
            x => bail!("Expected C, found {x:?}"),
        };

        Ok(())
    }
}
