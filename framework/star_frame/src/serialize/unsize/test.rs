use crate::prelude::{
    CombinedExt, CombinedRef, CombinedTRef, CombinedURef, CombinedUnsized, List, UnsizedInit,
    UnsizedType,
};
use crate::serialize::list::ListExt;
use crate::serialize::ref_wrapper::{
    AsBytes, AsMutBytes, RefBytes, RefBytesMut, RefResize, RefWrapper, RefWrapperMutExt,
    RefWrapperTypes,
};
use crate::serialize::unsize::resize::Resize;
use crate::serialize::unsize::FromBytesReturn;
use bytemuck::{Pod, Zeroable};
use star_frame_proc::Align1;

#[derive(Debug, Copy, Clone, Pod, Zeroable, Align1, PartialEq, Eq)]
#[repr(C, packed)]
pub struct TestStruct {
    pub val1: u32,
    pub val2: u64,
}

// #[unsized_type]
// pub struct CombinedTest {
//     pub list1: List<u8>,
//     pub list2: List<TestStruct>,
// }

#[derive(Debug, Align1)]
#[repr(transparent)]
pub struct CombinedTest(CombinedUnsized<List<u8>, List<TestStruct>>);
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct CombinedTestMeta(<CombinedUnsized<List<u8>, List<TestStruct>> as UnsizedType>::RefMeta);
// TODO: Where clause for derives?
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct CombinedTestRef(<CombinedUnsized<List<u8>, List<TestStruct>> as UnsizedType>::RefData);
pub struct CombinedTestOwned {
    pub list1: <List<u8> as UnsizedType>::Owned,
    pub list2: <List<TestStruct> as UnsizedType>::Owned,
}

pub type CombinedTestRefWrapper<S> = RefWrapper<S, CombinedTestRef>;

unsafe impl UnsizedType for CombinedTest {
    type RefMeta = CombinedTestMeta;
    type RefData = CombinedTestRef;
    type Owned = CombinedTestOwned;
    type IsUnsized = <CombinedUnsized<List<u8>, List<TestStruct>> as UnsizedType>::IsUnsized;

    unsafe fn from_bytes<S: AsBytes>(
        super_ref: S,
    ) -> anyhow::Result<FromBytesReturn<S, Self::RefData, Self::RefMeta>> {
        unsafe {
            Ok(
                <CombinedUnsized<List<u8>, List<TestStruct>> as UnsizedType>::from_bytes(
                    super_ref,
                )?
                .map_ref(|_, r| CombinedTestRef(r))
                .map_meta(CombinedTestMeta),
            )
        }
    }

    fn owned<S: AsBytes>(r: RefWrapper<S, Self::RefData>) -> anyhow::Result<Self::Owned> {
        let (list1, list2) =
            CombinedUnsized::<List<u8>, List<TestStruct>>::owned(unsafe { r.wrap_r(|_, r| r.0) })?;
        Ok(CombinedTestOwned { list1, list2 })
    }
}
pub struct CombinedTestInit<List1, List2> {
    pub list1: List1,
    pub list2: List2,
}
impl<List1, List2> UnsizedInit<CombinedTestInit<List1, List2>> for CombinedTest
where
    List<u8>: UnsizedInit<List1>,
    List<TestStruct>: UnsizedInit<List2>,
{
    const INIT_BYTES: usize =
        <CombinedUnsized<List<u8>, List<TestStruct>> as UnsizedInit<(List1, List2)>>::INIT_BYTES;

    unsafe fn init<S: AsMutBytes>(
        super_ref: S,
        arg: CombinedTestInit<List1, List2>,
    ) -> anyhow::Result<(RefWrapper<S, Self::RefData>, Self::RefMeta)> {
        unsafe {
            let (r, m) = CombinedUnsized::<List<u8>, List<TestStruct>>::init(
                super_ref,
                (arg.list1, arg.list2),
            )?;
            Ok((r.wrap_r(|_, r| CombinedTestRef(r)), CombinedTestMeta(m)))
        }
    }
}
impl UnsizedInit<()> for CombinedTest
where
    List<u8>: UnsizedInit<()>,
    List<TestStruct>: UnsizedInit<()>,
{
    const INIT_BYTES: usize =
        <CombinedUnsized<List<u8>, List<TestStruct>> as UnsizedInit<()>>::INIT_BYTES;

    unsafe fn init<S: AsMutBytes>(
        super_ref: S,
        arg: (),
    ) -> anyhow::Result<(RefWrapper<S, Self::RefData>, Self::RefMeta)> {
        unsafe {
            let (r, m) = CombinedUnsized::<List<u8>, List<TestStruct>>::init(super_ref, arg)?;
            Ok((r.wrap_r(|_, r| CombinedTestRef(r)), CombinedTestMeta(m)))
        }
    }
}

unsafe impl<S> RefBytes<S> for CombinedTestRef
where
    S: AsBytes,
{
    fn bytes(wrapper: &RefWrapper<S, Self>) -> anyhow::Result<&[u8]> {
        wrapper.sup().as_bytes()
    }
}
unsafe impl<S> RefBytesMut<S> for CombinedTestRef
where
    S: AsMutBytes,
{
    fn bytes_mut(wrapper: &mut RefWrapper<S, Self>) -> anyhow::Result<&mut [u8]> {
        unsafe { wrapper.sup_mut().as_mut_bytes() }
    }
}
unsafe impl<S, M> RefResize<S, M> for CombinedTestRef
where
    S: AsMutBytes,
    S: Resize<M>,
{
    unsafe fn resize(
        wrapper: &mut RefWrapper<S, Self>,
        new_byte_len: usize,
        new_meta: M,
    ) -> anyhow::Result<()> {
        wrapper.sup_mut().resize(new_byte_len, new_meta)
    }

    unsafe fn set_meta(wrapper: &mut RefWrapper<S, Self>, new_meta: M) -> anyhow::Result<()> {
        wrapper.sup_mut().set_meta(new_meta)
    }
}

type List1<S> = RefWrapper<
    RefWrapper<
        RefWrapper<S, <CombinedUnsized<List<u8>, List<TestStruct>> as UnsizedType>::RefData>,
        CombinedTRef<List<u8>, List<TestStruct>>,
    >,
    <List<u8> as UnsizedType>::RefData,
>;
type List2<S> = RefWrapper<
    RefWrapper<
        RefWrapper<S, <CombinedUnsized<List<u8>, List<TestStruct>> as UnsizedType>::RefData>,
        CombinedURef<List<u8>, List<TestStruct>>,
    >,
    <List<TestStruct> as UnsizedType>::RefData,
>;

pub trait CombinedTestExt: Sized + RefWrapperTypes {
    fn list1(self) -> anyhow::Result<List1<Self>>;
    fn list2(self) -> anyhow::Result<List2<Self>>;
}
impl<R> CombinedTestExt for R
where
    R: RefWrapperTypes<Ref = CombinedTestRef> + AsBytes,
{
    fn list1(self) -> anyhow::Result<List1<Self>> {
        let r = self.r().0;
        unsafe { RefWrapper::new(self, r).t() }
    }

    fn list2(self) -> anyhow::Result<List2<Self>> {
        let r = self.r().0;
        unsafe { RefWrapper::new(self, r).u() }
    }
}

#[cfg(test)]
mod tests {
    use crate::serialize::list::ListExt;
    use crate::serialize::ref_wrapper::{AsMutBytes, RefWrapper};
    use crate::serialize::test::TestByteSet;
    use crate::serialize::unsize::test::{
        CombinedTest, CombinedTestExt, CombinedTestRef, TestStruct,
    };

    fn cool(r: &mut RefWrapper<impl AsMutBytes, CombinedTestRef>, val: u32) -> anyhow::Result<()> {
        r.list1()?.push(0)?;
        r.list2()?.insert(0, TestStruct { val1: val, val2: 0 })?;
        Ok(())
    }

    #[test]
    fn test() -> anyhow::Result<()> {
        let mut bytes = TestByteSet::<CombinedTest>::new(())?;
        let mut r = bytes.mutable()?;
        cool(&mut r, 1)?;
        assert_eq!(&**(&r).list1()?, &[0]);
        assert_eq!(&**(&r).list2()?, &[TestStruct { val1: 1, val2: 0 }]);
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
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::serialize::list::ListExt;
    #[test]
    fn test_stuff() -> anyhow::Result<()> {
        let bytes = vec![0u8; 100];
        let mut combined: CombinedTestRefWrapper<_> =
            unsafe { CombinedTest::from_bytes(bytes).unwrap() }.ref_wrapper;
        println!("{:?}", combined);
        let mut list = combined.list1().unwrap();
        list.push(1)?;
        list.insert(0, 2)?;
        println!("{:?}", list.len());
        println!("{:?}", list.as_slice());
        Ok(())
    }
}
