//! Unsized implementation for enums.

use crate::util::MaybeMutRef;
use crate::versioned_account::context::AccountDataMutContext;
use crate::versioned_account::unsized_data::UnsizedData;
use crate::{error, Advance, Align1, PackedValue, Result, UnitEnumFromRepr, UtilError};
use common_utils::versioned_account::context::AccountDataContext;
use solana_program::msg;
use solana_program::program_memory::sol_memset;
use std::mem::size_of;
use std::ops::DerefMut;
use std::ptr;
use std::ptr::{NonNull, Pointee};

/// Enum that can be unsized.
///
/// # Safety
/// [`UnsizedDataEnum::MetadataEnum`] must be enumerated as the metadata of all variants.
/// [`UnsizedDataEnum::Context`] and [`UnsizedDataEnum::ContextMut`] must be enumerated as the context of all variants.
pub unsafe trait UnsizedDataEnum: 'static + UnitEnumFromRepr {
    /// Metadata for the unsized data to be able to construct sub-contexts.
    type MetadataEnum: 'static;
    /// Context for the unsized data.
    type Context<'a>;
    /// Mutable context for the unsized data.
    type ContextMut<'a>;

    /// Reads a given discriminant's variant data.
    fn byte_advance_for_data(discriminant: Self, bytes: &mut &[u8]) -> Result<Self::MetadataEnum>;
    /// Reads a given discriminant's variant data mutably.
    fn byte_advance_for_data_mut(
        discriminant: Self,
        bytes: &mut &mut [u8],
    ) -> Result<Self::MetadataEnum>;

    /// Gets the context for the given data.
    fn context(context: &impl AccountDataContext<UnsizedEnum<Self>>) -> Result<Self::Context<'_>>
    where
        Self: UnsizedDataEnumDefaultVariant;
    /// Gets the mutable context for the given data.
    fn context_mut<'a>(
        context: &'a mut AccountDataMutContext<UnsizedEnum<Self>>,
    ) -> Result<Self::ContextMut<'a>>
    where
        Self: UnsizedDataEnumDefaultVariant;
}
/// Implementation for a variant of an enum.
///
/// # Safety
/// Must be implemented exactly once for each variant of an enum with the proper types.
pub unsafe trait UnsizedDataEnumVariant<Marker>: UnsizedDataEnum {
    /// The variant this is for.
    const VARIANT: Self;
    /// The data type for this variant.
    type VariantType: ?Sized + VariantType<Self, Marker>;
}
/// Implementation for a variant of an enum.
///
/// # Safety
/// Must be matched with a [`UnsizedDataEnumVariant`] implementation.
pub unsafe trait VariantType<E: UnsizedDataEnum, Marker>: UnsizedData {
    /// Converts the metadata for this variant into the metadata for the enum.
    fn meta_into_enum_meta(
        meta: <Self as UnsizedData>::Metadata,
        ptr_meta: <Self as Pointee>::Metadata,
    ) -> E::MetadataEnum;
}
/// The default variant for an enum.
pub trait UnsizedDataEnumDefaultVariant: UnsizedDataEnumVariant<Self::DefaultMarker> {
    /// The default variant marker.
    type DefaultMarker;
}

/// Unsized wrapper for an enum.
#[derive(Align1, Debug)]
pub struct UnsizedEnum<E: UnsizedDataEnum> {
    discriminant: PackedValue<E::Repr>,
    bytes: [u8],
}
impl<E: UnsizedDataEnum> UnsizedEnum<E> {
    /// Gets the discriminant of this enum.
    pub fn discriminant(&self) -> Result<E> {
        E::from_repr(self.discriminant.0).map_err(|e| {
            msg!("Invalid enum discriminant: {}", e);
            error!(UtilError::InvalidEnumDiscriminant)
        })
    }

    /// Gets the bytes of this enum.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Gets the bytes of this enum mutably.
    ///
    /// # Safety
    /// Must not invalidate the underlying type.
    pub unsafe fn bytes_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }
}

/// Extension for [`AccountDataContext`] wrapping enums.
pub trait UnsizedDataEnumContextExtension<E: UnsizedDataEnumDefaultVariant>:
    AccountDataContext<UnsizedEnum<E>>
{
    /// Gets the immutable context for the enum.
    fn enum_context(&self) -> Result<E::Context<'_>>;
}
impl<T, E: UnsizedDataEnumDefaultVariant> UnsizedDataEnumContextExtension<E> for T
where
    T: AccountDataContext<UnsizedEnum<E>>,
{
    fn enum_context(&self) -> Result<E::Context<'_>> {
        E::context(self)
    }
}
/// Extension for [`AccountDataMutContext`] wrapping enums.
pub trait UnsizedDataEnumMutContextExtension<E: UnsizedDataEnumDefaultVariant>:
    UnsizedDataEnumContextExtension<E> + DerefMut
{
    /// Gets the mutable context for the enum.
    fn enum_context_mut(&mut self) -> Result<E::ContextMut<'_>>;

    /// Sets the variant of the enum, defaulting to the default data for that variant.
    fn set_enum<Marker>(&mut self) -> Result<()>
    where
        E: UnsizedDataEnumVariant<Marker>;

    /// Sets the variant of the enum, using the given mapper to set the data.
    ///
    /// # Safety
    /// The mapper must return the size of the data needed for the new variant.
    /// Must also manipulate the bytes to be valid for the new variant.
    unsafe fn set_enum_with_mapper<Marker, F>(
        &mut self,
        mapper: impl FnOnce(
            E,
            &mut [u8],
            &mut E::MetadataEnum,
        ) -> Result<(
            usize,
            <<E as UnsizedDataEnumVariant<Marker>>::VariantType as UnsizedData>::Metadata,
            <<E as UnsizedDataEnumVariant<Marker>>::VariantType as Pointee>::Metadata,
        )>,
    ) -> Result<()>
    where
        E: UnsizedDataEnumVariant<Marker>;
}
impl<'a, E: UnsizedDataEnumDefaultVariant> UnsizedDataEnumMutContextExtension<E>
    for AccountDataMutContext<'a, UnsizedEnum<E>>
{
    fn enum_context_mut(&mut self) -> Result<E::ContextMut<'_>> {
        E::context_mut(self)
    }

    fn set_enum<Marker>(&mut self) -> Result<()>
    where
        E: UnsizedDataEnumVariant<Marker>,
    {
        let init_data_size = <E as UnsizedDataEnumVariant<Marker>>::VariantType::init_data_size();

        (self.set_length)(size_of::<E::Repr>() + init_data_size, init_data_size)?;
        self.data = NonNull::from_raw_parts(self.data.cast(), init_data_size);

        // Safety: We don't set length while data is reffed.
        unsafe {
            self.data.as_mut().discriminant = <E as UnsizedDataEnumVariant<Marker>>::VARIANT
                .into_repr()
                .into();
            sol_memset(&mut self.data.as_mut().bytes, 0, init_data_size);
            let (data, meta) = <E as UnsizedDataEnumVariant<Marker>>::VariantType::init(
                &mut self.data.as_mut().bytes,
            )?;
            self.data_meta = MaybeMutRef::Owned(
                <E as UnsizedDataEnumVariant<Marker>>::VariantType::meta_into_enum_meta(
                    meta,
                    ptr::metadata(data),
                ),
            );
        };

        Ok(())
    }

    unsafe fn set_enum_with_mapper<Marker, F>(
        &mut self,
        mapper: impl FnOnce(
            E,
            &mut [u8],
            &mut E::MetadataEnum,
        ) -> Result<(
            usize,
            <<E as UnsizedDataEnumVariant<Marker>>::VariantType as UnsizedData>::Metadata,
            <<E as UnsizedDataEnumVariant<Marker>>::VariantType as Pointee>::Metadata,
        )>,
    ) -> Result<()>
    where
        E: UnsizedDataEnumVariant<Marker>,
    {
        let (new_length, meta, ptr_meta) = mapper(
            self.discriminant()?,
            &mut self.data.as_mut().bytes,
            &mut self.data_meta,
        )?;
        self.data.as_mut().discriminant = <E as UnsizedDataEnumVariant<Marker>>::VARIANT
            .into_repr()
            .into();

        (self.set_length)(size_of::<E::Repr>() + new_length, new_length)?;
        self.data_meta = MaybeMutRef::Owned(
            <E as UnsizedDataEnumVariant<Marker>>::VariantType::meta_into_enum_meta(meta, ptr_meta),
        );
        Ok(())
    }
}

// Safety:
unsafe impl<E> UnsizedData for UnsizedEnum<E>
where
    E: UnsizedDataEnumDefaultVariant,
{
    type Metadata = E::MetadataEnum;

    fn init_data_size() -> usize {
        size_of::<E::Repr>()
            + <E as UnsizedDataEnumVariant<E::DefaultMarker>>::VariantType::init_data_size()
    }

    unsafe fn init(mut bytes: &mut [u8]) -> Result<(&mut Self, Self::Metadata)> {
        assert_eq!(bytes.len(), Self::init_data_size());
        let bytes_ptr = bytes.as_mut_ptr();
        let (discriminant, _) =
            PackedValue::<E::Repr>::init(bytes.advance(PackedValue::<E::Repr>::init_data_size()))?;
        *discriminant = <E as UnsizedDataEnumVariant<E::DefaultMarker>>::VARIANT
            .into_repr()
            .into();
        let start = bytes.as_ptr() as usize;
        let (default_data, default_meta) =
            <E as UnsizedDataEnumVariant<E::DefaultMarker>>::VariantType::init(bytes.advance(
                <E as UnsizedDataEnumVariant<E::DefaultMarker>>::VariantType::init_data_size(),
            ))?;
        let end = bytes.as_ptr() as usize;
        assert_eq!(bytes.len(), 0);
        Ok((
            &mut *ptr::from_raw_parts_mut(bytes_ptr.cast(), end - start),
            <E as UnsizedDataEnumVariant<E::DefaultMarker>>::VariantType::meta_into_enum_meta(
                default_meta,
                ptr::metadata(default_data),
            ),
        ))
    }

    fn from_bytes<'a>(bytes: &mut &'a [u8]) -> Result<(&'a Self, Self::Metadata)> {
        let bytes_ptr = bytes.as_ptr();
        let (discriminant, _) = PackedValue::<E::Repr>::from_bytes(bytes)?;
        let start = bytes.as_ptr() as usize;
        let data_meta = E::byte_advance_for_data(
            E::from_repr(discriminant.0).map_err(|e| {
                msg!("Invalid enum discriminant: {}", e);
                error!(UtilError::InvalidEnumDiscriminant)
            })?,
            bytes,
        )?;
        let end = bytes.as_ptr() as usize;
        Ok((
            // Safety: Pointer is same size as advanced.
            unsafe { &*ptr::from_raw_parts(bytes_ptr.cast(), end - start) },
            data_meta,
        ))
    }

    fn from_mut_bytes<'a>(bytes: &mut &'a mut [u8]) -> Result<(&'a mut Self, Self::Metadata)> {
        let bytes_ptr = bytes.as_mut_ptr();
        let (discriminant, _) = PackedValue::<E::Repr>::from_mut_bytes(bytes)?;
        let start = bytes.as_ptr() as usize;
        let data_meta = E::byte_advance_for_data_mut(
            E::from_repr(discriminant.0).map_err(|e| {
                msg!("Invalid enum discriminant: {}", e);
                error!(UtilError::InvalidEnumDiscriminant)
            })?,
            bytes,
        )?;
        let end = bytes.as_ptr() as usize;
        Ok((
            // Safety: Pointer is same size as advanced.
            unsafe { &mut *ptr::from_raw_parts_mut(bytes_ptr.cast(), end - start) },
            data_meta,
        ))
    }
}

#[cfg(test)]
mod test {
    use crate::testing::TestAccountInfo;
    use crate::util::MaybeMutRef;
    use crate::versioned_account::account_info::AccountInfoData;
    use crate::versioned_account::context::{AccountDataContext, AccountDataRefContext};
    use crate::versioned_account::enum_impl::{
        UnsizedDataEnumContextExtension, UnsizedDataEnumDefaultVariant,
        UnsizedDataEnumMutContextExtension, UnsizedDataEnumVariant, VariantType,
    };
    use crate::versioned_account::list::{List, ListContext};
    use crate::versioned_account::test::{AccountStuff, AccountStuffMutExtension, Data2};
    use crate::versioned_account::unsized_data::UnsizedData;
    use crate::{error, PackedValue, Result, UnitEnumFromRepr, UtilError};
    use common_utils::util::MaybeRef;
    use common_utils::versioned_account::context::AccountDataMutContext;
    use common_utils::versioned_account::enum_impl::{UnsizedDataEnum, UnsizedEnum};
    use solana_program::msg;
    use solana_program::pubkey::Pubkey;
    use std::mem::{size_of, size_of_val};
    use std::ptr;
    use std::ptr::{NonNull, Pointee};

    #[repr(u8)]
    #[derive(UnitEnumFromRepr, Copy, Clone, Debug, Eq, PartialEq)]
    pub(crate) enum EnumAccountStuff {
        // Type = List<Data2, u32>
        List = 5,
        // Type = AccountStuff
        AccountStuff = 20,
    }

    pub(crate) struct ListMarker;
    pub(crate) struct AccountStuffMarker;
    #[repr(u8)]
    pub(crate) enum EnumAccountStuffMeta {
        List {
            meta: <List<PackedValue<u32>, u32> as UnsizedData>::Metadata,
            ptr_meta: <List<PackedValue<u32>, u32> as Pointee>::Metadata,
        },
        AccountStuff {
            meta: <AccountStuff as UnsizedData>::Metadata,
            ptr_meta: <AccountStuff as Pointee>::Metadata,
        },
    }
    // Safety: Impl matches paired impl.
    unsafe impl VariantType<EnumAccountStuff, ListMarker> for List<PackedValue<u32>, u32> {
        fn meta_into_enum_meta(
            meta: <Self as UnsizedData>::Metadata,
            ptr_meta: <Self as Pointee>::Metadata,
        ) -> <EnumAccountStuff as UnsizedDataEnum>::MetadataEnum {
            EnumAccountStuffMeta::List { meta, ptr_meta }
        }
    }
    // Safety: Impl matches paired impl.
    unsafe impl VariantType<EnumAccountStuff, AccountStuffMarker> for AccountStuff {
        fn meta_into_enum_meta(
            meta: <Self as UnsizedData>::Metadata,
            ptr_meta: <Self as Pointee>::Metadata,
        ) -> <EnumAccountStuff as UnsizedDataEnum>::MetadataEnum {
            EnumAccountStuffMeta::AccountStuff { meta, ptr_meta }
        }
    }

    pub(crate) enum EnumAccountStuffContext<'a> {
        List(AccountDataRefContext<'a, List<PackedValue<u32>, u32>>),
        AccountStuff(AccountDataRefContext<'a, AccountStuff>),
    }

    pub(crate) enum EnumAccountStuffContextMut<'a> {
        List(AccountDataMutContext<'a, List<PackedValue<u32>, u32>>),
        AccountStuff(AccountDataMutContext<'a, AccountStuff>),
    }

    // Safety: Everything in impl-ed properly
    unsafe impl UnsizedDataEnum for EnumAccountStuff {
        type MetadataEnum = EnumAccountStuffMeta;
        type Context<'a> = EnumAccountStuffContext<'a>;
        type ContextMut<'a> = EnumAccountStuffContextMut<'a>;

        fn byte_advance_for_data(
            discriminant: Self,
            bytes: &mut &[u8],
        ) -> Result<Self::MetadataEnum> {
            let bytes_start = bytes.as_ptr() as usize;
            match discriminant {
                Self::List => {
                    let (data, meta) = List::<PackedValue<u32>, u32>::from_bytes(bytes)?;
                    assert_eq!(bytes_start + size_of_val(data), bytes.as_ptr() as usize);
                    Ok(EnumAccountStuffMeta::List {
                        meta,
                        ptr_meta: ptr::metadata(data),
                    })
                }
                Self::AccountStuff => {
                    let (data, meta) = AccountStuff::from_bytes(bytes)?;
                    assert_eq!(bytes_start + size_of_val(data), bytes.as_ptr() as usize);
                    Ok(EnumAccountStuffMeta::AccountStuff {
                        meta,
                        ptr_meta: ptr::metadata(data),
                    })
                }
            }
        }

        fn byte_advance_for_data_mut(
            discriminant: Self,
            bytes: &mut &mut [u8],
        ) -> Result<Self::MetadataEnum> {
            let bytes_start = bytes.as_ptr() as usize;
            match discriminant {
                Self::List => {
                    let (data, meta) = List::<PackedValue<u32>, u32>::from_mut_bytes(bytes)?;
                    assert_eq!(bytes_start + size_of_val(data), bytes.as_ptr() as usize);
                    Ok(EnumAccountStuffMeta::List {
                        meta,
                        ptr_meta: ptr::metadata(data),
                    })
                }
                Self::AccountStuff => {
                    let (data, meta) = AccountStuff::from_mut_bytes(bytes)?;
                    assert_eq!(bytes_start + size_of_val(data), bytes.as_ptr() as usize);
                    Ok(EnumAccountStuffMeta::AccountStuff {
                        meta,
                        ptr_meta: ptr::metadata(data),
                    })
                }
            }
        }

        fn context(
            context: &impl AccountDataContext<UnsizedEnum<Self>>,
        ) -> Result<Self::Context<'_>>
        where
            Self: UnsizedDataEnumDefaultVariant,
        {
            match Self::from_repr(context.discriminant.0).map_err(|e| {
                msg!("Invalid enum discriminant: {}", e);
                error!(UtilError::InvalidEnumDiscriminant)
            })? {
                Self::List => Ok(EnumAccountStuffContext::List(context.try_sub_context(
                    |data, meta| {
                        if let Self::MetadataEnum::List { meta, ptr_meta } = meta {
                            Ok((
                                // Safety: We don't change lifetimes and limit the size of the slice.
                                unsafe {
                                    &*ptr::from_raw_parts(data.bytes.as_ptr().cast(), *ptr_meta)
                                },
                                MaybeRef::Ref(meta),
                            ))
                        } else {
                            msg!("Enum variant mismatch");
                            Err(error!(UtilError::GenericError))
                        }
                    },
                )?)),
                Self::AccountStuff => Ok(EnumAccountStuffContext::AccountStuff(
                    context.try_sub_context(|data, meta| {
                        if let Self::MetadataEnum::AccountStuff { meta, ptr_meta } = meta {
                            Ok((
                                // Safety: We don't change lifetimes and limit the size of the slice.
                                unsafe {
                                    &*ptr::from_raw_parts(data.bytes.as_ptr().cast(), *ptr_meta)
                                },
                                MaybeRef::Ref(meta),
                            ))
                        } else {
                            msg!("Enum variant mismatch");
                            Err(error!(UtilError::GenericError))
                        }
                    })?,
                )),
            }
        }

        fn context_mut<'a>(
            context: &'a mut AccountDataMutContext<UnsizedEnum<Self>>,
        ) -> Result<Self::ContextMut<'a>>
        where
            Self: UnsizedDataEnumDefaultVariant,
        {
            match Self::from_repr(context.discriminant.0).map_err(|e| {
                msg!("Invalid enum discriminant: {}", e);
                error!(UtilError::InvalidEnumDiscriminant)
            })? {
                Self::List => Ok(Self::ContextMut::List(context.try_sub_context_mut(
                    |args| {
                        if let Self::MetadataEnum::List { meta, ptr_meta } = args.data_meta {
                            Ok((
                                // Safety: We don't change lifetimes and limit the size of the slice.
                                unsafe {
                                    &mut *ptr::from_raw_parts_mut(
                                        args.data.as_mut().bytes.as_mut_ptr().cast(),
                                        *ptr_meta,
                                    )
                                },
                                MaybeMutRef::Mut(meta),
                                Box::new(|new_length, new_meta| {
                                    *ptr_meta = new_meta;
                                    *args.data =
                                        NonNull::from_raw_parts(args.data.cast(), new_length);
                                    (args.set_length)(
                                        new_length + size_of::<PackedValue<Self::Repr>>(),
                                        new_length,
                                    )?;
                                    Ok(())
                                }),
                            ))
                        } else {
                            msg!("Enum variant mismatch");
                            Err(error!(UtilError::GenericError))
                        }
                    },
                )?)),
                Self::AccountStuff => Ok(Self::ContextMut::AccountStuff(
                    context.try_sub_context_mut(|args| {
                        if let Self::MetadataEnum::AccountStuff { meta, ptr_meta } = args.data_meta
                        {
                            Ok((
                                // Safety: We don't change lifetimes and limit the size of the slice.
                                unsafe {
                                    &mut *ptr::from_raw_parts_mut(
                                        args.data.as_mut().bytes.as_mut_ptr().cast(),
                                        *ptr_meta,
                                    )
                                },
                                MaybeMutRef::Mut(meta),
                                Box::new(|new_length, new_meta| {
                                    *ptr_meta = new_meta;
                                    *args.data =
                                        NonNull::from_raw_parts(args.data.cast(), new_length);
                                    (args.set_length)(
                                        new_length + size_of::<PackedValue<Self::Repr>>(),
                                        new_length,
                                    )?;
                                    Ok(())
                                }),
                            ))
                        } else {
                            msg!("Enum variant mismatch");
                            Err(error!(UtilError::GenericError))
                        }
                    })?,
                )),
            }
        }
    }
    // Safety: Impl matches paired impl.
    unsafe impl UnsizedDataEnumVariant<ListMarker> for EnumAccountStuff {
        const VARIANT: Self = Self::List;
        type VariantType = List<PackedValue<u32>, u32>;
    }
    // Safety: Impl matches paired impl.
    unsafe impl UnsizedDataEnumVariant<AccountStuffMarker> for EnumAccountStuff {
        const VARIANT: Self = Self::AccountStuff;
        type VariantType = AccountStuff;
    }
    impl UnsizedDataEnumDefaultVariant for EnumAccountStuff {
        type DefaultMarker = AccountStuffMarker;
    }

    #[test]
    fn enum_test() -> Result<()> {
        println!(
            "Init Size: {}",
            UnsizedEnum::<EnumAccountStuff>::init_data_size()
        );
        let mut account_data =
            TestAccountInfo::new(UnsizedEnum::<EnumAccountStuff>::init_data_size());
        {
            let account = account_data.account_info();
            // Safety: We are using a valid account and bytes are zeroed.
            unsafe { UnsizedEnum::<EnumAccountStuff>::init(&mut account.data.borrow_mut())? };

            {
                // Safety: We are using a valid account.
                let account_data_access = unsafe { account.data_access() }?;
                let context = account_data_access.context::<UnsizedEnum<EnumAccountStuff>>();

                if let EnumAccountStuffContext::AccountStuff(stuff) = context.enum_context()? {
                    assert_eq!({ stuff.header_val.0 }, 0);
                    assert_eq!({ stuff.other_val.0 }, 0);
                    assert_eq!(stuff.key, Pubkey::from([0; 32]));
                    assert_eq!(*stuff.items, []);
                } else {
                    panic!("Wrong Default");
                }
            }

            {
                // Safety: We are using a valid account.
                let mut account_data_access = unsafe { account.data_access_mut() }?;
                {
                    let mut context =
                        account_data_access.context_mut::<UnsizedEnum<EnumAccountStuff>>();

                    let context = context.enum_context_mut()?;
                    if let EnumAccountStuffContextMut::AccountStuff(mut stuff) = context {
                        stuff.header_val = PackedValue(1);
                        stuff.key = Pubkey::from([5; 32]);

                        stuff.items_mut().push_all([
                            Data2 { val1: 4, val2: 76 },
                            Data2 {
                                val1: 123,
                                val2: 65,
                            },
                        ])?;
                    } else {
                        panic!("Wrong disc");
                    }
                }

                {
                    let context = account_data_access.context::<UnsizedEnum<EnumAccountStuff>>();
                    if let EnumAccountStuffContext::AccountStuff(stuff) = context.enum_context()? {
                        assert_eq!({ stuff.header_val.0 }, 1);
                        assert_eq!({ stuff.other_val.0 }, 0);
                        assert_eq!(stuff.key, Pubkey::from([5; 32]));
                        assert_eq!(
                            *stuff.items,
                            [
                                Data2 { val1: 4, val2: 76 },
                                Data2 {
                                    val1: 123,
                                    val2: 65,
                                }
                            ]
                        );
                    } else {
                        panic!("Wrong disc");
                    }
                }

                {
                    let mut context =
                        account_data_access.context_mut::<UnsizedEnum<EnumAccountStuff>>();

                    context.set_enum::<ListMarker>()?;
                    let context = context.enum_context_mut()?;
                    if let EnumAccountStuffContextMut::List(mut list) = context {
                        list.push_all([1.into(), 2.into(), 3.into()])?;
                    } else {
                        panic!("Wrong dics");
                    }
                }

                {
                    let context = account_data_access.context::<UnsizedEnum<EnumAccountStuff>>();
                    if let EnumAccountStuffContext::List(list) = context.enum_context()? {
                        assert_eq!(**list, [PackedValue(1), PackedValue(2), PackedValue(3)]);
                    } else {
                        panic!("Wrong disc");
                    }
                }
            }
            {
                // Safety: We are using a valid account.
                let account_data_access = unsafe { account.data_access() }?;
                let context = account_data_access.context::<UnsizedEnum<EnumAccountStuff>>();

                if let EnumAccountStuffContext::List(list) = context.enum_context()? {
                    assert_eq!(**list, [PackedValue(1), PackedValue(2), PackedValue(3)]);
                } else {
                    panic!("Wrong disc");
                }
            }
        }

        Ok(())
    }
}
