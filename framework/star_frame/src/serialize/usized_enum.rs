use crate::align1::Align1;
use crate::packed_value::PackedValue;
use crate::serialize::combined_unsized::CombinedUnsized;
use crate::serialize::list::List;
use crate::serialize::pointer_breakup::{BuildPointer, PointerBreakup};
use crate::serialize::serialize_with::SerializeWith;
use crate::serialize::{FrameworkFromBytes, FrameworkSerialize};
use advance::Advance;
use bytemuck::{Pod, Zeroable};
use solana_program::program_error::ProgramError;
use star_frame::serialize::ResizeFn;
use std::mem::size_of;
use std::ops::Deref;
use std::ptr;

pub trait EnumRepr {
    type Repr;

    fn to_repr(&self) -> Self::Repr;
    fn from_repr(repr: Self::Repr) -> Self;
}
pub trait UnsizedEnumType: EnumRepr {
    type RefMeta: 'static + Copy;
    type Ref<'a>;
    type RefMut<'a>;
}

#[derive(Align1)]
pub struct UnsizedEnum<E>
where
    E: EnumRepr,
    E::Repr: Pod,
{
    repr: PackedValue<E::Repr>,
    bytes: [u8],
}

pub struct UnsizedEnumMeta<M> {
    byte_length: usize,
    sub_meta: M,
}

pub struct UnsizedEnumRef<'a, E>
where
    E: UnsizedEnumType,
    E::Repr: Pod,
{
    pub ptr: *const (),
    pub bytes_len: usize,
    pub r: E::Ref<'a>,
}

pub struct UnsizedEnumRefMut<'a, E>
where
    E: UnsizedEnumType,
    E::Repr: Pod,
{
    pub ptr: *mut (),
    pub bytes_len: usize,
    pub r: E::RefMut<'a>,
    pub resize: Box<dyn ResizeFn<'a, E::RefMeta>>,
}

// ---------------------------- Test Stuff ----------------------------

#[derive(Pod, Zeroable, Copy, Clone, Align1)]
#[repr(C, packed)]
struct TestStruct {
    val1: u32,
    val2: u64,
}

#[derive(Copy, Clone)]
#[repr(u8)]
enum TestEnum {
    // #[enum(PackedValue<u32>)]
    A,
    // #[enum(CombinedUnsized<TestStruct, List<u8, u8>>)]
    B = 4,
    // #[enum(List<u32>)]
    C,
}
impl EnumRepr for TestEnum {
    type Repr = u8;

    fn to_repr(&self) -> Self::Repr {
        *self as u8
    }

    fn from_repr(repr: Self::Repr) -> Self {
        match repr {
            0 => Self::A,
            4 => Self::B,
            5 => Self::C,
            _ => panic!("Invalid enum repr"),
        }
    }
}
impl UnsizedEnumType for TestEnum {
    type RefMeta = TestEnumMeta;
    type Ref<'a> = TestEnumRef<'a>;
    type RefMut<'a> = TestEnumRefMut<'a>;
}
impl SerializeWith for UnsizedEnum<TestEnum> {
    type RefMeta = TestEnumMeta;
    type Ref<'a> = UnsizedEnumRef<'a, TestEnum>
        where Self: 'a,;
    type RefMut<'a> = UnsizedEnumRefMut<'a, TestEnum>
        where Self: 'a,;
}

#[derive(Copy, Clone)]
enum TestEnumMeta {
    A(<PackedValue<u32> as SerializeWith>::RefMeta),
    B(<CombinedUnsized<TestStruct, List<u8, u8>> as SerializeWith>::RefMeta),
    C(<List<u32> as SerializeWith>::RefMeta),
}

enum TestEnumRef<'a> {
    A(<PackedValue<u32> as SerializeWith>::Ref<'a>),
    B(<CombinedUnsized<TestStruct, List<u8, u8>> as SerializeWith>::Ref<'a>),
    C(<List<u32> as SerializeWith>::Ref<'a>),
}
impl Deref for UnsizedEnumRef<'_, TestEnum> {
    type Target = UnsizedEnum<TestEnum>;

    fn deref(&self) -> &Self::Target {
        unsafe { &*ptr::from_raw_parts(self.ptr, self.bytes_len) }
    }
}
impl FrameworkSerialize for UnsizedEnumRef<'_, TestEnum> {
    fn to_bytes(&self, output: &mut &mut [u8]) -> crate::Result<()> {
        match &self.r {
            TestEnumRef::A(a) => {
                0u8.to_bytes(output)?;
                a.to_bytes(output)
            }
            TestEnumRef::B(b) => {
                4u8.to_bytes(output)?;
                b.to_bytes(output)
            }
            TestEnumRef::C(c) => {
                5u8.to_bytes(output)?;
                c.to_bytes(output)
            }
        }
    }
}
unsafe impl<'a> FrameworkFromBytes<'a> for UnsizedEnumRef<'a, TestEnum> {
    fn from_bytes(bytes: &mut &'a [u8]) -> crate::Result<Self> {
        let discriminant = <&PackedValue<<TestEnum as EnumRepr>::Repr>>::from_bytes(&mut &**bytes)?;
        let ptr = bytes
            .try_advance(size_of::<<TestEnum as EnumRepr>::Repr>())?
            .as_ptr()
            .cast();
        match discriminant.0 {
            0 => Ok(Self {
                ptr, r: TestEnumRef::A(
                <<PackedValue<u32> as SerializeWith>::Ref<'a> as FrameworkFromBytes>::from_bytes(
                    bytes,
                )?,
            ) }),
            4 => Ok(Self {
                ptr, r:TestEnumRef::B(
                <<CombinedUnsized<TestStruct, List<u8, u8>> as SerializeWith>::Ref<'a>
                    as FrameworkFromBytes>::from_bytes(bytes)?,
            )}),
            5 => Ok(Self {
                ptr, r: TestEnumRef::C(
                <<List<u32> as SerializeWith>::Ref<'a> as FrameworkFromBytes>::from_bytes(bytes)?,
            )}),
            _ => Err(ProgramError::InvalidAccountData),
        }
    }
}
impl PointerBreakup for UnsizedEnumRef<'_, TestEnum> {
    type Metadata = TestEnumMeta;

    fn break_pointer(&self) -> (*const (), Self::Metadata) {
        (
            self.ptr,
            match &self.r {
                TestEnumRef::A(a) => TestEnumMeta::A(a.break_pointer().1),
                TestEnumRef::B(b) => TestEnumMeta::B(b.break_pointer().1),
                TestEnumRef::C(c) => TestEnumMeta::C(c.break_pointer().1),
            },
        )
    }
}
impl BuildPointer for UnsizedEnumRef<'_, TestEnum> {
    unsafe fn build_pointer(pointee: *const (), metadata: Self::Metadata) -> Self {
        let r_ptr =
            unsafe { pointee.byte_add(size_of::<PackedValue<<TestEnum as EnumRepr>::Repr>>()) };
        Self {
            ptr: pointee,
            r: match metadata {
                TestEnumMeta::A(meta) => TestEnumRef::A(<<PackedValue<u32> as SerializeWith>::Ref<
                    '_,
                > as BuildPointer>::build_pointer(
                    r_ptr, meta
                )),
                TestEnumMeta::B(meta) => TestEnumRef::B(
                    <<CombinedUnsized<TestStruct, List<u8, u8>> as SerializeWith>::Ref<'_>
                        as BuildPointer>::build_pointer(
                        r_ptr, meta
                    )
                ),
                TestEnumMeta::C(meta) => TestEnumRef::C(
                    <<List<u32> as SerializeWith>::Ref<'_> as BuildPointer>::build_pointer(
                        r_ptr, meta
                    )
                ),
            },
        }
    }
}

enum TestEnumRefMut<'a> {
    A(<PackedValue<u32> as SerializeWith>::RefMut<'a>),
    B(<CombinedUnsized<TestStruct, List<u8, u8>> as SerializeWith>::RefMut<'a>),
    C(<List<u32> as SerializeWith>::RefMut<'a>),
}
impl Deref for UnsizedEnumRefMut<'_, TestEnum> {
    type Target = UnsizedEnum<TestEnum>;

    fn deref(&self) -> &Self::Target {
        todo!()
    }
}
