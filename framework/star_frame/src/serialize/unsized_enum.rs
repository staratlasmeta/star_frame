use crate::align1::Align1;
use crate::packed_value::{PackedValue, PackedValueChecked};
use crate::serialize::combined_unsized::CombinedUnsized;
use crate::serialize::list::List;
use crate::serialize::pointer_breakup::{BuildPointer, BuildPointerMut, PointerBreakup};
use crate::serialize::unsized_type::UnsizedType;
use crate::serialize::{FrameworkFromBytes, FrameworkFromBytesMut, FrameworkSerialize};
use advance::Advance;
use bytemuck::{CheckedBitPattern, Pod, Zeroable};
use solana_program::program_error::ProgramError;
use star_frame::serialize::ResizeFn;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::{Deref, DerefMut};
use std::ptr;
use std::ptr::NonNull;

pub trait UnsizedEnum:
    'static
    + for<'a> UnsizedType<
        Ref<'a> = Self::EnumRefWrapper<'a>,
        RefMut<'a> = Self::EnumRefMutWrapper<'a>,
    >
{
    type Discriminant: 'static + Copy + Debug + CheckedBitPattern;
    type EnumRefWrapper<'a>: EnumRefWrapper;
    type EnumRefMutWrapper<'a>: EnumRefMutWrapper;
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

#[derive(Pod, Zeroable, Copy, Clone, Align1)]
#[repr(C, packed)]
pub struct TestStruct {
    val1: u32,
    val2: u64,
}

// #[derive(Copy, Clone)]
// #[repr(u8)]
// enum TestEnum {
//     // #[enum(PackedValue<u32>)]
//     A,
//     // #[enum(CombinedUnsized<TestStruct, List<u8, u8>>)]
//     B = 4,
//     // #[enum(List<PackedValue<u32>>)]
//     C,
// }

// ---------------------------- Generated ----------------------------

#[derive(Align1)]
#[allow(dead_code)]
pub struct TestEnum {
    discriminant: PackedValueChecked<TestEnumDiscriminant>,
    bytes: [u8],
}
impl TestEnum {
    pub fn discriminant(&self) -> TestEnumDiscriminant {
        self.discriminant.0
    }
}
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum TestEnumDiscriminant {
    A = 0,
    B = 4,
    C = 5,
}
unsafe impl CheckedBitPattern for TestEnumDiscriminant {
    type Bits = u8;

    fn is_valid_bit_pattern(bits: &Self::Bits) -> bool {
        matches!(bits, 0 | 4 | 5)
    }
}

impl UnsizedType for TestEnum {
    type RefMeta = TestEnumMeta;
    type Ref<'a> = TestEnumRefWrapper<'a>
        where Self: 'a,;
    type RefMut<'a> = TestEnumRefMutWrapper<'a>
        where Self: 'a,;
}

#[derive(Copy, Clone)]
enum TestEnumMetaInner {
    A(<PackedValue<u32> as UnsizedType>::RefMeta),
    B(<CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedType>::RefMeta),
    C(<List<PackedValue<u32>> as UnsizedType>::RefMeta),
}
#[derive(Copy, Clone)]
pub struct TestEnumMeta {
    byte_len: usize,
    inner: TestEnumMetaInner,
}

pub enum TestEnumRef<'a> {
    A(<PackedValue<u32> as UnsizedType>::Ref<'a>),
    B(<CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedType>::Ref<'a>),
    C(<List<PackedValue<u32>> as UnsizedType>::Ref<'a>),
}
pub struct TestEnumRefWrapper<'a> {
    phantom_ref: PhantomData<&'a ()>,
    ptr: NonNull<()>,
    meta: TestEnumMeta,
}
impl<'a> TestEnumRefWrapper<'a> {
    unsafe fn data_ptr(&self) -> NonNull<()> {
        NonNull::new_unchecked(
            self.ptr
                .as_ptr()
                .byte_add(size_of::<PackedValueChecked<TestEnumDiscriminant>>()),
        )
    }
}
impl Deref for TestEnumRefWrapper<'_> {
    type Target = TestEnum;

    fn deref(&self) -> &Self::Target {
        unsafe { &*ptr::from_raw_parts(self.ptr.as_ptr(), self.meta.byte_len) }
    }
}
impl FrameworkSerialize for TestEnumRefWrapper<'_> {
    fn to_bytes(&self, output: &mut &mut [u8]) -> crate::Result<()> {
        unsafe {
            match self.meta.inner {
                TestEnumMetaInner::A(meta) => {
                    0u8.to_bytes(output)?;
                    <<PackedValue<u32> as UnsizedType>::Ref<'_> as FrameworkSerialize>::to_bytes(
                        &<<PackedValue<u32> as UnsizedType>::Ref<'_> as BuildPointer>::build_pointer(
                            self.data_ptr(),
                            meta,
                        ),
                        output,
                    )
                }
                TestEnumMetaInner::B(meta) => {
                    4u8.to_bytes(output)?;
                    <<CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedType>::Ref<'_>
                        as FrameworkSerialize>::to_bytes(
                        &<<CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedType>::Ref<'_>
                            as BuildPointer>::build_pointer(self.data_ptr(), meta),
                        output,
                    )
                }
                TestEnumMetaInner::C(meta) => {
                    5u8.to_bytes(output)?;
                    <<List<PackedValue<u32>> as UnsizedType>::Ref<'_> as FrameworkSerialize>::
                        to_bytes(
                        &<<List<PackedValue<u32>> as UnsizedType>::Ref<'_> as BuildPointer>::
                            build_pointer(self.data_ptr(), meta),
                        output,
                    )
                }
            }
        }
    }
}
#[allow(clippy::unit_arg)]
unsafe impl<'a> FrameworkFromBytes<'a> for TestEnumRefWrapper<'a> {
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
                    <<PackedValue<u32> as UnsizedType>::Ref<'_> as FrameworkFromBytes>::from_bytes(
                        bytes,
                    )?;
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
impl PointerBreakup for TestEnumRefWrapper<'_> {
    type Metadata = TestEnumMeta;

    #[allow(clippy::unit_arg)]
    fn break_pointer(&self) -> (NonNull<()>, Self::Metadata) {
        (self.ptr, self.meta)
    }
}
impl BuildPointer for TestEnumRefWrapper<'_> {
    unsafe fn build_pointer(pointee: NonNull<()>, metadata: Self::Metadata) -> Self {
        Self {
            ptr: pointee,
            meta: metadata,
            phantom_ref: PhantomData,
        }
    }
}

pub enum TestEnumRefMut<'a> {
    A(<PackedValue<u32> as UnsizedType>::RefMut<'a>),
    B(<CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedType>::RefMut<'a>),
    C(<List<PackedValue<u32>> as UnsizedType>::RefMut<'a>),
}
pub struct TestEnumRefMutWrapper<'a> {
    ptr: NonNull<()>,
    meta: TestEnumMeta,
    resize: Box<dyn ResizeFn<'a, TestEnumMeta>>,
}
impl<'a> TestEnumRefMutWrapper<'a> {
    unsafe fn data_ptr(&self) -> NonNull<()> {
        NonNull::new_unchecked(
            self.ptr
                .as_ptr()
                .byte_add(size_of::<PackedValueChecked<TestEnumDiscriminant>>()),
        )
    }
}
impl Deref for TestEnumRefMutWrapper<'_> {
    type Target = TestEnum;

    fn deref(&self) -> &Self::Target {
        unsafe { &*ptr::from_raw_parts(self.ptr.as_ptr(), self.meta.byte_len) }
    }
}
impl DerefMut for TestEnumRefMutWrapper<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *ptr::from_raw_parts_mut(self.ptr.as_ptr(), self.meta.byte_len) }
    }
}
impl FrameworkSerialize for TestEnumRefMutWrapper<'_> {
    fn to_bytes(&self, output: &mut &mut [u8]) -> crate::Result<()> {
        unsafe {
            let data_ptr = self.data_ptr();
            match self.meta.inner {
                TestEnumMetaInner::A(a) => {
                    0u8.to_bytes(output)?;
                    <<PackedValue<u32> as UnsizedType>::Ref<'_> as FrameworkSerialize>::to_bytes(
                        &<<PackedValue<u32> as UnsizedType>::Ref<'_> as BuildPointer>::build_pointer(
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
impl PointerBreakup for TestEnumRefMutWrapper<'_> {
    type Metadata = TestEnumMeta;

    #[allow(clippy::unit_arg)]
    fn break_pointer(&self) -> (NonNull<()>, Self::Metadata) {
        (self.ptr, self.meta)
    }
}
#[allow(clippy::unit_arg)]
unsafe impl<'a> FrameworkFromBytesMut<'a> for TestEnumRefMutWrapper<'a> {
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
                let sub_ptr = <<PackedValue<u32> as UnsizedType>::RefMut<'_> as FrameworkFromBytesMut>::from_bytes_mut(bytes, |_, _| panic!("Cannot resize during `from_bytes`"))?;
                Ok(Self {
                    ptr,
                    meta: TestEnumMeta {
                        inner: TestEnumMetaInner::A(sub_ptr.break_pointer().1),
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
impl<'a> BuildPointerMut<'a> for TestEnumRefMutWrapper<'a> {
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

impl<'a> TestEnumRefWrapper<'a> {
    pub fn value<'b>(&'b self) -> TestEnumRef<'b> {
        unsafe {
            let data_ptr = self.data_ptr();

            match self.meta.inner {
                TestEnumMetaInner::A(meta) => TestEnumRef::A(<<PackedValue<u32> as UnsizedType>::Ref<
                    'b,
                > as BuildPointer>::build_pointer(
                    data_ptr, meta
                )),
                TestEnumMetaInner::B(meta) => TestEnumRef::B(
                    <<CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedType>::Ref<'b>
                        as BuildPointer>::build_pointer(data_ptr, meta),
                ),
                TestEnumMetaInner::C(meta) => TestEnumRef::C(
                    <<List<PackedValue<u32>> as UnsizedType>::Ref<'b>
                        as BuildPointer>::build_pointer(data_ptr, meta),
                ),
            }
        }
    }
}
impl<'a> TestEnumRefMutWrapper<'a> {
    pub fn value<'b>(&'b self) -> TestEnumRef<'b> {
        unsafe {
            let data_ptr = self.data_ptr();

            match self.meta.inner {
                TestEnumMetaInner::A(meta) => TestEnumRef::A(<<PackedValue<u32> as UnsizedType>::Ref<
                    'b,
                > as BuildPointer>::build_pointer(
                    data_ptr, meta
                )),
                TestEnumMetaInner::B(meta) => TestEnumRef::B(
                    <<CombinedUnsized<TestStruct, List<u8, u8>> as UnsizedType>::Ref<'b>
                        as BuildPointer>::build_pointer(data_ptr, meta),
                ),
                TestEnumMetaInner::C(meta) => TestEnumRef::C(
                    <<List<PackedValue<u32>> as UnsizedType>::Ref<'b>
                        as BuildPointer>::build_pointer(data_ptr, meta),
                ),
            }
        }
    }

    pub fn value_mut<'b>(&'b mut self) -> TestEnumRefMut<'b> {
        unsafe {
            let data_ptr = self.data_ptr();
            let Self { ptr, meta, resize } = self;
            match meta.inner {
                TestEnumMetaInner::A(inner_meta) => TestEnumRefMut::A(
                    <<PackedValue<u32> as UnsizedType>::RefMut<'b> as BuildPointerMut>::
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
