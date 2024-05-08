use crate::account_set::{SignedAccount, WritableAccount};
use crate::prelude::*;
use crate::serialize::ref_wrapper::{AsBytes, AsMutBytes};
use crate::serialize::unsize::resize::Resize;
use crate::util::*;
use advance::Advance;
use anyhow::bail;
use bytemuck::{bytes_of, from_bytes};
use derivative::Derivative;
use solana_program::entrypoint::MAX_PERMITTED_DATA_INCREASE;
use solana_program::program_memory::sol_memset;
use solana_program::system_program;
use star_frame::serialize::ref_wrapper::RefWrapper;
use std::cell::{Ref, RefMut};
use std::marker::PhantomData;
use std::mem::{size_of, size_of_val};
use std::slice::from_raw_parts_mut;

pub trait ProgramAccount {
    type OwnerProgram: StarFrameProgram;
    const DISCRIMINANT: <Self::OwnerProgram as StarFrameProgram>::AccountDiscriminant;

    fn account_data_size(&self) -> usize {
        size_of::<<Self::OwnerProgram as StarFrameProgram>::AccountDiscriminant>()
            + size_of_val(self)
    }
}

fn validate_data_account<T>(account: &DataAccount<T>, sys_calls: &impl SysCallCore) -> Result<()>
where
    T: ProgramAccount + UnsizedType + ?Sized,
{
    if account.info.owner != &T::OwnerProgram::program_id(sys_calls)? {
        bail!(ProgramError::IllegalOwner);
    }

    let data = account.info.try_borrow_data()?;
    if data.len() < size_of::<<T::OwnerProgram as StarFrameProgram>::AccountDiscriminant>() {
        bail!(ProgramError::InvalidAccountData);
    }
    let discriminant: &<T::OwnerProgram as StarFrameProgram>::AccountDiscriminant = from_bytes(
        &data[0..size_of::<<T::OwnerProgram as StarFrameProgram>::AccountDiscriminant>()],
    );
    if discriminant != &T::DISCRIMINANT {
        bail!(ProgramError::InvalidAccountData);
    }
    Ok(())
}

#[derive(Debug, Derivative)]
#[derivative(Copy(bound = ""), Clone(bound = ""))]
pub struct NormalizeRent<'a, 'info, F> {
    pub system_program: &'a Program<'info, SystemProgram>,
    pub funder: &'a F,
}

#[derive(Debug, Copy, Clone)]
pub struct RefundRent<'a, F> {
    pub recipient: &'a F,
}

#[derive(Debug, Copy, Clone)]
pub struct CloseAccount<'a, F> {
    pub recipient: &'a F,
}

#[derive(AccountSet, Debug)]
#[validate(extra_validation = validate_data_account(self, sys_calls))]
#[cleanup(extra_cleanup = self.check_cleanup(sys_calls))]
#[cleanup(
    id = "normalize_rent",
    generics = [<'a, F> where F: WritableAccount<'info> + SignedAccount<'info>],
    arg = NormalizeRent<'a, 'info, F>,
    extra_cleanup = self.normalize_rent(arg, sys_calls)
)]
#[cleanup(
    id = "refund_rent",
    generics = [<'a, F> where F: WritableAccount<'info>],
    arg = RefundRent<'a, F>,
    extra_cleanup = self.refund_rent(&arg, sys_calls)
)]
#[cleanup(
    id = "close_account",
    generics = [<'a, F> where F: WritableAccount<'info>],
    arg = CloseAccount<'a, F>,
    extra_cleanup = self.close(&arg)
)]
pub struct DataAccount<'info, T: ProgramAccount + UnsizedType + ?Sized> {
    info: AccountInfo<'info>,
    phantom_t: PhantomData<T>,
}

impl<'info, T> DataAccount<'info, T>
where
    T: ProgramAccount + UnsizedType + ?Sized,
{
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

    pub fn data<'a>(&'a self) -> Result<RefWrapper<AccountInfoRef<'a>, T::RefData>> {
        let r: Ref<'a, _> = self.info.try_borrow_data()?;
        Self::check_discriminant(&r)?;
        let r = try_map_ref(r, |bytes| {
            let bytes = &mut &**bytes;
            bytes.try_advance(size_of::<
                <T::OwnerProgram as StarFrameProgram>::AccountDiscriminant,
            >())
        })?;
        let account_info_ref = AccountInfoRef { r };
        T::from_bytes(account_info_ref).map(|ret| ret.ref_wrapper)
    }

    pub fn data_mut<'a>(
        &'a mut self,
    ) -> Result<RefWrapper<AccountInfoRefMut<'a, 'info, T::OwnerProgram>, T::RefData>> {
        let r: RefMut<'a, _> = self.info.try_borrow_mut_data()?;
        Self::check_discriminant(&r)?;
        let account_info_ref_mut = AccountInfoRefMut {
            account_info: &self.info,
            r,
            phantom: PhantomData,
        };
        T::from_bytes(account_info_ref_mut).map(|ret| ret.ref_wrapper)
    }

    /// Closes the account
    pub fn close(&mut self, arg: &CloseAccount<impl WritableAccount<'info>>) -> Result<()> {
        self.info.realloc(
            size_of::<<T::OwnerProgram as StarFrameProgram>::AccountDiscriminant>(),
            false,
        )?;
        self.info.try_borrow_mut_data()?.copy_from_slice(bytes_of(
            &<T::OwnerProgram as StarFrameProgram>::CLOSED_ACCOUNT_DISCRIMINANT,
        ));
        **arg.recipient.account_info().try_borrow_mut_lamports()? += self.info.lamports();
        **self.info.try_borrow_mut_lamports()? = 0;
        Ok(())
    }

    /// Closes the account by reallocing and transfering. This is the same as calling `close` but
    /// not abusable and harder for indexer detection.
    pub fn close_full(&mut self, arg: &CloseAccount<impl WritableAccount<'info>>) -> Result<()> {
        self.info.realloc(
            size_of::<<T::OwnerProgram as StarFrameProgram>::AccountDiscriminant>(),
            false,
        )?;
        self.info.try_borrow_mut_data()?.copy_from_slice(bytes_of(
            &<T::OwnerProgram as StarFrameProgram>::CLOSED_ACCOUNT_DISCRIMINANT,
        ));
        **arg.recipient.account_info().try_borrow_mut_lamports()? += self.info.lamports();
        **self.info.try_borrow_mut_lamports()? = 0;
        self.info.realloc(0, false)?;
        self.info.assign(&system_program::ID);
        Ok(())
    }

    pub fn normalize_rent(
        &mut self,
        arg: NormalizeRent<'_, 'info, impl WritableAccount<'info> + SignedAccount<'info>>,
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<()> {
        normalize_rent(self, arg.funder, arg.system_program, sys_calls)
    }

    pub fn refund_rent(
        &mut self,
        arg: &RefundRent<impl WritableAccount<'info>>,
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<()> {
        refund_rent(self, arg.recipient, sys_calls)
    }

    pub fn check_cleanup(&self, sys_calls: &mut impl SysCallCore) -> Result<()> {
        #[cfg(feature = "cleanup_rent_warning")]
        {
            use anyhow::Context;
            use std::cmp::Ordering;
            if self.is_writable() {
                let rent = sys_calls.get_rent()?;
                let lamports = self.account_info().lamports();
                let data_len = self.account_info().data_len();
                let rent_lamports = rent.minimum_balance(data_len);
                match rent_lamports.cmp(&lamports) {
                    Ordering::Greater => {
                        // is this more descriptive than just letting the runtime error out?
                        return Err(anyhow::anyhow!(ProgramError::AccountNotRentExempt))
                            .with_context(|| {
                                format!(
                                    "{} was left with less lamports than required by rent",
                                    self.key()
                                )
                            });
                    }
                    Ordering::Less => {
                        msg!(
                            "{} was left with more lamports than required by rent",
                            self.key()
                        );
                    }
                    Ordering::Equal => {}
                }
            }
        }
        Ok(())
    }
}

impl<'info, T> SingleAccountSet<'info> for DataAccount<'info, T>
where
    T: ProgramAccount + UnsizedType + ?Sized,
{
    fn account_info(&self) -> &AccountInfo<'info> {
        &self.info
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
            account_info_realloc(new_byte_len, true, &mut self.r, original_data_len)
                .map_err(Into::into)
        }
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
