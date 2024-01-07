use advance::{Advance, AdvanceArray};
use solana_program::account_info::AccountInfo;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use star_frame::account_set::mutable::Writable;
use star_frame::account_set::signer::Signer;
use star_frame::account_set::AccountSet;
use star_frame::idl::ty::TypeToIdl;
use star_frame::idl::{AccountSetToIdl, InstructionToIdl};
use star_frame::instruction::{FrameworkInstruction, FrameworkSerialize};
use star_frame::sys_calls::SysCallInvoke;
use star_frame::Result;
use star_frame_idl::instruction::IdlInstructionDef;
use star_frame_idl::ty::{IdlField, IdlTypeDef};
use star_frame_idl::IdlDefinition;
use std::mem::size_of;
use std::ptr;

#[repr(C, packed)]
pub struct TestInstruction2 {
    pub val: u32,
    pub val2: u64,
    pub val3: Pubkey,
    pub remaining: [u8],
}

impl<'a> FrameworkSerialize for &'a TestInstruction2 {
    fn to_bytes(self, output: &mut &mut [u8]) -> star_frame::Result<()> {
        *output.try_advance_array()? = self.val.to_le_bytes();
        *output.try_advance_array()? = self.val2.to_le_bytes();
        *output.try_advance_array()? = self.val3.to_bytes();
        output
            .try_advance(self.remaining.len())?
            .copy_from_slice(&self.remaining);
        Ok(())
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let remaining_length = bytes
            .len()
            .checked_sub(size_of::<u32>() + size_of::<u64>() + size_of::<Pubkey>())
            .ok_or(ProgramError::InvalidInstructionData)?;
        unsafe {
            Ok(&*ptr::from_raw_parts(
                bytes.as_ptr().cast(),
                remaining_length,
            ))
        }
    }
}

#[automatically_derived]
impl<'a> FrameworkInstruction<'a> for &'a TestInstruction2 {
    type DecodeArg = ();
    type ValidateArg = ();
    type RunArg = &'a TestInstruction2;
    type CleanupArg = ();
    type ReturnType = ();
    type Accounts<'b, 'info> = TestInstruction2Accounts<'b, 'info> where 'info: 'b;
    //
    // fn from_bytes_framework(bytes: &'a [u8]) -> Result<Self> {
    //     let remaining_length = bytes
    //         .len()
    //         .checked_sub(size_of::<u32>() + size_of::<u64>() + size_of::<Pubkey>())
    //         .ok_or(ProgramError::InvalidInstructionData)?;
    //     unsafe {
    //         Ok(&*ptr::from_raw_parts(
    //             bytes.as_ptr().cast(),
    //             remaining_length,
    //         ))
    //     }
    // }

    fn split_to_args(
        self,
    ) -> (
        Self::DecodeArg,
        Self::ValidateArg,
        Self::RunArg,
        Self::CleanupArg,
    ) {
        ((), (), self, ())
    }

    fn run_instruction(
        run_arg: Self::RunArg,
        program_id: &Pubkey,
        account_set: &Self::Accounts<'_, '_>,
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<Self::ReturnType> {
        todo!()
    }
}
#[automatically_derived]
impl<'a> InstructionToIdl<'a, ()> for &'a TestInstruction2 {
    fn instruction_to_idl(
        idl_definition: &mut IdlDefinition,
        arg: (),
    ) -> Result<IdlInstructionDef> {
        let val = <u32 as TypeToIdl>::type_to_idl(idl_definition)?;
        let val2 = <u64 as TypeToIdl>::type_to_idl(idl_definition)?;
        let val3 = <Pubkey as TypeToIdl>::type_to_idl(idl_definition)?;
        Ok(IdlInstructionDef {
            account_set: <Self as FrameworkInstruction<'a>>::Accounts::account_set_to_idl(
                idl_definition,
                (),
            )?,
            data: IdlTypeDef::Struct(vec![
                IdlField {
                    name: "val".to_string(),
                    description: "The first value".to_string(),
                    path_id: "val".to_string(),
                    type_def: val,
                    extension_fields: Default::default(),
                },
                IdlField {
                    name: "val2".to_string(),
                    description: "The second value".to_string(),
                    path_id: "val2".to_string(),
                    type_def: val2,
                    extension_fields: Default::default(),
                },
                IdlField {
                    name: "val3".to_string(),
                    description: "The third value".to_string(),
                    path_id: "val3".to_string(),
                    type_def: val3,
                    extension_fields: Default::default(),
                },
            ]),
        })
    }
}

#[derive(AccountSet)]
pub struct TestInstruction2Accounts<'a, 'info>
where
    'info: 'a,
{
    pub signer: Signer<Writable<&'a AccountInfo<'info>>>,
}
