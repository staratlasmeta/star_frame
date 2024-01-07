use crate::account_set::{
    AccountSet, AccountSetCleanup, AccountSetDecode, AccountSetValidate, SingleAccountSet,
};
use crate::anchor_replacement::{AnchorValidateArgs, ANCHOR_CLOSED_ACCOUNT_DISCRIMINATOR};
use crate::program::Program;
use crate::program_account::ProgramAccount;
use crate::sys_calls::{SysCallCore, SysCallInvoke};
use crate::Result;
use advance::AdvanceArray;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::AccountInfo;
use solana_program::instruction::AccountMeta;
use solana_program::program_error::ProgramError;
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct Account<'info, T>
where
    T: BorshSerialize + BorshDeserialize + ProgramAccount,
    T::OwnerProgram: Program<InstructionDiscriminant = [u8; 8]>,
{
    info: AccountInfo<'info>,
    data: Option<T>,
}
impl<'info, T> Account<'info, T>
where
    T: BorshSerialize + BorshDeserialize + ProgramAccount,
    T::OwnerProgram: Program<InstructionDiscriminant = [u8; 8]>,
{
    pub fn new(info: AccountInfo<'info>, data: T) -> Self {
        Self {
            info,
            data: Some(data),
        }
    }

    pub fn try_from(info: AccountInfo<'info>, runtime: impl SysCallCore) -> Result<Self> {
        if info.owner != T::OwnerProgram::program_id().find_network(runtime.current_network())? {
            return Err(ProgramError::IncorrectProgramId);
        }
        let data = Self::check_data(&info.info_data_bytes()?)?;
        Ok(Self::new(info, data))
    }

    fn check_data(data: &impl Deref<Target = [u8]>) -> Result<T> {
        let mut data: &[u8] = data.as_ref();
        let discriminant: &[u8; 8] = data.try_advance_array()?;
        if discriminant != &T::discriminant() {
            Err(ProgramError::InvalidAccountData)
        } else {
            Ok(T::deserialize(&mut data)?)
        }
    }

    pub fn reload(&mut self) -> Result<()> {
        self.data = Some(Self::check_data(&self.info.info_data_bytes()?)?);
        Ok(())
    }

    pub fn into_inner(self) -> T {
        self.data.unwrap()
    }

    pub fn set_inner(&mut self, data: T) {
        self.data = Some(data);
    }
}
impl<'info, T> AccountSet<'info> for Account<'info, T>
where
    T: BorshSerialize + BorshDeserialize + ProgramAccount,
    T::OwnerProgram: Program<InstructionDiscriminant = [u8; 8]>,
{
    fn try_to_accounts<'a, E>(
        &'a self,
        mut add_account: impl FnMut(&'a AccountInfo<'info>) -> Result<(), E>,
    ) -> Result<(), E>
    where
        'info: 'a,
    {
        add_account(&self.info)
    }

    fn to_account_metas(&self, mut add_account_meta: impl FnMut(AccountMeta)) {
        add_account_meta(self.info.account_meta());
    }
}
impl<'info, T> SingleAccountSet<'info> for Account<'info, T>
where
    T: BorshSerialize + BorshDeserialize + ProgramAccount,
    T::OwnerProgram: Program<InstructionDiscriminant = [u8; 8]>,
{
    fn account_info(&self) -> &AccountInfo<'info> {
        &self.info
    }
}
impl<'a, 'info, T> AccountSetDecode<'a, 'info, ()> for Account<'info, T>
where
    T: BorshSerialize + BorshDeserialize + ProgramAccount,
    T::OwnerProgram: Program<InstructionDiscriminant = [u8; 8]>,
{
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo<'info>],
        decode_input: (),
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<Self> {
        Ok(Self {
            info: AccountInfo::decode_accounts(accounts, decode_input, sys_calls)?,
            data: None,
        })
    }
}
impl<'a, 'info, T> AccountSetValidate<'info, AnchorValidateArgs<'a, 'info>> for Account<'info, T>
where
    T: BorshSerialize + BorshDeserialize + ProgramAccount,
    T::OwnerProgram: Program<InstructionDiscriminant = [u8; 8]>,
{
    fn validate_accounts(
        &mut self,
        validate_input: AnchorValidateArgs<'a, 'info>,
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<(), ProgramError> {
        validate_input.validate(self, sys_calls, T::discriminant())?;
        self.data = Some(Self::check_data(&self.info.info_data_bytes()?)?);

        self.info.validate_accounts((), sys_calls)
    }
}
impl<'a, 'info, T> AccountSetCleanup<'info, AnchorValidateArgs<'a, 'info>> for Account<'info, T>
where
    T: BorshSerialize + BorshDeserialize + ProgramAccount,
    T::OwnerProgram: Program<InstructionDiscriminant = [u8; 8]>,
{
    fn cleanup_accounts(
        &mut self,
        cleanup_input: AnchorValidateArgs<'a, 'info>,
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<()> {
        let mut data_bytes = self.info.info_data_bytes_mut()?;
        self.data
            .take()
            .unwrap()
            .serialize(&mut &mut data_bytes[8..])?;
        if let Some(close_to) = cleanup_input.close {
            let mut self_lamports = self.info.lamports.borrow_mut();
            let mut close_to_lamports = close_to.lamports.borrow_mut();
            **close_to_lamports += **self_lamports;
            **self_lamports = 0;
            data_bytes[..8].copy_from_slice(&ANCHOR_CLOSED_ACCOUNT_DISCRIMINATOR);
        }
        drop(data_bytes);

        self.info.cleanup_accounts((), sys_calls)
    }
}
