//! Combining multiple [`UnsizedData`]s

use crate::align1::Align1;
use crate::util::MaybeMutRef;
use crate::versioned_account::context::{
    AccountDataContext, AccountDataMutContext, AccountDataRefContext,
};
use crate::versioned_account::unsized_data::UnsizedData;
use crate::Advance;
use common_utils::util::MaybeRef;
use derivative::Derivative;
use solana_program::program_memory::sol_memmove;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::DerefMut;
use std::ptr;
use std::ptr::{metadata, NonNull, Pointee};

/// Combined unsized data, places `T` before `U`.
#[repr(transparent)]
#[derive(Debug, Align1)]
pub struct CombinedUnsizedData<T: ?Sized, U: ?Sized> {
    phantom_t: PhantomData<T>,
    phantom_u: PhantomData<U>,
    bytes: [u8],
}
/// Extension trait for immutable [`CombinedUnsizedData`].
pub trait CombinedAccountDataContext<T: ?Sized + UnsizedData, U: ?Sized + UnsizedData>:
    AccountDataContext<CombinedUnsizedData<T, U>>
{
    /// Gets the `T` context immutably.
    fn t(&self) -> AccountDataRefContext<T> {
        // Safety: Pointer verified during construction.
        unsafe {
            self.sub_context(|this, meta| {
                (
                    &*ptr::from_raw_parts(this.bytes.as_ptr().cast(), meta.t_ptr_meta),
                    MaybeRef::Ref(&meta.t_meta),
                )
            })
        }
    }

    /// Gets the `U` context immutably.
    fn u(&self) -> AccountDataRefContext<U> {
        // Safety: Pointer verified during construction.
        unsafe {
            self.sub_context(|this, meta| {
                (
                    &*ptr::from_raw_parts(
                        this.bytes.as_ptr().add(meta.t_length).cast(),
                        meta.u_ptr_meta,
                    ),
                    MaybeRef::Ref(&meta.u_meta),
                )
            })
        }
    }

    /// Splits the context into `T` and `U` contexts immutably.
    fn split(&self) -> (AccountDataRefContext<T>, AccountDataRefContext<U>) {
        // Safety: Pointers verified during construction.
        unsafe {
            self.split_context(|this, meta| {
                (
                    (
                        &*ptr::from_raw_parts(this.bytes.as_ptr().cast(), meta.t_ptr_meta),
                        MaybeRef::Ref(&meta.t_meta),
                    ),
                    (
                        &*ptr::from_raw_parts(
                            this.bytes.as_ptr().add(meta.t_length).cast(),
                            meta.u_ptr_meta,
                        ),
                        MaybeRef::Ref(&meta.u_meta),
                    ),
                )
            })
        }
    }
}
impl<S, T: ?Sized + UnsizedData, U: ?Sized + UnsizedData> CombinedAccountDataContext<T, U> for S where
    S: AccountDataContext<CombinedUnsizedData<T, U>>
{
}
/// Extension trait for mutable [`CombinedUnsizedData`].
pub trait CombinedAccountDataMutContext<T: ?Sized + UnsizedData, U: ?Sized + UnsizedData>:
    CombinedAccountDataContext<T, U> + DerefMut
{
    /// Gets the `T` context mutably.
    fn t_mut(&mut self) -> AccountDataMutContext<T>;
    /// Gets the `U` context mutably.
    fn u_mut(&mut self) -> AccountDataMutContext<U>;
    /// Gets both the `T` mutably and the `U` as a mutable context.
    fn split_mut(&mut self) -> (&mut T, AccountDataMutContext<U>);
}
impl<'a, T: ?Sized + UnsizedData, U: ?Sized + UnsizedData> CombinedAccountDataMutContext<T, U>
    for AccountDataMutContext<'a, CombinedUnsizedData<T, U>>
{
    fn t_mut(&mut self) -> AccountDataMutContext<T> {
        // Safety: Pointer verified during construction.
        unsafe {
            self.sub_context_mut(move |args| {
                let bytes = &mut args.data.as_mut().bytes;
                let old_length = bytes.len();
                let CombinedUnsizedDataMeta {
                    t_ptr_meta,
                    t_meta,
                    t_length,
                    ..
                } = args.data_meta;
                (
                    &mut *ptr::from_raw_parts_mut(bytes.as_mut_ptr().cast(), *t_ptr_meta),
                    MaybeMutRef::Mut(t_meta),
                    Box::new(move |new_t_length, new_t_ptr_meta| {
                        let old_t_length = *t_length;
                        *t_length = new_t_length;
                        let new_length = old_length - old_t_length + new_t_length;
                        *args.data = NonNull::from_raw_parts(args.data.cast(), new_length);
                        *t_ptr_meta = new_t_ptr_meta;
                        (args.set_length)(new_length, new_length)?;
                        sol_memmove(
                            bytes.as_mut_ptr().add(new_t_length),
                            bytes.as_mut_ptr().add(old_t_length),
                            old_length - old_t_length,
                        );
                        Ok(())
                    }),
                )
            })
        }
    }

    fn u_mut(&mut self) -> AccountDataMutContext<U> {
        // Safety: Pointer verified during construction.
        unsafe {
            self.sub_context_mut(move |args| {
                let bytes = &mut args.data.as_mut().bytes;
                let old_length = bytes.len();
                let CombinedUnsizedDataMeta {
                    u_ptr_meta,
                    u_meta,
                    t_length,
                    ..
                } = args.data_meta;
                (
                    &mut *ptr::from_raw_parts_mut(
                        bytes.as_mut_ptr().add(*t_length).cast(),
                        *u_ptr_meta,
                    ),
                    MaybeMutRef::Mut(u_meta),
                    Box::new(move |new_u_length, new_u_ptr_meta| {
                        let old_u_length = old_length - *t_length;
                        let new_length = old_length - old_u_length + new_u_length;
                        *args.data = NonNull::from_raw_parts(args.data.cast(), new_length);
                        *u_ptr_meta = new_u_ptr_meta;
                        (args.set_length)(new_length, new_length)?;
                        Ok(())
                    }),
                )
            })
        }
    }

    fn split_mut(&mut self) -> (&mut T, AccountDataMutContext<U>) {
        // Safety: Pointers verified during construction.
        unsafe {
            self.split_context_mut(|args| {
                let bytes = &mut args.data.as_mut().bytes;
                let old_length = bytes.len();
                let CombinedUnsizedDataMeta {
                    t_ptr_meta,
                    u_ptr_meta,
                    u_meta,
                    t_length,
                    ..
                } = args.data_meta;
                (
                    &mut *ptr::from_raw_parts_mut(bytes.as_mut_ptr().cast(), *t_ptr_meta),
                    &mut *ptr::from_raw_parts_mut(
                        bytes.as_mut_ptr().add(*t_length).cast(),
                        *u_ptr_meta,
                    ),
                    MaybeMutRef::Mut(u_meta),
                    Box::new(move |new_u_length, new_u_ptr_meta| {
                        let old_u_length = old_length - *t_length;
                        let new_length = old_length - old_u_length + new_u_length;
                        *args.data = NonNull::from_raw_parts(args.data.cast(), new_length);
                        *u_ptr_meta = new_u_ptr_meta;
                        (args.set_length)(new_length, new_length)?;
                        Ok(())
                    }),
                )
            })
        }
    }
}

/// The [`UnsizedData::Metadata`] for [`CombinedUnsizedData`].
#[derive(Derivative)]
#[derivative(
    Debug(
        bound = "T::Metadata: Debug, U::Metadata: Debug, <T as Pointee>::Metadata: Debug, <U as Pointee>::Metadata: Debug"
    ),
    Clone(
        bound = "T::Metadata: Clone, U::Metadata: Clone, <T as Pointee>::Metadata: Clone, <U as Pointee>::Metadata: Clone"
    ),
    Copy(
        bound = "T::Metadata: Copy, U::Metadata: Copy, <T as Pointee>::Metadata: Copy, <U as Pointee>::Metadata: Copy"
    )
)]
pub struct CombinedUnsizedDataMeta<T: ?Sized + UnsizedData, U: ?Sized + UnsizedData> {
    t_meta: T::Metadata,
    t_ptr_meta: <T as Pointee>::Metadata,
    u_meta: U::Metadata,
    u_ptr_meta: <U as Pointee>::Metadata,
    t_length: usize,
}

// Safety: Pointers are the same as input.
unsafe impl<T: ?Sized + UnsizedData, U: ?Sized + UnsizedData> UnsizedData
    for CombinedUnsizedData<T, U>
{
    type Metadata = CombinedUnsizedDataMeta<T, U>;

    fn min_data_size() -> usize {
        T::min_data_size() + U::min_data_size()
    }

    fn from_bytes<'a>(bytes: &mut &'a [u8]) -> common_utils::Result<(&'a Self, Self::Metadata)> {
        let bytes_advance = &mut &**bytes;
        let bytes_ptr_val = bytes_advance.as_ptr() as usize;
        let (t_val, t_meta) = T::from_bytes(bytes_advance)?;
        let t_length = bytes_advance.as_ptr() as usize - bytes_ptr_val;
        let (u_val, u_meta) = U::from_bytes(bytes_advance)?;
        let total = bytes_advance.as_ptr() as usize - bytes_ptr_val;
        Ok((
            // Safety: Pointer verified above.
            unsafe { &*ptr::from_raw_parts(bytes.advance(total).as_ptr().cast(), total) },
            CombinedUnsizedDataMeta {
                t_meta,
                t_ptr_meta: metadata(t_val),
                u_meta,
                u_ptr_meta: metadata(u_val),
                t_length,
            },
        ))
    }

    fn from_mut_bytes<'a>(
        bytes: &mut &'a mut [u8],
    ) -> common_utils::Result<(&'a mut Self, Self::Metadata)> {
        let bytes_advance = &mut &mut **bytes;
        let bytes_ptr_val = bytes_advance.as_ptr() as usize;
        let (t_val, t_meta) = T::from_mut_bytes(bytes_advance)?;
        let t_length = bytes_advance.as_ptr() as usize - bytes_ptr_val;
        let (u_val, u_meta) = U::from_mut_bytes(bytes_advance)?;
        let total = bytes_advance.as_ptr() as usize - bytes_ptr_val;
        let t_ptr_meta = metadata(t_val);
        let u_ptr_meta = metadata(u_val);
        Ok((
            // Safety: Pointer verified above.
            unsafe {
                &mut *ptr::from_raw_parts_mut(bytes.advance(total).as_mut_ptr().cast(), total)
            },
            CombinedUnsizedDataMeta {
                t_meta,
                t_ptr_meta,
                u_meta,
                u_ptr_meta,
                t_length,
            },
        ))
    }
}

#[cfg(test)]
mod test {
    use crate::testing::TestAccountInfo;
    use crate::versioned_account::account_info::AccountInfoData;
    use crate::versioned_account::combined::{
        CombinedAccountDataContext, CombinedAccountDataMutContext, CombinedUnsizedData,
    };
    use crate::versioned_account::list::ListContext;
    use crate::Result;
    use array_init::array_init;
    use bytemuck::{Pod, Zeroable};
    use common_proc::Align1;
    use common_utils::versioned_account::list::List;
    use num_traits::ToPrimitive;
    use solana_program::pubkey::Pubkey;

    #[repr(C, packed)]
    #[derive(Align1, Debug, Pod, Zeroable, Copy, Clone, Eq, PartialEq)]
    pub struct Data1 {
        pub data1: u64,
        pub data2: u32,
    }

    type BasicTest = CombinedUnsizedData<List<Data1, u32>, List<Pubkey, u8>>;

    #[test]
    fn basic_test() -> Result<()> {
        let mut account_data = TestAccountInfo::new(5);
        let pubkeys: [_; 20] = array_init(|i| Pubkey::from(array_init(|_| i.to_u8().unwrap())));

        {
            let account = account_data.account_info();
            {
                // Safety: We are using a valid account.
                let mut account_data_access = unsafe { account.data_access_mut() }?;
                {
                    let mut context = account_data_access.context_mut::<BasicTest>();
                    context.t_mut().push(Data1 { data1: 1, data2: 2 })?;

                    assert_eq!(**context.t(), [Data1 { data1: 1, data2: 2 }]);
                    assert_eq!(**context.u(), []);
                }
                {
                    let context = account_data_access.context::<BasicTest>();
                    assert_eq!(**context.t(), [Data1 { data1: 1, data2: 2 }]);
                    assert_eq!(**context.u(), []);
                }

                {
                    let mut context = account_data_access.context_mut::<BasicTest>();
                    context.u_mut().push_all(pubkeys)?;

                    assert_eq!(**context.t(), [Data1 { data1: 1, data2: 2 }]);
                    assert_eq!(**context.u(), pubkeys);
                }
            }
            {
                // Safety: We are using a valid account.
                let account_data_access = unsafe { account.data_access() }?;
                let context = account_data_access.context::<BasicTest>();
                assert_eq!(**context.t(), [Data1 { data1: 1, data2: 2 }]);
                assert_eq!(**context.u(), pubkeys);
            }
        }
        account_data.refresh_data_increase();
        {
            let account = account_data.account_info();
            // Safety: We are using a valid account.
            let account_data_access = unsafe { account.data_access() }?;
            let context = account_data_access.context::<BasicTest>();
            assert_eq!(**context.t(), [Data1 { data1: 1, data2: 2 }]);
            assert_eq!(**context.u(), pubkeys);
        }

        Ok(())
    }
}
