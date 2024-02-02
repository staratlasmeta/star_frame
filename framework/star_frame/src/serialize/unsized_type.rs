pub use star_frame_proc::unsized_type;

use crate::align1::Align1;
use crate::serialize::pointer_breakup::{BuildPointer, BuildPointerMut};
use crate::serialize::{FrameworkFromBytes, FrameworkFromBytesMut};
use bytemuck::Pod;
use std::ops::{Deref, DerefMut};

pub trait UnsizedType: 'static + Align1 {
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

#[cfg(test)]
mod test {
    use crate::align1::Align1;
    use crate::packed_value::PackedValue;
    use crate::serialize::list::List;
    use crate::serialize::test::TestByteSet;
    use crate::Result;
    use bytemuck::Pod;
    use star_frame_proc::unsized_type;
    use std::fmt::{Debug, Formatter};

    #[unsized_type]
    #[derive(Align1)]
    #[repr(C, packed)]
    pub struct TestUnsized<T> {
        pub(super) val1: T,
        pub val2: u64,
        pub list: List<u8>,
    }
    impl<T> Debug for TestUnsized<T>
    where
        T: Debug + Pod,
    {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("TestUnsized")
                .field("val1", &{ self.val1 })
                .field("val2", &{ self.val2 })
                // .field("list", &self.list) // For some reason this takes a dyn ref rather than impl ref
                .finish()
        }
    }

    #[unsized_type]
    #[derive(Align1, Debug)]
    #[repr(C)]
    pub struct TestUnsized2<U> {
        pub(super) val1: u8,
        pub val2: PackedValue<u64>,
        inner: U,
    }

    #[test]
    fn test_unsized_type() -> Result<()> {
        let mut test_bytes = TestByteSet::<TestUnsized<PackedValue<u16>>>::new(())?;
        {
            let immut = test_bytes.immut()?;
            assert_eq!({ immut.val1.0 }, 0);
            assert_eq!({ immut.val2 }, 0);
            assert!(immut.list.is_empty());
        }
        {
            let mut mutable = test_bytes.mutable()?;
            mutable.val1.0 = 100;
            mutable.val2 = 200;
            mutable.list_mut().push(1)?;

            assert_eq!({ mutable.val1.0 }, 100);
            assert_eq!({ mutable.val2 }, 200);
            assert_eq!(mutable.list, &[1]);
        }
        {
            let immut = test_bytes.immut()?;
            assert_eq!({ immut.val1.0 }, 100);
            assert_eq!({ immut.val2 }, 200);
            assert_eq!(immut.list, &[1]);
        }
        {
            let mut mutable = test_bytes.mutable()?;
            mutable.val1.0 += 100;
            mutable.val2 += 200;
            mutable.list_mut().push_all(2..=100)?;

            assert_eq!({ mutable.val1.0 }, 200);
            assert_eq!({ mutable.val2 }, 400);
            assert_eq!(mutable.list, (1..=100).collect::<Vec<_>>().as_slice());
        }
        Ok(())
    }
}
