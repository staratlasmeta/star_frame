use crate::account_set::{SignedAccount, WritableAccount};
use crate::prelude::*;
use anyhow::bail;
use bytemuck::{bytes_of, from_bytes, from_bytes_mut};
use derivative::Derivative;
use solana_program::entrypoint::MAX_PERMITTED_DATA_INCREASE;
use solana_program::system_instruction::MAX_PERMITTED_DATA_LENGTH;
use std::cell::{Ref, RefMut};
use std::marker::PhantomData;
use std::mem::{size_of, size_of_val};
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

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
pub struct DataAccount<'info, T: ProgramAccount + UnsizedType + ?Sized> {
    info: AccountInfo<'info>,
    phantom_t: PhantomData<T>,
    #[account_set(skip = false)]
    closed: bool,
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

    pub fn data<'a>(&'a self) -> Result<DataRef<'a, T>> {
        let r: Ref<'a, _> = self.info.try_borrow_data()?;
        Self::check_discriminant(&r)?;
        let mut r_ptr: Option<NonNull<[u8]>> = None;
        let r = Ref::map(r, |bytes| {
            r_ptr = Some(NonNull::from(&**bytes));
            from_bytes(&bytes[0..0])
        });
        let data: T::Ref<'a> = T::Ref::from_bytes(&mut unsafe {
            &*r_ptr.unwrap().as_ptr().byte_add(size_of::<
                <T::OwnerProgram as StarFrameProgram>::AccountDiscriminant,
            >())
        })?;
        Ok(DataRef { _r: r, data })
    }

    pub fn data_mut<'a>(&'a mut self) -> Result<DataRefMut<'a, T>> {
        let original_data_len = unsafe { self.info.original_data_len() };
        let r: RefMut<'a, _> = self.info.try_borrow_mut_data()?;
        Self::check_discriminant(&r)?;
        let mut r_ptr: Option<NonNull<[u8]>> = None;
        let r = RefMut::map(r, |bytes| {
            r_ptr = Some(NonNull::from(&**bytes));
            from_bytes_mut(&mut bytes[0..0])
        });
        let r_ptr = r_ptr.unwrap();
        let mut data_ptr = unsafe {
            NonNull::new(r_ptr.as_ptr().byte_add(size_of::<
                <T::OwnerProgram as StarFrameProgram>::AccountDiscriminant,
            >()))
            .unwrap()
        };
        let data_len_ptr = unsafe { r_ptr.as_ptr().byte_sub(8).cast::<PackedValue<u64>>() };
        Ok(DataRefMut {
            data: T::RefMut::from_bytes_mut(
                &mut unsafe { data_ptr.as_mut() },
                move |new_len, _| {
                    let new_len = new_len
                        + size_of::<<T::OwnerProgram as StarFrameProgram>::AccountDiscriminant>();
                    if new_len > original_data_len + MAX_PERMITTED_DATA_INCREASE
                        || new_len as u64 > MAX_PERMITTED_DATA_LENGTH
                    {
                        bail!(ProgramError::InvalidRealloc)
                    }
                    unsafe { data_len_ptr.write(PackedValue(new_len as u64)) };
                    Ok(data_ptr.cast())
                },
            )?,
            _r: r,
        })
    }

    /// Closes the account
    pub fn close(&mut self) -> Result<()> {
        self.info.realloc(
            size_of::<<T::OwnerProgram as StarFrameProgram>::AccountDiscriminant>(),
            false,
        )?;
        self.info.try_borrow_mut_data()?.copy_from_slice(bytes_of(
            &<T::OwnerProgram as StarFrameProgram>::CLOSED_ACCOUNT_DISCRIMINANT,
        ));
        self.closed = true;
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
pub struct DataRef<'a, T>
where
    T: 'a + ProgramAccount + UnsizedType + ?Sized,
{
    data: T::Ref<'a>,
    _r: Ref<'a, [u8; 0]>,
}

impl<'a, T> Deref for DataRef<'a, T>
where
    T: 'a + ProgramAccount + UnsizedType + ?Sized,
{
    type Target = T::Ref<'a>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

#[derive(Debug)]
pub struct DataRefMut<'a, T>
where
    T: 'a + ProgramAccount + UnsizedType + ?Sized,
{
    data: T::RefMut<'a>,
    _r: RefMut<'a, [u8; 0]>,
}

impl<'a, T> Deref for DataRefMut<'a, T>
where
    T: 'a + ProgramAccount + UnsizedType + ?Sized,
{
    type Target = T::RefMut<'a>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'a, T> DerefMut for DataRefMut<'a, T>
where
    T: 'a + ProgramAccount + UnsizedType + ?Sized,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
