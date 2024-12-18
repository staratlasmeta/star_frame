use crate::prelude::*;
use crate::unsize::tests::test::TestStruct;

#[unsized_type(skip_idl)]
#[repr(u8)]
pub enum TestEnum<A: UnsizedGenerics> {
    #[default_init]
    A,
    B(List<A>) = 4,
    C(CombinedTest),
}

#[unsized_type]
pub struct CombinedTest {
    #[unsized_start]
    pub list1: List<u8>,
    pub list2: List<TestStruct>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() -> anyhow::Result<()> {
        let mut bytes = TestByteSet::<TestEnum<u8>>::new(TestEnumInitA(()))?;
        assert_eq!(bytes.immut()?.discriminant(), TestEnumDiscriminant::A);
        {
            let mut mutable = bytes.mutable()?;
            {
                let b = (&mut mutable).set_b(Zeroed)?;
                assert_eq!(&**b, &[] as &[u8]);
            }
            assert_eq!(mutable.discriminant(), TestEnumDiscriminant::B);
            let mutable_b = match mutable.get()? {
                TestEnumRefWrapper::A(_) | TestEnumRefWrapper::C(_) => unreachable!(),
                TestEnumRefWrapper::B(r) => r,
            };
            assert_eq!(&**mutable_b, &[] as &[u8]);
        }
        match bytes.immut()?.get()? {
            TestEnumRefWrapper::A(_) | TestEnumRefWrapper::C(_) => unreachable!(),
            TestEnumRefWrapper::B(r) => assert_eq!(&**r, &[] as &[u8]),
        };
        {
            let mutable = bytes.mutable()?;
            assert_eq!(mutable.discriminant(), TestEnumDiscriminant::B);
            let mut mutable_b = match mutable.get()? {
                TestEnumRefWrapper::A(_) | TestEnumRefWrapper::C(_) => unreachable!(),
                TestEnumRefWrapper::B(r) => r,
            };
            mutable_b.push(0)?;
            assert_eq!(&**mutable_b, &[0]);
        }
        match bytes.immut()?.get()? {
            TestEnumRefWrapper::A(_) | TestEnumRefWrapper::C(_) => unreachable!(),
            TestEnumRefWrapper::B(r) => assert_eq!(&**r, &[0]),
        };
        {
            let mut mutable = bytes.mutable()?;
            {
                let c = (&mut mutable).set_c(Zeroed)?;
                assert_eq!(&**(&c).list1()?, &[] as &[u8]);
                assert_eq!(&**c.list2()?, &[]);
            }
            assert_eq!(mutable.discriminant(), TestEnumDiscriminant::C);
            let mut mutable_c = match mutable.get()? {
                TestEnumRefWrapper::A(_) | TestEnumRefWrapper::B(_) => unreachable!(),
                TestEnumRefWrapper::C(r) => r,
            };
            assert_eq!(&**(&mut mutable_c).list1()?, &[] as &[u8]);
            assert_eq!(&**mutable_c.list2()?, &[]);
        }
        match bytes.immut()?.get()? {
            TestEnumRefWrapper::A(_) | TestEnumRefWrapper::B(_) => unreachable!(),
            TestEnumRefWrapper::C(r) => {
                assert_eq!(&**(&r).list1()?, &[] as &[u8]);
                assert_eq!(&**(&r).list2()?, &[]);
            }
        };
        {
            let mutable = bytes.mutable()?;
            assert_eq!(mutable.discriminant(), TestEnumDiscriminant::C);
            let mut mutable_c = match mutable.get()? {
                TestEnumRefWrapper::A(_) | TestEnumRefWrapper::B(_) => unreachable!(),
                TestEnumRefWrapper::C(r) => r,
            };
            (&mut mutable_c).list1()?.push(0)?;
            (&mut mutable_c)
                .list2()?
                .insert(0, TestStruct { val1: 1, val2: 0 })?;
            assert_eq!(&**(&mutable_c).list1()?, &[0]);
            assert_eq!(&**(&mutable_c).list2()?, &[TestStruct { val1: 1, val2: 0 }]);
        }
        match bytes.immut()?.get()? {
            TestEnumRefWrapper::A(_) | TestEnumRefWrapper::B(_) => unreachable!(),
            TestEnumRefWrapper::C(r) => {
                assert_eq!(&**(&r).list1()?, &[0]);
                assert_eq!(&**(&r).list2()?, &[TestStruct { val1: 1, val2: 0 }]);
            }
        };

        Ok(())
    }
}
