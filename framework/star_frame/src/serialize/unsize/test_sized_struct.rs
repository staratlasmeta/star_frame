use crate::prelude::*;
use crate::prelude::{
    CombinedExt, CombinedTRef, CombinedURef, CombinedUnsized, List, UnsizedInit, UnsizedType,
};
use crate::serialize::ref_wrapper::{
    AsBytes, AsMutBytes, RefBytes, RefBytesMut, RefResize, RefWrapper, RefWrapperMutExt,
    RefWrapperTypes,
};
use crate::serialize::unsize::resize::Resize;
use crate::serialize::unsize::test::TestStruct;
use crate::serialize::unsize::FromBytesReturn;
use star_frame::prelude::CombinedRef;
use star_frame::serialize::unsize::test::CombinedTest;
use star_frame_proc::Align1;

// #[unsized_type]
// pub struct CombinedTest3 {
//     pub sized1: u8,
//     pub sized2: PackedValue<u16>,
//     pub sized3: u8,
//     #[unsized_start]
//     pub list1: List<u8>,
//     pub list2: List<TestStruct>,
//     pub other: CombinedTest,
// }

#[derive(Debug, Copy, Clone, CheckedBitPattern, Zeroable, Align1, NoUninit, PartialEq, Eq)]
#[repr(C, packed)]
pub struct CombinedTest3Sized {
    pub sized1: bool,
    pub sized2: PackedValue<u16>,
    pub sized3: u8,
}

pub use combined_test_3_impls::*;

mod combined_test_3_impls {
    use super::*;
    use crate::serialize::ref_wrapper::RefDerefMut;
    use crate::serialize::unsize::checked::Zeroed;
    use star_frame::serialize::ref_wrapper::RefDeref;
    use std::ops::{Deref, DerefMut};

    type SizedField = CombinedTest3Sized;
    type Field1 = List<u8>;
    type Field2 = List<TestStruct>;
    type Field3 = CombinedTest;

    #[derive(Debug, Align1)]
    #[repr(transparent)]
    pub struct CombinedTest3(CombinedTest3Inner);

    type CombinedTest3Inner =
        CombinedUnsized<SizedField, CombinedUnsized<Field1, CombinedUnsized<Field2, Field3>>>;

    #[derive(Debug, Copy, Clone)]
    #[repr(transparent)]
    pub struct CombinedTest3Meta(<CombinedTest3Inner as UnsizedType>::RefMeta);

    // TODO: Where clause for derives?
    #[derive(Debug, Copy, Clone)]
    #[repr(transparent)]
    pub struct CombinedTest3Ref(<CombinedTest3Inner as UnsizedType>::RefData);

    #[derive(Debug)]
    pub struct CombinedTest3Owned {
        sized_struct: <SizedField as UnsizedType>::Owned,
        pub list1: <Field1 as UnsizedType>::Owned,
        pub list2: <Field2 as UnsizedType>::Owned,
        pub other: <Field3 as UnsizedType>::Owned,
    }

    impl Deref for CombinedTest3Owned {
        type Target = SizedField;
        fn deref(&self) -> &Self::Target {
            &self.sized_struct
        }
    }

    impl DerefMut for CombinedTest3Owned {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.sized_struct
        }
    }

    unsafe impl UnsizedType for CombinedTest3 {
        type RefMeta = CombinedTest3Meta;
        type RefData = CombinedTest3Ref;
        type Owned = CombinedTest3Owned;
        type IsUnsized = <CombinedTest3Inner as UnsizedType>::IsUnsized;

        unsafe fn from_bytes<S: AsBytes>(
            super_ref: S,
        ) -> anyhow::Result<FromBytesReturn<S, Self::RefData, Self::RefMeta>> {
            unsafe {
                Ok(<CombinedTest3Inner as UnsizedType>::from_bytes(super_ref)?
                    .map_ref(|_, r| CombinedTest3Ref(r))
                    .map_meta(CombinedTest3Meta))
            }
        }

        fn owned<S: AsBytes>(r: RefWrapper<S, Self::RefData>) -> anyhow::Result<Self::Owned> {
            let (sized_struct, (list1, (list2, other))) =
                <CombinedTest3Inner as UnsizedType>::owned(unsafe { r.wrap_r(|_, r| r.0) })?;
            Ok(CombinedTest3Owned {
                sized_struct,
                list1,
                list2,
                other,
            })
        }
    }

    // CombinedUnsized<CombinedTest3Sized, CombinedUnsized<<List<u8>, CombinedUnsized<List<TestStruct>, CombinedTest>>>
    pub struct CombinedTest3Init<SizedStruct, List1, List2, Other> {
        pub sized_struct: SizedStruct,
        pub list1: List1,
        pub list2: List2,
        pub other: Other,
    }
    impl<SizedStruct, List1, List2, Other>
        UnsizedInit<CombinedTest3Init<SizedStruct, List1, List2, Other>> for CombinedTest3
    where
        SizedField: UnsizedInit<SizedStruct>,
        Field1: UnsizedInit<List1>,
        Field2: UnsizedInit<List2>,
        Field3: UnsizedInit<Other>,
    {
        const INIT_BYTES: usize =
            <CombinedTest3Inner as UnsizedInit<(SizedStruct, (List1, (List2, Other)))>>::INIT_BYTES;

        unsafe fn init<S: AsMutBytes>(
            super_ref: S,
            arg: CombinedTest3Init<SizedStruct, List1, List2, Other>,
        ) -> anyhow::Result<(RefWrapper<S, Self::RefData>, Self::RefMeta)> {
            unsafe {
                let (r, m) = <CombinedTest3Inner as UnsizedInit<(
                    SizedStruct,
                    (List1, (List2, Other)),
                )>>::init(
                    super_ref,
                    (arg.sized_struct, (arg.list1, (arg.list2, arg.other))),
                )?;
                Ok((r.wrap_r(|_, r| CombinedTest3Ref(r)), CombinedTest3Meta(m)))
            }
        }
    }

    impl UnsizedInit<()> for CombinedTest3Sized
    where
        CombinedTest3Sized: UnsizedInit<Zeroed>,
    {
        const INIT_BYTES: usize = <CombinedTest3Sized as UnsizedInit<Zeroed>>::INIT_BYTES;

        unsafe fn init<S: AsMutBytes>(
            super_ref: S,
            _arg: (),
        ) -> Result<(RefWrapper<S, Self::RefData>, Self::RefMeta)> {
            //todo: is this valid?
            unsafe { CombinedTest3Sized::init(super_ref, Zeroed) }
        }
    }

    impl UnsizedInit<()> for CombinedTest3
    where
        SizedField: UnsizedInit<()>,
        Field1: UnsizedInit<()>,
        Field2: UnsizedInit<()>,
        Field3: UnsizedInit<()>,
    {
        const INIT_BYTES: usize = <CombinedTest3Inner as UnsizedInit<()>>::INIT_BYTES;

        unsafe fn init<S: AsMutBytes>(
            super_ref: S,
            arg: (),
        ) -> anyhow::Result<(RefWrapper<S, Self::RefData>, Self::RefMeta)> {
            unsafe {
                let (r, m) = <CombinedTest3Inner as UnsizedInit<()>>::init(super_ref, arg)?;
                Ok((r.wrap_r(|_, r| CombinedTest3Ref(r)), CombinedTest3Meta(m)))
            }
        }
    }

    unsafe impl<S> RefBytes<S> for CombinedTest3Ref
    where
        S: AsBytes,
    {
        fn bytes(wrapper: &RefWrapper<S, Self>) -> anyhow::Result<&[u8]> {
            wrapper.sup().as_bytes()
        }
    }
    unsafe impl<S> RefBytesMut<S> for CombinedTest3Ref
    where
        S: AsMutBytes,
    {
        fn bytes_mut(wrapper: &mut RefWrapper<S, Self>) -> anyhow::Result<&mut [u8]> {
            unsafe { wrapper.sup_mut().as_mut_bytes() }
        }
    }
    unsafe impl<S> RefResize<S, <CombinedTest3Inner as UnsizedType>::RefMeta> for CombinedTest3Ref
    where
        S: Resize<CombinedTest3Meta>,
    {
        unsafe fn resize(
            wrapper: &mut RefWrapper<S, Self>,
            new_byte_len: usize,
            new_meta: <CombinedTest3Inner as UnsizedType>::RefMeta,
        ) -> anyhow::Result<()> {
            unsafe {
                wrapper.r_mut().0 = CombinedRef::new(new_meta);
                wrapper
                    .sup_mut()
                    .resize(new_byte_len, CombinedTest3Meta(new_meta))
            }
        }

        unsafe fn set_meta(
            wrapper: &mut RefWrapper<S, Self>,
            new_meta: <CombinedTest3Inner as UnsizedType>::RefMeta,
        ) -> anyhow::Result<()> {
            unsafe {
                wrapper.r_mut().0 = CombinedRef::new(new_meta);
                wrapper.sup_mut().set_meta(CombinedTest3Meta(new_meta))
            }
        }
    }

    impl<S> RefDeref<S> for CombinedTest3Ref
    where
        S: AsBytes,
    {
        type Target = CombinedTest3Sized;
        fn deref(wrapper: &RefWrapper<S, Self>) -> &Self::Target {
            let bytes = wrapper.sup().as_bytes().expect("Invalid bytes");
            unsafe { &*bytes.as_ptr().cast::<Self::Target>() }
        }
    }

    impl<S> RefDerefMut<S> for CombinedTest3Ref
    where
        S: AsMutBytes,
    {
        fn deref_mut(wrapper: &mut RefWrapper<S, Self>) -> &mut Self::Target {
            let bytes = unsafe { wrapper.sup_mut() }
                .as_mut_bytes()
                .expect("Invalid bytes");
            unsafe { &mut *bytes.as_mut_ptr().cast::<Self::Target>() }
        }
    }

    // impl<S> RefDeref<S> for CombinedTest3Ref
    //     where
    //         S: AsBytes,
    // {
    //     type Target = CombinedTest3Sized;
    //     fn deref(wrapper: &RefWrapper<S, Self>) -> &Self::Target {
    //         let bytes = wrapper.sup().as_bytes().expect("Invalid bytes");
    //         unsafe { &*(bytes.as_ptr() as *const Self::Target) }
    //     }
    // }

    // type SizedStruct<S> = RefWrapper<RefWrapper<RefWrapper<S, CombinedTest3Inner>>, <SizedField as UnsizedType>::RefData>;

    // type List1<S> =
    // RefWrapper<
    //     RefWrapper<
    //         RefWrapper<S, <CombinedUnsized<
    //             List<u8>,
    //             CombinedUnsized<List<TestStruct>, CombinedTest>,
    //         > as UnsizedType>::RefData>,
    //         CombinedTRef<List<u8>, CombinedUnsized<List<TestStruct>, CombinedTest>>,
    //     >,
    //     <List<u8> as UnsizedType>::RefData,
    // >;
    // type List2<S> = RefWrapper<
    //     RefWrapper<RefWrapper<RefWrapper<
    //         RefWrapper<S, <CombinedUnsized<
    //             List<u8>,
    //             CombinedUnsized<List<TestStruct>, CombinedTest>,
    //         > as UnsizedType>::RefData>,
    //         CombinedURef<List<u8>, CombinedUnsized<List<TestStruct>, CombinedTest>>,
    //     >, <CombinedUnsized<List<TestStruct>, CombinedTest> as UnsizedType>::RefData>, CombinedTRef<List<TestStruct>, CombinedTest>>,
    //     <List<TestStruct> as UnsizedType>::RefData,
    // >;
    // type Other<S> = RefWrapper<
    //     RefWrapper<RefWrapper<RefWrapper<
    //         RefWrapper<S, <CombinedUnsized<
    //             List<u8>,
    //             CombinedUnsized<List<TestStruct>, CombinedTest>,
    //         > as UnsizedType>::RefData>,
    //         CombinedURef<List<u8>, CombinedUnsized<List<TestStruct>, CombinedTest>>,
    //     >, <CombinedUnsized<List<TestStruct>, CombinedTest> as UnsizedType>::RefData>, CombinedURef<List<TestStruct>, CombinedTest>>,
    //     <CombinedTest as UnsizedType>::RefData,
    // >;
    //

    type CombinedTest3RefInner =
        CombinedRef<SizedField, CombinedUnsized<Field1, CombinedUnsized<Field2, Field3>>>;

    type SizedStruct<S> = RefWrapper<
        RefWrapper<
            RefWrapper<S, CombinedTest3RefInner>,
            CombinedTRef<SizedField, CombinedUnsized<Field1, CombinedUnsized<Field2, Field3>>>,
        >,
        <SizedField as UnsizedType>::RefData,
    >;

    type List1<S> = RefWrapper<
        RefWrapper<
            RefWrapper<
                RefWrapper<
                    RefWrapper<S, CombinedTest3RefInner>,
                    CombinedURef<
                        SizedField,
                        CombinedUnsized<Field1, CombinedUnsized<Field2, Field3>>,
                    >,
                >,
                CombinedRef<Field1, CombinedUnsized<Field2, Field3>>,
            >,
            CombinedTRef<Field1, CombinedUnsized<Field2, Field3>>,
        >,
        <Field1 as UnsizedType>::RefData,
    >;

    type List2<S> = RefWrapper<
        RefWrapper<
            RefWrapper<
                RefWrapper<
                    RefWrapper<
                        RefWrapper<
                            RefWrapper<
                                S,
                                CombinedRef<
                                    SizedField,
                                    CombinedUnsized<Field1, CombinedUnsized<Field2, Field3>>,
                                >,
                            >,
                            CombinedURef<
                                SizedField,
                                CombinedUnsized<Field1, CombinedUnsized<Field2, Field3>>,
                            >,
                        >,
                        CombinedRef<Field1, CombinedUnsized<Field2, Field3>>,
                    >,
                    CombinedURef<Field1, CombinedUnsized<Field2, Field3>>,
                >,
                CombinedRef<Field2, Field3>,
            >,
            CombinedTRef<Field2, Field3>,
        >,
        <Field2 as UnsizedType>::RefData,
    >;

    type Other<S> = RefWrapper<
        RefWrapper<
            RefWrapper<
                RefWrapper<
                    RefWrapper<
                        RefWrapper<
                            RefWrapper<
                                S,
                                CombinedRef<
                                    SizedField,
                                    CombinedUnsized<Field1, CombinedUnsized<Field2, Field3>>,
                                >,
                            >,
                            CombinedURef<
                                SizedField,
                                CombinedUnsized<Field1, CombinedUnsized<Field2, Field3>>,
                            >,
                        >,
                        CombinedRef<Field1, CombinedUnsized<Field2, Field3>>,
                    >,
                    CombinedURef<Field1, CombinedUnsized<Field2, Field3>>,
                >,
                CombinedRef<Field2, Field3>,
            >,
            CombinedURef<Field2, Field3>,
        >,
        <Field3 as UnsizedType>::RefData,
    >;

    pub trait CombinedTest3Ext: Sized + RefWrapperTypes {
        // fn sized_struct(self) -> anyhow::Result<SizedStruct<Self>>;
        fn list1(self) -> anyhow::Result<List1<Self>>;
        fn list2(self) -> anyhow::Result<List2<Self>>;
        fn other(self) -> anyhow::Result<Other<Self>>;
    }
    impl<R> CombinedTest3Ext for R
    where
        R: RefWrapperTypes<Ref = CombinedTest3Ref> + AsBytes,
    {
        // fn sized_struct(self) -> anyhow::Result<SizedStruct<Self>> {
        //     let r = self.r().0;
        //     unsafe { RefWrapper::new(self, r).t() }
        // }

        fn list1(self) -> anyhow::Result<List1<Self>> {
            let r = self.r().0;
            unsafe { RefWrapper::new(self, r).u()?.t() }
        }

        fn list2(self) -> anyhow::Result<List2<Self>> {
            let r = self.r().0;
            unsafe { RefWrapper::new(self, r).u()?.u()?.t() }
        }

        fn other(self) -> anyhow::Result<Other<Self>> {
            let r = self.r().0;
            unsafe { RefWrapper::new(self, r).u()?.u()?.u() }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;
    use crate::serialize::list::ListExt;
    use crate::serialize::ref_wrapper::RefWrapper;
    use crate::serialize::test::TestByteSet;
    use crate::serialize::unsize::test::{CombinedTestExt, TestStruct};
    use crate::serialize::unsize::test2::{
        CombinedTest2, CombinedTest2Ext, CombinedTest2Meta, CombinedTest2Ref,
    };
    use star_frame::serialize::unsize::resize::Resize;

    fn cool(
        r: &mut RefWrapper<impl Resize<CombinedTest2Meta>, CombinedTest2Ref>,
        val: u32,
    ) -> anyhow::Result<()> {
        r.list1()?.push(0)?;
        r.list2()?.insert(0, TestStruct { val1: val, val2: 0 })?;
        r.other()?.list1()?.push(0)?;
        r.other()?
            .list2()?
            .insert(0, TestStruct { val1: val, val2: 0 })?;
        Ok(())
    }

    // #[unsized_type]
    // pub struct CombinedTest3 {
    //     pub sized1: bool,
    //     pub sized2: PackedValue<u16>,
    //     pub sized3: u8,
    //     #[unsized_start]
    //     pub list1: List<u8>,
    //     pub list2: List<TestStruct>,
    //     pub other: CombinedTest,
    // }

    #[test]
    fn test() -> anyhow::Result<()> {
        let mut bytes = TestByteSet::<CombinedTest3>::new(())?;
        let mut r = bytes.mutable()?;
        r.sized1 = true;
        r.sized2 = PackedValue(69);
        r.sized3 = 2;
        (&mut r).list1()?.push(10)?;
        (&mut r).list2()?.push(TestStruct { val1: 1, val2: 0 })?;
        (&mut r).other()?.list1()?.push(10)?;
        (&mut r)
            .other()?
            .list2()?
            .push(TestStruct { val1: 1, val2: 0 })?;
        println!("{:#?}", <CombinedTest3 as UnsizedType>::owned(r));
        Ok(())
    }

    type CombinedTest2RefWrapper<S> = RefWrapper<S, CombinedTest2Ref>;
    #[test]
    fn test_stuff() -> anyhow::Result<()> {
        let bytes = vec![0u8; 100];
        let combined: CombinedTest2RefWrapper<_> =
            unsafe { CombinedTest2::from_bytes(bytes).unwrap() }.ref_wrapper;
        println!("{combined:?}");
        let mut list = combined.list1().unwrap();
        list.push(1)?;
        list.insert(0, 2)?;
        println!("{:?}", list.len());
        println!("{:?}", list.as_slice());
        Ok(())
    }
}
