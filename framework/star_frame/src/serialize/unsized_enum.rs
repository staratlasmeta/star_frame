pub use star_frame_proc::unsized_enum;

use crate::serialize::unsized_type::UnsizedType;
use bytemuck::CheckedBitPattern;
use std::fmt::Debug;

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
    use crate::packed_value::PackedValue;
    use crate::serialize::combined_unsized::CombinedUnsized;
    use crate::serialize::list::List;
    use crate::serialize::test::TestByteSet;
    use crate::serialize::unsized_enum::{EnumRefMutWrapper, EnumRefWrapper};
    use crate::serialize::unsized_type::UnsizedType;
    use bytemuck::{Pod, Zeroable};
    use star_frame_proc::{unsized_enum, Align1};
    use std::ops::Deref;

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

    #[unsized_enum]
    enum TestEnum2 {
        #[variant_type(u8)]
        A,
        #[variant_type(CombinedUnsized<TestStruct, List<u8, u8>>)]
        B = 4,
        #[variant_type(List<PackedValue<u32>>)]
        C,
    }

    #[test]
    fn test_enum() -> crate::Result<()> {
        type EnumToTest = TestEnum<TestStruct>;
        let mut test_byte_set = TestByteSet::<EnumToTest>::new((
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

        test_byte_set.mutable()?.set_c(())?;

        match test_byte_set.immut()?.value() {
            TestEnumRef::C(val) => {
                assert_eq!(val.deref().deref(), &[]);
            }
            x => panic!("Invalid variant: {:?}", x),
        }

        let mut mutable = test_byte_set.mutable()?;

        match mutable.value_mut() {
            TestEnumRefMut::C(mut val) => {
                val.push(PackedValue(1))?;
                val.push(PackedValue(2))?;
                val.push(PackedValue(3))?;

                assert_eq!(
                    val.deref().deref(),
                    &[PackedValue(1), PackedValue(2), PackedValue(3)]
                );
            }
            x => panic!("Invalid variant: {:?}", x),
        }

        match mutable.value_mut() {
            TestEnumRefMut::C(val) => {
                assert_eq!(
                    val.deref().deref(),
                    &[PackedValue(1), PackedValue(2), PackedValue(3)]
                );
            }
            x => panic!("Invalid variant: {:?}", x),
        }

        drop(mutable);

        match test_byte_set.immut()?.value() {
            TestEnumRef::C(val) => {
                assert_eq!(
                    val.deref().deref(),
                    &[PackedValue(1), PackedValue(2), PackedValue(3)]
                );
            }
            x => panic!("Invalid variant: {:?}", x),
        }

        Ok(())
    }
}
