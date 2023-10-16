//! Context for accessing subsets of account data.

use crate::Result;
use common_utils::util::{MaybeMutRef, MaybeRef};
use common_utils::versioned_account::unsized_data::UnsizedData;
use derivative::Derivative;
use std::convert::Infallible;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::ptr::{NonNull, Pointee};

/// Combination trait for both [`AccountDataRefContext`] and [`AccountDataMutContext`].
pub trait AccountDataContext<T: ?Sized + UnsizedData>: Deref<Target = T> {
    /// Gets the [`UnsizedData::Metadata`] for this data.
    fn data_meta(&self) -> &T::Metadata;

    /// Tries to get a sub-context using the index operation.
    fn try_sub_context<'b, O: ?Sized + UnsizedData, E>(
        &'b self,
        index_op: impl FnOnce(&'b T, &'b T::Metadata) -> Result<(&'b O, MaybeRef<'b, O::Metadata>), E>,
    ) -> Result<AccountDataRefContext<'b, O>, E>;
    /// Gets a sub-context using the index operation.
    fn sub_context<'b, O: ?Sized + UnsizedData>(
        &'b self,
        index_op: impl FnOnce(&'b T, &'b T::Metadata) -> (&'b O, MaybeRef<'b, O::Metadata>),
    ) -> AccountDataRefContext<'b, O> {
        self.try_sub_context(move |val, meta| Ok::<_, Infallible>(index_op(val, meta)))
            .unwrap()
    }

    /// Tries to get two sub-contexts using the index operation.
    fn try_split_context<'b, O1: ?Sized + UnsizedData, O2: ?Sized + UnsizedData, E>(
        &'b self,
        index_op: impl FnOnce(
            &'b T,
            &'b T::Metadata,
        ) -> Result<
            (
                (&'b O1, MaybeRef<'b, O1::Metadata>),
                (&'b O2, MaybeRef<'b, O2::Metadata>),
            ),
            E,
        >,
    ) -> Result<(AccountDataRefContext<'b, O1>, AccountDataRefContext<'b, O2>), E>;
    /// Gets two sub-contexts using the index operation.
    fn split_context<'b, O1: ?Sized + UnsizedData, O2: ?Sized + UnsizedData>(
        &'b self,
        index_op: impl FnOnce(
            &'b T,
            &'b T::Metadata,
        ) -> (
            (&'b O1, MaybeRef<'b, O1::Metadata>),
            (&'b O2, MaybeRef<'b, O2::Metadata>),
        ),
    ) -> (AccountDataRefContext<'b, O1>, AccountDataRefContext<'b, O2>) {
        self.try_split_context(move |val, meta| Ok::<_, Infallible>(index_op(val, meta)))
            .unwrap()
    }
}

/// Immutable context for accessing subsets of account data.
#[derive(Debug, Copy, Clone)]
pub struct AccountDataRefContext<'a, T: ?Sized + UnsizedData> {
    pub(crate) meta: MaybeRef<'a, T::Metadata>,
    pub(crate) data: &'a T,
}

impl<'a, T: ?Sized + UnsizedData> Deref for AccountDataRefContext<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<'a, T: ?Sized + UnsizedData> AccountDataContext<T> for AccountDataRefContext<'a, T> {
    fn data_meta(&self) -> &T::Metadata {
        &self.meta
    }

    fn try_sub_context<'b, O: ?Sized + UnsizedData, E>(
        &'b self,
        index_op: impl FnOnce(&'b T, &'b T::Metadata) -> Result<(&'b O, MaybeRef<'b, O::Metadata>), E>,
    ) -> Result<AccountDataRefContext<'b, O>, E> {
        let (data, meta) = index_op(self.data, &*self.meta)?;
        Ok(AccountDataRefContext { meta, data })
    }

    fn try_split_context<'b, O1: ?Sized + UnsizedData, O2: ?Sized + UnsizedData, E>(
        &'b self,
        index_op: impl FnOnce(
            &'b T,
            &'b T::Metadata,
        ) -> Result<
            (
                (&'b O1, MaybeRef<'b, O1::Metadata>),
                (&'b O2, MaybeRef<'b, O2::Metadata>),
            ),
            E,
        >,
    ) -> Result<(AccountDataRefContext<'b, O1>, AccountDataRefContext<'b, O2>), E> {
        let ((data1, meta1), (data2, meta2)) = index_op(self.data, &*self.meta)?;
        Ok((
            AccountDataRefContext {
                meta: meta1,
                data: data1,
            },
            AccountDataRefContext {
                meta: meta2,
                data: data2,
            },
        ))
    }
}

/// Mutable context for accessing subsets of account data.
#[derive(Derivative)]
#[derivative(Debug(bound = "T::Metadata: Debug"))]
pub struct AccountDataMutContext<'a, T: ?Sized + UnsizedData> {
    pub(super) original_data_length: usize,
    pub(super) data_meta: MaybeMutRef<'a, T::Metadata>,
    /// This also has the data and bytes_ptr lifetime.
    /// This should never be called while bytes_ptr or data is being used.
    #[derivative(Debug = "ignore")]
    pub(super) set_length: Box<dyn FnMut(usize, <T as Pointee>::Metadata) -> Result<()> + 'a>,
    pub(super) data: NonNull<T>,
}

impl<'a, T: ?Sized + UnsizedData> AccountDataContext<T> for AccountDataMutContext<'a, T> {
    fn data_meta(&self) -> &T::Metadata {
        &self.data_meta
    }

    fn try_sub_context<'b, O: ?Sized + UnsizedData, E>(
        &'b self,
        index_op: impl FnOnce(&'b T, &'b T::Metadata) -> Result<(&'b O, MaybeRef<'b, O::Metadata>), E>,
    ) -> Result<AccountDataRefContext<'b, O>, E> {
        // Safety: Neither bytes_ptr nor data can be mutated without a mutable reference to self.
        let (data, meta) = index_op(unsafe { self.data.as_ref() }, &*self.data_meta)?;
        Ok(AccountDataRefContext { meta, data })
    }

    fn try_split_context<'b, O1: ?Sized + UnsizedData, O2: ?Sized + UnsizedData, E>(
        &'b self,
        index_op: impl FnOnce(
            &'b T,
            &'b T::Metadata,
        ) -> Result<
            (
                (&'b O1, MaybeRef<'b, O1::Metadata>),
                (&'b O2, MaybeRef<'b, O2::Metadata>),
            ),
            E,
        >,
    ) -> Result<(AccountDataRefContext<'b, O1>, AccountDataRefContext<'b, O2>), E> {
        let ((data1, meta1), (data2, meta2)) =
            // Safety: Neither bytes_ptr nor data can be mutated without a mutable reference to self.
            index_op(unsafe { self.data.as_ref() }, &*self.data_meta)?;
        Ok((
            AccountDataRefContext {
                meta: meta1,
                data: data1,
            },
            AccountDataRefContext {
                meta: meta2,
                data: data2,
            },
        ))
    }
}

/// Args for building a mutable context.
#[derive(Derivative)]
#[derivative(Debug(bound = "T::Metadata: Debug"))]
pub struct MutContextIndexArgs<'a, T: ?Sized + UnsizedData> {
    /// Parent Metadata
    pub data_meta: &'a mut T::Metadata,
    /// Data to access, should not be accessed at the same time as [`MutContextIndexArgs::set_length`]
    pub data: &'a mut NonNull<T>,
    /// Sets the items length.
    #[derivative(Debug = "ignore")]
    pub set_length: &'a mut (dyn FnMut(usize, <T as Pointee>::Metadata) -> Result<()> + 'a),
}
impl<'a, T: ?Sized + UnsizedData> AccountDataMutContext<'a, T> {
    /// Tries to get a sub-context using the index operation.
    pub fn try_sub_context_mut<'b, O: 'b + ?Sized + UnsizedData, E>(
        &'b mut self,
        index_op: impl FnOnce(
            MutContextIndexArgs<'b, T>,
        ) -> Result<
            (
                &'b mut O,
                MaybeMutRef<'b, O::Metadata>,
                Box<dyn FnMut(usize, <O as Pointee>::Metadata) -> Result<()> + 'b>,
            ),
            E,
        >,
    ) -> Result<AccountDataMutContext<'b, O>, E> {
        let original_data_length = self.original_data_length;
        let (data, data_meta, set_length) = index_op(MutContextIndexArgs {
            data_meta: &mut self.data_meta,
            data: &mut self.data,
            set_length: &mut self.set_length,
        })?;
        Ok(AccountDataMutContext {
            original_data_length,
            data_meta,
            set_length,
            data: NonNull::from(data),
        })
    }

    /// Gets a sub-context using the index operation.
    pub fn sub_context_mut<'b, O: 'b + ?Sized + UnsizedData>(
        &'b mut self,
        index_op: impl FnOnce(
            MutContextIndexArgs<'b, T>,
        ) -> (
            &'b mut O,
            MaybeMutRef<'b, O::Metadata>,
            Box<dyn FnMut(usize, <O as Pointee>::Metadata) -> Result<()> + 'b>,
        ),
    ) -> AccountDataMutContext<'b, O> {
        self.try_sub_context_mut(move |this| Ok::<_, Infallible>(index_op(this)))
            .unwrap()
    }

    /// Tries to get two sub-contexts using the index operation.
    pub fn try_split_context_mut<'b, O1: 'b + ?Sized, O2: 'b + ?Sized + UnsizedData, E>(
        &'b mut self,
        index_op: impl FnOnce(
            MutContextIndexArgs<'b, T>,
        ) -> Result<
            (
                &'b mut O1,
                &'b mut O2,
                MaybeMutRef<'b, O2::Metadata>,
                Box<dyn FnMut(usize, <O2 as Pointee>::Metadata) -> Result<()> + 'b>,
            ),
            E,
        >,
    ) -> Result<(&'b mut O1, AccountDataMutContext<'b, O2>), E> {
        let original_data_length = self.original_data_length;
        let (data1, data2, data_meta, set_length) = index_op(MutContextIndexArgs {
            data_meta: &mut self.data_meta,
            data: &mut self.data,
            set_length: &mut self.set_length,
        })?;
        Ok((
            data1,
            AccountDataMutContext {
                original_data_length,
                data_meta,
                set_length,
                data: NonNull::from(data2),
            },
        ))
    }

    /// Gets two sub-contexts using the index operation.
    pub fn split_context_mut<'b, O1: 'b + ?Sized, O2: 'b + ?Sized + UnsizedData>(
        &'b mut self,
        index_op: impl FnOnce(
            MutContextIndexArgs<'b, T>,
        ) -> (
            &'b mut O1,
            &'b mut O2,
            MaybeMutRef<'b, O2::Metadata>,
            Box<dyn FnMut(usize, <O2 as Pointee>::Metadata) -> Result<()> + 'b>,
        ),
    ) -> (&'b mut O1, AccountDataMutContext<'b, O2>) {
        self.try_split_context_mut(move |this| Ok::<_, Infallible>(index_op(this)))
            .unwrap()
    }

    /// Gets the metadata mutably.
    pub fn data_meta_mut(&mut self) -> &mut T::Metadata {
        &mut self.data_meta
    }

    /// Gets the data and metadata.
    pub fn data_and_meta_mut(&mut self) -> (&mut T, &mut T::Metadata) {
        // Safety: We don't access bytes while this function runs.
        unsafe { (self.data.as_mut(), &mut *self.data_meta) }
    }
}

impl<'a, T: ?Sized + UnsizedData> Deref for AccountDataMutContext<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // Safety: The pointer is valid.
        unsafe { self.data.as_ref() }
    }
}

impl<'a, T: ?Sized + UnsizedData> DerefMut for AccountDataMutContext<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // Safety: The pointer is valid.
        unsafe { self.data.as_mut() }
    }
}
