pub mod un_callable;

pub use star_frame_proc::FrameworkInstruction;

use crate::account_set::{AccountSetCleanup, AccountSetDecode, AccountSetValidate};
use crate::serialize::FrameworkFromBytes;
use crate::sys_calls::{SysCallInvoke, SysCalls};
use crate::Result;
use bytemuck::{Pod, Zeroable};
use solana_program::account_info::AccountInfo;
use solana_program::program::MAX_RETURN_DATA;
use solana_program::pubkey::Pubkey;
use star_frame::serialize::FrameworkSerialize;
pub use star_frame_proc::InstructionSet;

/// A set of instructions that can be used as input to a program.
pub trait InstructionSet<'a> {
    /// The discriminant type used by this program's accounts.
    type Discriminant: Pod;

    /// Handles the instruction obtained from [`InstructionSet::from_bytes`].
    fn handle_ix(
        self,
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        sys_calls: &mut impl SysCalls,
    ) -> Result<()>;
}

#[derive(Pod, Zeroable, Copy, Clone, Align1)]
#[repr(C)]
struct Instruction1 {}
impl Instruction for Instruction1 {
    fn run_ix_from_raw(
        self,
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        sys_calls: &mut impl SysCalls,
    ) -> Result<()> {
        todo!()
    }
}

#[derive(Pod, Zeroable, Copy, Clone, Align1)]
#[repr(C)]
struct Instruction2 {}
impl Instruction for Instruction2 {
    fn run_ix_from_raw(
        self,
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        sys_calls: &mut impl SysCalls,
    ) -> Result<()> {
        todo!()
    }
}

// #[instruction_set]
// #[discriminant([u8; 8])]
// enum InstructSetThing {
//     IX1 = Instuction1,
//     IX2 = Instruction2,
// }

enum InstructionSetThing<'a> {
    IX1(<Instruction1 as UnsizedType>::Ref<'a>),
    IX2(<Instruction2 as UnsizedType>::Ref<'a>),
}
impl<'a> FrameworkSerialize for InstructionSetThing<'a> {
    fn to_bytes(&self, output: &mut &mut [u8]) -> Result<()> {
        todo!()
    }
}
unsafe impl<'a> FrameworkFromBytes<'a> for InstructionSetThing<'a> {
    fn from_bytes(bytes: &mut &'a [u8]) -> Result<Self> {
        todo!()
    }
}
impl<'a> InstructionSet<'a> for InstructionSetThing<'a> {
    type Discriminant = [u8; 8];

    fn handle_ix(
        self,
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        sys_calls: &mut impl SysCalls,
    ) -> Result<()> {
        todo!()
    }
}

/// A callable instruction that can be used as input to a program.
pub trait Instruction<'a>: FrameworkFromBytes<'a> {
    /// Runs the instruction from a raw solana input.
    fn run_ix_from_raw(
        self,
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        sys_calls: &mut impl SysCalls,
    ) -> Result<()>;
}

/// A framework defined instruction using [`AccountSet`] and other traits.
///
/// The steps are as follows:
/// 1. Split self into decode, validate, and run args using [`Instruction::split_to_args`].
/// 2. Decode the accounts using [`Instruction::Accounts::decode_accounts`](AccountSetDecode::decode_accounts).
/// 3. Run any extra instruction validations using [`Instruction::extra_validations`].
/// 4. Validate the accounts using [`Instruction::Accounts::validate_accounts`](AccountSetValidate::validate_accounts).
/// 5. Run the instruction using [`Instruction::run_instruction`].
/// 6. Set the solana return data using [`Instruction::ReturnType::to_bytes`].
pub trait FrameworkInstruction: UnsizedType {
    /// The instruction data type used to decode accounts.
    type DecodeArg;
    /// The instruction data type used to validate accounts.
    type ValidateArg;
    /// The instruction data type used to run the instruction.
    type RunArg;
    /// The instruction data type used to cleanup accounts.
    type CleanupArg;

    /// The return type of this instruction.
    type ReturnType: FrameworkSerialize;

    /// The [`AccountSet`] used by this instruction.
    type Accounts<'b, 'info>: AccountSetDecode<'b, 'info, Self::DecodeArg>
        + AccountSetValidate<'info, Self::ValidateArg>
        + AccountSetCleanup<'info, Self::CleanupArg>
    where
        'info: 'b;

    /// Splits self into decode, validate, and run args.
    fn split_to_args(
        self,
    ) -> (
        Self::DecodeArg,
        Self::ValidateArg,
        Self::RunArg,
        Self::CleanupArg,
    );
    /// Runs any extra validations on the accounts.
    #[allow(unused_variables)]
    fn extra_validations(
        account_set: &Self::Accounts<'_, '_>,
        validate: &Self::ValidateArg,
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<()> {
        Ok(())
    }
    /// Runs the instruction.
    fn run_instruction<'b, 'info>(
        run_arg: Self::RunArg,
        program_id: &Pubkey,
        account_set: &mut Self::Accounts<'b, 'info>,
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<Self::ReturnType>
    where
        'info: 'b;
}
impl<'a, T> Instruction<'a> for T
where
    T: FrameworkInstruction,
{
    fn run_ix_from_raw(
        self,
        program_id: &Pubkey,
        mut accounts: &[AccountInfo],
        sys_calls: &mut impl SysCalls,
    ) -> Result<()> {
        {
            let (decode, validate, run, cleanup) = self.split_to_args();
            let mut account_set = <Self as FrameworkInstruction<'a>>::Accounts::decode_accounts(
                &mut accounts,
                decode,
                sys_calls,
            )?;
            Self::extra_validations(&account_set, &validate, sys_calls)?;
            account_set.validate_accounts(validate, sys_calls)?;
            let ret = Self::run_instruction(run, program_id, &mut account_set, sys_calls)?;
            account_set.cleanup_accounts(cleanup, sys_calls)?;
            let mut return_data = vec![0u8; MAX_RETURN_DATA];
            let mut return_data_ref = &mut return_data[..];
            ret.to_bytes(&mut return_data_ref)?;
            let return_data_len = return_data_ref.len();
            sys_calls.set_return_data(&return_data[..return_data_len]);
            Ok(())
        }
    }
}
