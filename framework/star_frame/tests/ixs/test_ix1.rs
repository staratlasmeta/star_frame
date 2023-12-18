use advance::AdvanceArray;
use bytemuck::{from_bytes, Pod, Zeroable};
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;
use star_frame::account_set::AccountSet;
use star_frame::idl::ty::TypeToIdl;
use star_frame::idl::{AccountSetToIdl, InstructionToIdl};
use star_frame::impls::option::Remaining;
use star_frame::instruction::{FrameworkInstruction, ToBytes};
use star_frame::sys_calls::SysCallInvoke;
use star_frame_idl::instruction::IdlInstructionDef;
use star_frame_idl::ty::{IdlField, IdlTypeDef};
use star_frame_idl::IdlDefinition;
use std::mem::size_of;

#[derive(Pod, Copy, Clone, Zeroable)]
#[repr(C, packed)]
pub struct TestInstruction1 {
    pub val: u32,
    pub val2: u64,
    pub val3: i8,
}

impl<'a> ToBytes for &'a TestInstruction1 {
    fn to_bytes(self, output: &mut &mut [u8]) -> star_frame::Result<()> {
        let output_bytes: &mut [_; size_of::<TestInstruction1>()] = output.try_advance_array()?;
        output_bytes.copy_from_slice(bytemuck::bytes_of(self));
        Ok(())
    }
}

impl<'a> FrameworkInstruction<'a> for &'a TestInstruction1 {
    type DecodeArg = i8;
    type ValidateArg = u64;
    type RunArg = &'a i8;
    type CleanupArg = (u32, u64);
    type ReturnType = ();
    type Accounts<'b, 'info> = TestInstruction1Accounts<'b, 'info> where 'info: 'b;

    fn from_bytes_framework(mut bytes: &'a [u8]) -> star_frame::Result<Self> {
        let bytes: &[_; size_of::<TestInstruction1>()] = bytes.try_advance_array()?;
        Ok(from_bytes(bytes))
    }

    fn split_to_args(
        self,
    ) -> (
        Self::DecodeArg,
        Self::ValidateArg,
        Self::RunArg,
        Self::CleanupArg,
    ) {
        (self.val3, self.val2, &self.val3, (self.val, self.val2))
    }

    fn run_instruction(
        _run_arg: Self::RunArg,
        _program_id: &Pubkey,
        _account_set: &Self::Accounts<'_, '_>,
        _sys_calls: &mut impl SysCallInvoke,
    ) -> star_frame::Result<Self::ReturnType> {
        todo!()
    }
}

#[automatically_derived]
impl<'a> InstructionToIdl<'a, ()> for &'a TestInstruction1 {
    fn instruction_to_idl(
        idl_definition: &mut IdlDefinition,
        arg: (),
    ) -> star_frame::Result<IdlInstructionDef> {
        let val = <u32 as TypeToIdl>::type_to_idl(idl_definition)?;
        let val2 = <u64 as TypeToIdl>::type_to_idl(idl_definition)?;
        let val3 = <i8 as TypeToIdl>::type_to_idl(idl_definition)?;
        Ok(IdlInstructionDef {
            account_set: <Self as FrameworkInstruction<'a>>::Accounts::account_set_to_idl(
                idl_definition,
                (),
            )?,
            data: IdlTypeDef::Struct(vec![
                IdlField {
                    name: "val".to_string(),
                    description: "First Value".to_string(),
                    path_id: "val".to_string(),
                    type_def: val,
                    extension_fields: Default::default(),
                },
                IdlField {
                    name: "val 2".to_string(),
                    description: "The second Value".to_string(),
                    path_id: "val2".to_string(),
                    type_def: val2,
                    extension_fields: Default::default(),
                },
                IdlField {
                    name: "val 3".to_string(),
                    description: "The third value".to_string(),
                    path_id: "val3".to_string(),
                    type_def: val3,
                    extension_fields: Default::default(),
                },
            ]),
        })
    }
}

/// Hello
/// Cioi
#[derive(AccountSet)]
#[decode(arg = i8)]
#[validate(arg = u64)]
#[cleanup(arg = (u32, u64))]
pub struct TestInstruction1Accounts<'a, 'info>
where
    'info: 'a,
{
    pub account1: &'a AccountInfo<'info>,
    #[decode(arg = Remaining(()))]
    pub account2: Option<&'a AccountInfo<'info>>,
}

#[derive(AccountSet)]
pub struct AccountStuff {}
