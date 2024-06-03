use crate::prelude::{
    CombinedExt, CombinedTRef, CombinedURef, CombinedUnsized, List, UnsizedInit, UnsizedType,
};
use crate::serialize::ref_wrapper::{
    AsBytes, AsMutBytes, RefBytes, RefBytesMut, RefResize, RefWrapper, RefWrapperMutExt,
    RefWrapperTypes,
};
use crate::serialize::unsize::init::Zeroed;
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

// pub struct CombinedTest3Sized {
//     pub sized1: u8,
//     pub sized2: PackedValue<u16>,
//     pub sized3: u8,
// }

// CombinedUnsized<CombinedTest3Sized, CombinedUnsized<<List<u8>, CombinedUnsized<List<TestStruct>, CombinedTest>>>

// #[unsized_type]
// pub struct CombinedTest2 {
//     pub list1: List<u8>,
//     pub list2: List<TestStruct>,
//     pub other: CombinedTest,
// }

#[derive(Debug, Align1)]
#[repr(transparent)]
pub struct CombinedTest2(
    CombinedUnsized<List<u8>, CombinedUnsized<List<TestStruct>, CombinedTest>>,
);
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct CombinedTest2Meta(
    <CombinedUnsized<List<u8>, CombinedUnsized<List<TestStruct>, CombinedTest>> as UnsizedType>::RefMeta,
);
// TODO: Where clause for derives?
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct CombinedTest2Ref(
    <CombinedUnsized<List<u8>, CombinedUnsized<List<TestStruct>, CombinedTest>> as UnsizedType>::RefData,
);
pub struct CombinedTest2Owned {
    pub list1: <List<u8> as UnsizedType>::Owned,
    pub list2: <List<TestStruct> as UnsizedType>::Owned,
    pub other: <CombinedTest as UnsizedType>::Owned,
}

unsafe impl UnsizedType for CombinedTest2 {
    type RefMeta = CombinedTest2Meta;
    type RefData = CombinedTest2Ref;
    type Owned = CombinedTest2Owned;
    type IsUnsized = <CombinedUnsized<List<u8>, CombinedUnsized<List<TestStruct>, CombinedTest>> as UnsizedType>::IsUnsized;

    unsafe fn from_bytes<S: AsBytes>(
        super_ref: S,
    ) -> anyhow::Result<FromBytesReturn<S, Self::RefData, Self::RefMeta>> {
        unsafe {
            Ok(
                <CombinedUnsized<List<u8>, CombinedUnsized<List<TestStruct>, CombinedTest>> as UnsizedType>::from_bytes(
                    super_ref,
                )?
                .map_ref(|_, r| CombinedTest2Ref(r))
                .map_meta(CombinedTest2Meta),
            )
        }
    }

    fn owned<S: AsBytes>(r: RefWrapper<S, Self::RefData>) -> anyhow::Result<Self::Owned> {
        let (list1, (list2, other)) = <CombinedUnsized<
            List<u8>,
            CombinedUnsized<List<TestStruct>, CombinedTest>,
        > as UnsizedType>::owned(unsafe {
            r.wrap_r(|_, r| r.0)
        })?;
        Ok(CombinedTest2Owned {
            list1,
            list2,
            other,
        })
    }
}
pub struct CombinedTest2Init<List1, List2, Other> {
    pub list1: List1,
    pub list2: List2,
    pub other: Other,
}
impl<List1, List2, Other> UnsizedInit<CombinedTest2Init<List1, List2, Other>> for CombinedTest2
where
    List<u8>: UnsizedInit<List1>,
    List<TestStruct>: UnsizedInit<List2>,
    CombinedTest: UnsizedInit<Other>,
{
    const INIT_BYTES: usize = <CombinedUnsized<
        List<u8>,
        CombinedUnsized<List<TestStruct>, CombinedTest>,
    > as UnsizedInit<(List1, (List2, Other))>>::INIT_BYTES;

    unsafe fn init<S: AsMutBytes>(
        super_ref: S,
        arg: CombinedTest2Init<List1, List2, Other>,
    ) -> anyhow::Result<(RefWrapper<S, Self::RefData>, Self::RefMeta)> {
        unsafe {
            let (r, m) = <CombinedUnsized<
                List<u8>,
                CombinedUnsized<List<TestStruct>, CombinedTest>,
            > as UnsizedInit<(List1, (List2, Other))>>::init(
                super_ref,
                (arg.list1, (arg.list2, arg.other)),
            )?;
            Ok((r.wrap_r(|_, r| CombinedTest2Ref(r)), CombinedTest2Meta(m)))
        }
    }
}
impl UnsizedInit<Zeroed> for CombinedTest2
where
    List<u8>: UnsizedInit<Zeroed>,
    List<TestStruct>: UnsizedInit<Zeroed>,
    CombinedTest: UnsizedInit<Zeroed>,
{
    const INIT_BYTES: usize = <CombinedUnsized<
        List<u8>,
        CombinedUnsized<List<TestStruct>, CombinedTest>,
    > as UnsizedInit<Zeroed>>::INIT_BYTES;

    unsafe fn init<S: AsMutBytes>(
        super_ref: S,
        arg: Zeroed,
    ) -> anyhow::Result<(RefWrapper<S, Self::RefData>, Self::RefMeta)> {
        unsafe {
            let (r, m) = <CombinedUnsized<
                List<u8>,
                CombinedUnsized<List<TestStruct>, CombinedTest>,
            > as UnsizedInit<Zeroed>>::init(super_ref, arg)?;
            Ok((r.wrap_r(|_, r| CombinedTest2Ref(r)), CombinedTest2Meta(m)))
        }
    }
}

unsafe impl<S> RefBytes<S> for CombinedTest2Ref
where
    S: AsBytes,
{
    fn bytes(wrapper: &RefWrapper<S, Self>) -> anyhow::Result<&[u8]> {
        wrapper.sup().as_bytes()
    }
}
unsafe impl<S> RefBytesMut<S> for CombinedTest2Ref
where
    S: AsMutBytes,
{
    fn bytes_mut(wrapper: &mut RefWrapper<S, Self>) -> anyhow::Result<&mut [u8]> {
        unsafe { wrapper.sup_mut().as_mut_bytes() }
    }
}
unsafe impl<S> RefResize<S, <CombinedUnsized<
    List<u8>,
    CombinedUnsized<List<TestStruct>, CombinedTest>,
> as UnsizedType>::RefMeta>
    for CombinedTest2Ref
where
    S: Resize<CombinedTest2Meta>,
{
    unsafe fn resize(
        wrapper: &mut RefWrapper<S, Self>,
        new_byte_len: usize,
        new_meta: <CombinedUnsized<
    List<u8>,
    CombinedUnsized<List<TestStruct>, CombinedTest>, > as UnsizedType>::RefMeta,
    ) -> anyhow::Result<()> {
        unsafe {
            wrapper.r_mut().0 = CombinedRef::new(new_meta);
            wrapper
                .sup_mut()
                .resize(new_byte_len, CombinedTest2Meta(new_meta))
        }
    }

    unsafe fn set_meta(
        wrapper: &mut RefWrapper<S, Self>,
        new_meta: <CombinedUnsized<
    List<u8>,
    CombinedUnsized<List<TestStruct>, CombinedTest>,
> as UnsizedType>::RefMeta,
    ) -> anyhow::Result<()> {
        unsafe {
            wrapper.r_mut().0 = CombinedRef::new(new_meta);
            wrapper.sup_mut().set_meta(CombinedTest2Meta(new_meta))
        }
    }
}

type List1<S> = RefWrapper<
    RefWrapper<
        RefWrapper<S, <CombinedUnsized<
            List<u8>,
            CombinedUnsized<List<TestStruct>, CombinedTest>,
        > as UnsizedType>::RefData>,
        CombinedTRef<List<u8>, CombinedUnsized<List<TestStruct>, CombinedTest>>,
    >,
    <List<u8> as UnsizedType>::RefData,
>;
type List2<S> = RefWrapper<
    RefWrapper<RefWrapper<RefWrapper<
        RefWrapper<S, <CombinedUnsized<
            List<u8>,
            CombinedUnsized<List<TestStruct>, CombinedTest>,
        > as UnsizedType>::RefData>,
        CombinedURef<List<u8>, CombinedUnsized<List<TestStruct>, CombinedTest>>,
    >, <CombinedUnsized<List<TestStruct>, CombinedTest> as UnsizedType>::RefData>, CombinedTRef<List<TestStruct>, CombinedTest>>,
    <List<TestStruct> as UnsizedType>::RefData,
>;
type Other<S> = RefWrapper<
    RefWrapper<RefWrapper<RefWrapper<
        RefWrapper<S, <CombinedUnsized<
            List<u8>,
            CombinedUnsized<List<TestStruct>, CombinedTest>,
        > as UnsizedType>::RefData>,
        CombinedURef<List<u8>, CombinedUnsized<List<TestStruct>, CombinedTest>>,
    >, <CombinedUnsized<List<TestStruct>, CombinedTest> as UnsizedType>::RefData>, CombinedURef<List<TestStruct>, CombinedTest>>,
    <CombinedTest as UnsizedType>::RefData,
>;

pub trait CombinedTest2Ext: Sized + RefWrapperTypes {
    fn list1(self) -> anyhow::Result<List1<Self>>;
    fn list2(self) -> anyhow::Result<List2<Self>>;
    fn other(self) -> anyhow::Result<Other<Self>>;
}
impl<R> CombinedTest2Ext for R
where
    R: RefWrapperTypes<Ref = CombinedTest2Ref> + AsBytes,
{
    fn list1(self) -> anyhow::Result<List1<Self>> {
        let r = self.r().0;
        unsafe { RefWrapper::new(self, r).t() }
    }

    fn list2(self) -> anyhow::Result<List2<Self>> {
        let r = self.r().0;
        unsafe { RefWrapper::new(self, r).u()?.t() }
    }

    fn other(self) -> anyhow::Result<Other<Self>> {
        let r = self.r().0;
        unsafe { RefWrapper::new(self, r).u()?.u() }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::UnsizedType;
    use crate::serialize::list::ListExt;
    use crate::serialize::ref_wrapper::RefWrapper;
    use crate::serialize::test::TestByteSet;
    use crate::serialize::unsize::init::Zeroed;
    use crate::serialize::unsize::test::CombinedTestExt;
    use crate::serialize::unsize::test2::{
        CombinedTest2, CombinedTest2Ext, CombinedTest2Meta, CombinedTest2Ref, TestStruct,
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

    #[test]
    fn test() -> anyhow::Result<()> {
        let mut bytes = TestByteSet::<CombinedTest2>::new(Zeroed)?;
        let mut r = bytes.mutable()?;
        assert_eq!(&**(&r).list1()?, &[] as &[u8]);
        assert_eq!(&**(&r).list2()?, &[]);
        assert_eq!(&**(&r).other()?.list1()?, &[] as &[u8]);
        assert_eq!(&**(&r).other()?.list2()?, &[]);
        cool(&mut r, 1)?;
        assert_eq!(&**(&r).list1()?, &[0]);
        assert_eq!(&**(&r).list2()?, &[TestStruct { val1: 1, val2: 0 }]);
        assert_eq!(&**(&r).other()?.list1()?, &[0]);
        assert_eq!(
            &**(&r).other()?.list2()?,
            &[TestStruct { val1: 1, val2: 0 }]
        );
        cool(&mut r, 2)?;
        let r = bytes.immut()?;
        assert_eq!(&**r.list1()?, &[0, 0]);
        assert_eq!(
            &**r.list2()?,
            &[
                TestStruct { val1: 2, val2: 0 },
                TestStruct { val1: 1, val2: 0 }
            ]
        );
        assert_eq!(&**r.other()?.list1()?, &[0, 0]);
        assert_eq!(
            &**r.other()?.list2()?,
            &[
                TestStruct { val1: 2, val2: 0 },
                TestStruct { val1: 1, val2: 0 }
            ]
        );
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
