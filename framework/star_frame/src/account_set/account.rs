use crate::prelude::*;
use crate::util::*;
use advance::Advance;
use anyhow::{bail, Context};
use bytemuck::{bytes_of, from_bytes};
use derivative::Derivative;
use solana_program::entrypoint::MAX_PERMITTED_DATA_INCREASE;
use solana_program::program_memory::sol_memset;
pub use star_frame_proc::ProgramAccount;
use std::cell::{Ref, RefMut};
use std::marker::PhantomData;
use std::mem::size_of;
use std::slice::from_raw_parts_mut;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct NormalizeRent<T>(pub T);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct RefundRent<T>(pub T);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct CloseAccount<T>(pub T);

#[derive(AccountSet, Debug)]
#[account_set(skip_default_idl, skip_default_cleanup)]
#[validate(extra_validation = self.validate())]
#[cleanup(
    generics = [],
    extra_cleanup = self.check_cleanup(syscalls),
)]
#[cleanup(
    id = "normalize_rent",
    generics = [<'a, Funder> where Funder: WritableAccount<'info> + SignedAccount<'info>],
    arg = NormalizeRent<&'a Funder>,
    extra_cleanup = self.normalize_rent(arg.0, syscalls)
)]
#[cleanup(
    id = "normalize_rent_cached",
    arg = NormalizeRent<()>,
    generics = [],
    extra_cleanup = {
        let funder = syscalls.get_funder().context("Missing `funder` in cache for `NormalizeRent`")?;
        self.normalize_rent(funder, syscalls)
    },
)]
#[cleanup(
    id = "refund_rent",
    generics = [<'a, Recipient> where Recipient: WritableAccount<'info>],
    arg = RefundRent<&'a Recipient>,
    extra_cleanup = self.refund_rent(arg.0, syscalls)
)]
#[cleanup(
    id = "refund_rent_cached",
    arg = RefundRent<()>,
    generics = [],
    extra_cleanup = {
        let recipient = syscalls.get_recipient().context("Missing `recipient` in cache for `RefundRent`")?;
        self.refund_rent(recipient, syscalls)
    }
)]
#[cleanup(
    id = "close_account",
    generics = [<'a, Recipient> where Recipient: WritableAccount<'info>],
    arg = CloseAccount<&'a Recipient>,
    extra_cleanup = self.close(arg.0)
)]
#[cleanup(
    id = "close_account_cached",
    arg = CloseAccount<()>,
    generics = [],
    extra_cleanup = {
        let recipient = syscalls.get_recipient().context("Missing `recipient` in cache for `CloseAccount`")?;
        self.close(recipient)
    }
)]
pub struct Account<'info, T: ProgramAccount + UnsizedType + ?Sized> {
    #[single_account_set(
        skip_has_program_account,
        skip_can_init_account,
        skip_has_seeds,
        skip_has_owner_program
    )]
    info: AccountInfo<'info>,
    #[account_set(skip = PhantomData)]
    phantom_t: PhantomData<T>,
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use super::*;
    use star_frame::idl::AccountSetToIdl;
    use star_frame_idl::account_set::IdlAccountSetDef;
    use star_frame_idl::IdlDefinition;

    impl<'info, T: ProgramAccount + UnsizedType + ?Sized, A> AccountSetToIdl<'info, A>
        for Account<'info, T>
    where
        AccountInfo<'info>: AccountSetToIdl<'info, A>,
        T: AccountToIdl,
    {
        fn account_set_to_idl(
            idl_definition: &mut IdlDefinition,
            arg: A,
        ) -> Result<IdlAccountSetDef> {
            let mut set = <AccountInfo<'info>>::account_set_to_idl(idl_definition, arg)?;
            set.single()?
                .program_accounts
                .push(T::account_to_idl(idl_definition)?);
            Ok(set)
        }
    }
}

impl<'info, T> Account<'info, T>
where
    T: ProgramAccount + UnsizedType + ?Sized,
{
    /// Validates the owner and the discriminant of the account.
    #[inline]
    pub fn validate(&self) -> Result<()> {
        if self.owner() != &T::OwnerProgram::ID {
            bail!(ProgramError::IllegalOwner);
        }
        let data = self.info_data_bytes()?;
        Self::check_discriminant(&data)?;
        Ok(())
    }

    #[inline]
    fn check_discriminant(bytes: &[u8]) -> Result<()> {
        if bytes.len() < size_of::<<T::OwnerProgram as StarFrameProgram>::AccountDiscriminant>()
            || from_bytes::<PackedValue<<T::OwnerProgram as StarFrameProgram>::AccountDiscriminant>>(
                &bytes[..size_of::<<T::OwnerProgram as StarFrameProgram>::AccountDiscriminant>()],
            ) != &PackedValue(T::DISCRIMINANT)
        {
            bail!(ProgramError::InvalidAccountData)
        }
        Ok(())
    }

    #[inline]
    pub fn data<'a>(&'a self) -> Result<RefWrapper<AccountInfoRef<'a>, T::RefData>> {
        let r: Ref<'a, _> = self.info_data_bytes()?;
        Self::check_discriminant(&r)?;
        let r = try_map_ref(r, |bytes| {
            let bytes = &mut &*bytes;
            bytes.try_advance(size_of::<
                <T::OwnerProgram as StarFrameProgram>::AccountDiscriminant,
            >())?;
            Result::<_>::Ok(*bytes)
        })?;
        let account_info_ref = AccountInfoRef { r };
        T::from_bytes(account_info_ref).map(|ret| ret.ref_wrapper)
    }

    #[inline]
    pub fn data_mut<'a>(
        &'a mut self,
    ) -> Result<RefWrapper<AccountInfoRefMut<'a, 'info, T::OwnerProgram>, T::RefData>> {
        let r: RefMut<'a, _> = self.info_data_bytes_mut()?;
        Self::check_discriminant(&r)?;
        let account_info_ref_mut = AccountInfoRefMut {
            account_info: &self.info,
            r,
            phantom: PhantomData,
        };
        T::from_bytes(account_info_ref_mut).map(|ret| ret.ref_wrapper)
    }
}

impl<'info, T: ProgramAccount + UnsizedType + ?Sized> HasProgramAccount for Account<'info, T> {
    type ProgramAccount = T;
}

impl<'info, T: ProgramAccount + UnsizedType + ?Sized> HasOwnerProgram for Account<'info, T> {
    type OwnerProgram = T::OwnerProgram;
}

impl<'info, T: ProgramAccount + UnsizedType + ?Sized> HasSeeds for Account<'info, T>
where
    T: HasSeeds,
{
    type Seeds = T::Seeds;
}

impl<'info, T: ProgramAccount + UnsizedType + ?Sized> CanInitAccount<'info, ()>
    for Account<'info, T>
where
    T: UnsizedInit<DefaultInit>,
{
    fn init_account<const IF_NEEDED: bool>(
        &mut self,
        _arg: (),
        account_seeds: Option<Vec<&[u8]>>,
        syscalls: &impl SyscallInvoke<'info>,
    ) -> Result<()> {
        self.init_account::<IF_NEEDED>((DefaultInit,), account_seeds, syscalls)
    }
}

impl<'info, T: ProgramAccount + UnsizedType + ?Sized, InitArg> CanInitAccount<'info, (InitArg,)>
    for Account<'info, T>
where
    T: UnsizedInit<InitArg>,
{
    fn init_account<const IF_NEEDED: bool>(
        &mut self,
        arg: (InitArg,),
        account_seeds: Option<Vec<&[u8]>>,
        syscalls: &impl SyscallInvoke<'info>,
    ) -> Result<()> {
        let funder = syscalls
            .get_funder()
            .context("Missing tagged `funder` for DataAccount `init_account`")?;
        self.init_account::<IF_NEEDED>((arg.0, funder), account_seeds, syscalls)
    }
}

impl<'info, T: ProgramAccount + UnsizedType + ?Sized, InitArg, Funder>
    CanInitAccount<'info, (InitArg, &Funder)> for Account<'info, T>
where
    T: UnsizedInit<InitArg>,
    Funder: SignedAccount<'info> + WritableAccount<'info>,
{
    fn init_account<const IF_NEEDED: bool>(
        &mut self,
        arg: (InitArg, &Funder),
        account_seeds: Option<Vec<&[u8]>>,
        syscalls: &impl SyscallInvoke<'info>,
    ) -> Result<()> {
        if IF_NEEDED {
            let needs_init = self.owner() == &System::ID
                || self.info_data_bytes()?
                    [..size_of::<<T::OwnerProgram as StarFrameProgram>::AccountDiscriminant>()]
                    .iter()
                    .all(|x| *x == 0);
            if !needs_init {
                return Ok(());
            }
        }
        self.check_writable()?;
        let (arg, funder) = arg;
        let size =
            T::INIT_BYTES + size_of::<<T::OwnerProgram as StarFrameProgram>::AccountDiscriminant>();
        self.system_create_account(funder, T::OwnerProgram::ID, size, &account_seeds, syscalls)?;
        {
            let mut data_bytes = self.info_data_bytes_mut()?;
            let mut data_bytes = &mut **data_bytes;

            data_bytes
                .try_advance(size_of::<
                    <T::OwnerProgram as StarFrameProgram>::AccountDiscriminant,
                >())?
                .copy_from_slice(bytes_of(&T::DISCRIMINANT));
            let data_bytes = data_bytes.try_advance(T::INIT_BYTES)?;
            sol_memset(data_bytes, 0, data_bytes.len());
            unsafe {
                T::init(data_bytes, arg)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct AccountInfoRef<'a> {
    pub(crate) r: Ref<'a, [u8]>,
}
unsafe impl<'a> AsBytes for AccountInfoRef<'a> {
    fn as_bytes(&self) -> Result<&[u8]> {
        Ok(self.r.as_ref())
    }
}
impl<'a> Clone for AccountInfoRef<'a> {
    fn clone(&self) -> Self {
        Self {
            r: Ref::clone(&self.r),
        }
    }
}

#[derive(Derivative)]
#[derivative(Debug(bound = ""))]
pub struct AccountInfoRefMut<'a, 'info, P: StarFrameProgram> {
    pub(crate) account_info: &'a AccountInfo<'info>,
    pub(crate) r: RefMut<'a, &'info mut [u8]>,
    pub(crate) phantom: PhantomData<fn() -> P>,
}
unsafe impl<'a, 'info, P: StarFrameProgram> AsBytes for AccountInfoRefMut<'a, 'info, P> {
    fn as_bytes(&self) -> Result<&[u8]> {
        let mut bytes = &**self.r;
        bytes.try_advance(size_of::<P::AccountDiscriminant>())?;
        Ok(bytes)
    }
}
unsafe impl<'a, 'info, P: StarFrameProgram> AsMutBytes for AccountInfoRefMut<'a, 'info, P> {
    fn as_mut_bytes(&mut self) -> Result<&mut [u8]> {
        let mut bytes = &mut **self.r;
        bytes.try_advance(size_of::<P::AccountDiscriminant>())?;
        Ok(bytes)
    }
}
unsafe impl<'a, 'info, P: StarFrameProgram, M> Resize<M> for AccountInfoRefMut<'a, 'info, P> {
    unsafe fn resize(&mut self, new_byte_len: usize, _new_meta: M) -> Result<()> {
        let original_data_len = unsafe { self.account_info.original_data_len() };
        unsafe {
            account_info_realloc(
                new_byte_len + size_of::<P::AccountDiscriminant>(),
                true,
                &mut self.r,
                original_data_len,
            )
            .map_err(Into::into)
        }
    }

    unsafe fn set_meta(&mut self, _new_meta: M) -> Result<()> {
        Ok(())
    }
}
/// Copied code from solana
unsafe fn account_info_realloc(
    new_len: usize,
    zero_init: bool,
    data: &mut RefMut<&mut [u8]>,
    original_data_len: usize,
) -> Result<(), ProgramError> {
    let old_len = data.len();

    // Return early if length hasn't changed
    if new_len == old_len {
        return Ok(());
    }

    // Return early if the length increase from the original serialized data
    // length is too large and would result in an out of bounds allocation.
    if new_len.saturating_sub(original_data_len) > MAX_PERMITTED_DATA_INCREASE {
        return Err(ProgramError::InvalidRealloc);
    }

    // realloc
    #[allow(clippy::cast_ptr_alignment)]
    unsafe {
        let data_ptr = data.as_mut_ptr();

        // First set new length in the serialized data

        *(data_ptr.offset(-8).cast::<u64>()) = new_len as u64;

        // Then recreate the local slice with the new length
        **data = from_raw_parts_mut(data_ptr, new_len);
    }

    if zero_init {
        let len_increase = new_len.saturating_sub(old_len);
        if len_increase > 0 {
            sol_memset(&mut data[old_len..], 0, len_increase);
        }
    }

    Ok(())
}
