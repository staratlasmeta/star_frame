use crate::prelude::*;
use crate::unsize::init::DefaultInit;
use crate::unsize::UnsizedType;
use advancer::Advance;
use anyhow::Context as _;
use bytemuck::bytes_of;
pub use star_frame_proc::ProgramAccount;
use std::marker::PhantomData;
use std::mem::size_of;

/// Increases or decreases the rent of self to be the minimum required using [`CanModifyRent::normalize_rent`].
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct NormalizeRent<T>(pub T);

/// Decreases the rent of self to be the minimum required using [`CanModifyRent::refund_rent`].
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct RefundRent<T>(pub T);

/// Increases the rent of self to be at least the minimum rent using [`CanModifyRent::receive_rent`].
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct ReceiveRent<T>(pub T);

/// Closes the account using [`CanCloseAccount::close`].
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct CloseAccount<T>(pub T);

#[derive(AccountSet, derive_where::DeriveWhere)]
#[derive_where(Clone, Debug, Copy)]
#[account_set(skip_default_idl, skip_default_cleanup)]
#[validate(extra_validation = self.validate())]
#[cleanup(
    generics = [],
    extra_cleanup = self.check_cleanup(ctx),
)]
#[cleanup(
    id = "normalize_rent",
    generics = [<'a, Funder> where Funder: CanFundRent],
    arg = NormalizeRent<&'a Funder>,
    extra_cleanup = self.normalize_rent(arg.0, ctx)
)]
#[cleanup(
    id = "normalize_rent_cached",
    arg = NormalizeRent<()>,
    generics = [],
    extra_cleanup = {
        let funder = ctx.get_funder().context("Missing `funder` in cache for `NormalizeRent`")?;
        self.normalize_rent(funder, ctx)
    },
)]
#[cleanup(
    id = "receive_rent",
    generics = [<'a, Funder> where Funder: CanFundRent],
    arg = ReceiveRent<&'a Funder>,
    extra_cleanup = self.receive_rent(arg.0, ctx)
)]
#[cleanup(
    id = "receive_rent_cached",
    arg = ReceiveRent<()>,
    generics = [],
    extra_cleanup = {
        let funder = ctx.get_funder().context("Missing `funder` in cache for `ReceiveRent`")?;
        self.receive_rent(funder, ctx)
    }
)]
#[cleanup(
    id = "refund_rent",
    generics = [<'a, Recipient> where Recipient: CanRecieveLamports],
    arg = RefundRent<&'a Recipient>,
    extra_cleanup = self.refund_rent(arg.0, ctx)
)]
#[cleanup(
    id = "refund_rent_cached",
    arg = RefundRent<()>,
    generics = [],
    extra_cleanup = {
        let recipient = ctx.get_recipient().context("Missing `recipient` in cache for `RefundRent`")?;
        self.refund_rent(recipient, ctx)
    }
)]
#[cleanup(
    id = "close_account",
    generics = [<'a, Recipient> where Recipient: CanRecieveLamports],
    arg = CloseAccount<&'a Recipient>,
    extra_cleanup = self.close(arg.0)
)]
#[cleanup(
    id = "close_account_cached",
    arg = CloseAccount<()>,
    generics = [],
    extra_cleanup = {
        let recipient = ctx.get_recipient().context("Missing `recipient` in cache for `CloseAccount`")?;
        self.close(recipient)
    }
)]
pub struct Account<T: ProgramAccount + UnsizedType + ?Sized> {
    #[single_account_set(
        skip_has_inner_type,
        skip_can_init_account,
        skip_has_seeds,
        skip_has_owner_program
    )]
    info: AccountInfo,
    #[account_set(skip = PhantomData)]
    phantom_t: PhantomData<T>,
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use super::*;
    use star_frame::idl::AccountSetToIdl;
    use star_frame_idl::account_set::IdlAccountSetDef;
    use star_frame_idl::IdlDefinition;

    impl<T: ProgramAccount + UnsizedType + ?Sized, A> AccountSetToIdl<A> for Account<T>
    where
        AccountInfo: AccountSetToIdl<A>,
        T: AccountToIdl,
    {
        fn account_set_to_idl(
            idl_definition: &mut IdlDefinition,
            arg: A,
        ) -> Result<IdlAccountSetDef> {
            let mut set = <AccountInfo>::account_set_to_idl(idl_definition, arg)?;
            set.single()?
                .program_accounts
                .push(T::account_to_idl(idl_definition)?);
            Ok(set)
        }
    }
}

impl<T> Account<T>
where
    T: ProgramAccount + UnsizedType + ?Sized,
{
    /// Validates the owner and the discriminant of the account.
    #[inline]
    pub fn validate(&self) -> Result<()> {
        T::validate_account_info(self)
    }

    pub fn data(&self) -> Result<SharedWrapper<'_, T::Ref<'_>>> {
        // If the account is writable, changes could have been made after AccountSetValidate has been run
        if self.is_writable() {
            self.validate()?;
        }
        SharedWrapper::new::<AccountDiscriminant<T>>(&self.info)
    }

    pub fn data_mut(&self) -> Result<ExclusiveWrapperTop<'_, AccountDiscriminant<T>, AccountInfo>> {
        // If the account is writable, changes could have been made after AccountSetValidate has been run
        if self.is_writable() {
            self.validate()?;
        }
        ExclusiveWrapper::new(&self.info)
    }
}

pub mod discriminant {
    use crate::unsize::{FromOwned, RawSliceAdvance};

    use super::*;
    #[derive(Debug)]
    pub struct AccountDiscriminant<T: UnsizedType + ProgramAccount + ?Sized>(T);

    unsafe impl<T> UnsizedType for AccountDiscriminant<T>
    where
        T: ProgramAccount + UnsizedType + ?Sized,
    {
        type Ref<'a> = T::Ref<'a>;
        type Mut<'a> = T::Mut<'a>;
        type Owned = T::Owned;
        const ZST_STATUS: bool = T::ZST_STATUS;

        fn mut_as_ref<'a>(m: &'a Self::Mut<'_>) -> Self::Ref<'a> {
            T::mut_as_ref(m)
        }

        fn get_ref<'a>(data: &mut &'a [u8]) -> Result<Self::Ref<'a>> {
            data.try_advance(size_of::<OwnerProgramDiscriminant<T>>())
                .with_context(|| {
                    format!(
                        "Failed to advance past discriminant of size {}",
                        size_of::<OwnerProgramDiscriminant<T>>()
                    )
                })?;
            T::get_ref(data)
        }

        unsafe fn get_mut<'a>(data: &mut *mut [u8]) -> Result<Self::Mut<'a>> {
            data.try_advance(size_of::<OwnerProgramDiscriminant<T>>())
                .with_context(|| {
                    format!(
                        "Failed to advance past discriminant of size {}",
                        size_of::<OwnerProgramDiscriminant<T>>()
                    )
                })?;
            unsafe { T::get_mut(data) }
        }

        fn owned(mut data: &[u8]) -> Result<Self::Owned> {
            data.try_advance(size_of::<OwnerProgramDiscriminant<T>>())
                .with_context(|| {
                    format!(
                        "Failed to advance past discriminant of size {}",
                        size_of::<OwnerProgramDiscriminant<T>>()
                    )
                })?;
            T::owned(data)
        }

        fn owned_from_ref(r: &Self::Ref<'_>) -> Result<Self::Owned> {
            T::owned_from_ref(r)
        }

        unsafe fn resize_notification(
            self_mut: &mut Self::Mut<'_>,
            source_ptr: *const (),
            change: isize,
        ) -> Result<()> {
            unsafe { T::resize_notification(self_mut, source_ptr, change) }
        }
    }

    unsafe impl<T> FromOwned for AccountDiscriminant<T>
    where
        T: ProgramAccount + UnsizedType + FromOwned + ?Sized,
    {
        fn byte_size(owned: &T::Owned) -> usize {
            T::byte_size(owned) + size_of::<OwnerProgramDiscriminant<T>>()
        }

        fn from_owned(owned: T::Owned, bytes: &mut &mut [u8]) -> Result<usize> {
            bytes
                .try_advance(size_of::<OwnerProgramDiscriminant<T>>())
                .with_context(|| {
                    format!(
                        "Failed to advance past discriminant during initialization of {}",
                        std::any::type_name::<T>()
                    )
                })?
                .copy_from_slice(bytes_of(&T::DISCRIMINANT));
            T::from_owned(owned, bytes).map(|size| size + size_of::<OwnerProgramDiscriminant<T>>())
        }
    }
    unsafe impl<T, I> UnsizedInit<I> for AccountDiscriminant<T>
    where
        T: UnsizedType + ?Sized + ProgramAccount + UnsizedInit<I>,
    {
        const INIT_BYTES: usize = T::INIT_BYTES + size_of::<OwnerProgramDiscriminant<T>>();

        unsafe fn init(bytes: &mut &mut [u8], arg: I) -> Result<()> {
            bytes
                .try_advance(size_of::<OwnerProgramDiscriminant<T>>())
                .with_context(|| {
                    format!(
                        "Failed to advance past discriminant during initialization of {}",
                        std::any::type_name::<T>()
                    )
                })?
                .copy_from_slice(bytes_of(&T::DISCRIMINANT));
            unsafe { T::init(bytes, arg) }
        }
    }
}
use discriminant::AccountDiscriminant;

impl<T: ProgramAccount + UnsizedType + ?Sized> HasInnerType for Account<T> {
    type Inner = T;
}

impl<T: ProgramAccount + UnsizedType + ?Sized> HasOwnerProgram for Account<T> {
    type OwnerProgram = T::OwnerProgram;
}

impl<T: ProgramAccount + UnsizedType + ?Sized> HasSeeds for Account<T>
where
    T: HasSeeds,
{
    type Seeds = T::Seeds;
}

impl<T: ProgramAccount + UnsizedType + ?Sized> CanInitAccount<()> for Account<T>
where
    T: UnsizedInit<DefaultInit>,
{
    fn init_account<const IF_NEEDED: bool>(
        &mut self,
        _arg: (),
        account_seeds: Option<Vec<&[u8]>>,
        ctx: &Context,
    ) -> Result<()> {
        self.init_account::<IF_NEEDED>((DefaultInit,), account_seeds, ctx)
    }
}

impl<T: ProgramAccount + UnsizedType + ?Sized, InitArg> CanInitAccount<(InitArg,)> for Account<T>
where
    T: UnsizedInit<InitArg>,
{
    fn init_account<const IF_NEEDED: bool>(
        &mut self,
        arg: (InitArg,),
        account_seeds: Option<Vec<&[u8]>>,
        ctx: &Context,
    ) -> Result<()> {
        let funder = ctx
            .get_funder()
            .context("Missing tagged `funder` for Account `init_account`")?;
        self.init_account::<IF_NEEDED>((arg.0, funder), account_seeds, ctx)
    }
}

impl<T: ProgramAccount + UnsizedType + ?Sized, InitArg, Funder> CanInitAccount<(InitArg, &Funder)>
    for Account<T>
where
    T: UnsizedInit<InitArg>,
    Funder: CanFundRent + ?Sized,
{
    fn init_account<const IF_NEEDED: bool>(
        &mut self,
        arg: (InitArg, &Funder),
        account_seeds: Option<Vec<&[u8]>>,
        ctx: &Context,
    ) -> Result<()> {
        if IF_NEEDED {
            let needs_init = self.owner_pubkey() == System::ID
                || self.account_data()?[..size_of::<OwnerProgramDiscriminant<T>>()]
                    .iter()
                    .all(|x| *x == 0);
            if !needs_init {
                return Ok(());
            }
        }
        self.check_writable()?;
        let (arg, funder) = arg;
        self.system_create_account(
            funder,
            T::OwnerProgram::ID,
            <AccountDiscriminant<T>>::INIT_BYTES,
            &account_seeds,
            ctx,
        )?;
        let mut data_bytes = self.account_data_mut()?;
        let mut data_bytes = &mut *data_bytes;
        unsafe {
            <AccountDiscriminant<T>>::init(&mut data_bytes, arg)?;
        }
        Ok(())
    }
}
