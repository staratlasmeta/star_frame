pub mod un_callable;

pub use star_frame_proc::star_frame_instruction_set;
pub use star_frame_proc::InstructionToIdl;

use crate::account_set::{AccountSetCleanup, AccountSetDecode, AccountSetValidate};
use crate::sys_calls::{SysCallInvoke, SysCalls};
use crate::Result;
use bytemuck::Pod;
use solana_program::account_info::AccountInfo;
use solana_program::program::MAX_RETURN_DATA;
use solana_program::pubkey::Pubkey;
use star_frame::serialize::FrameworkSerialize;

/// A set of instructions that can be used as input to a program.
pub trait InstructionSet {
    /// The discriminant type used by this program's accounts.
    type Discriminant: Pod;

    /// Handles the instruction obtained from [`InstructionSet::from_bytes`].
    fn handle_ix(
        ix_bytes: &[u8],
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        sys_calls: &mut impl SysCalls,
    ) -> Result<()>;
}

/// A callable instruction that can be used as input to a program.
pub trait Instruction {
    type SelfData<'a>;

    fn data_from_bytes<'a>(bytes: &mut &'a [u8]) -> Result<Self::SelfData<'a>>;
    /// Runs the instruction from a raw solana input.
    fn run_ix_from_raw(
        data: &Self::SelfData<'_>,
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
pub trait FrameworkInstruction {
    type SelfData<'a>;

    /// The instruction data type used to decode accounts.
    type DecodeArg<'a>;
    /// The instruction data type used to validate accounts.
    type ValidateArg<'a>;
    /// The instruction data type used to run the instruction.
    type RunArg<'a>;
    /// The instruction data type used to cleanup accounts.
    type CleanupArg<'a>;

    /// The return type of this instruction.
    type ReturnType: FrameworkSerialize;

    /// The [`AccountSet`] used by this instruction.
    type Accounts<'b, 'c, 'info>: AccountSetDecode<'b, 'info, Self::DecodeArg<'c>>
        + AccountSetValidate<'info, Self::ValidateArg<'c>>
        + AccountSetCleanup<'info, Self::CleanupArg<'c>>
    where
        'info: 'b;

    fn data_from_bytes<'a>(bytes: &mut &'a [u8]) -> Result<Self::SelfData<'a>>;

    /// Splits self into decode, validate, and run args.
    fn split_to_args<'a>(
        r: &'a Self::SelfData<'_>,
    ) -> (
        Self::DecodeArg<'a>,
        Self::ValidateArg<'a>,
        Self::RunArg<'a>,
        Self::CleanupArg<'a>,
    );
    /// Runs any extra validations on the accounts.
    #[allow(unused_variables)]
    fn extra_validations(
        account_set: &mut Self::Accounts<'_, '_, '_>,
        validate: &mut Self::RunArg<'_>,
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<()> {
        Ok(())
    }
    /// Runs the instruction.
    fn run_instruction<'b, 'info>(
        run_arg: Self::RunArg<'_>,
        program_id: &Pubkey,
        account_set: &mut Self::Accounts<'b, '_, 'info>,
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<Self::ReturnType>
    where
        'info: 'b;
}

impl<T> Instruction for T
where
    T: ?Sized + FrameworkInstruction,
{
    type SelfData<'a> = <Self as FrameworkInstruction>::SelfData<'a>;

    fn data_from_bytes<'a>(bytes: &mut &'a [u8]) -> Result<Self::SelfData<'a>> {
        <T as FrameworkInstruction>::data_from_bytes(bytes)
    }

    fn run_ix_from_raw(
        data: &Self::SelfData<'_>,
        program_id: &Pubkey,
        mut accounts: &[AccountInfo],
        sys_calls: &mut impl SysCalls,
    ) -> Result<()> {
        let (decode, validate, mut run, cleanup) = Self::split_to_args(data);
        let mut account_set = <Self as FrameworkInstruction>::Accounts::decode_accounts(
            &mut accounts,
            decode,
            sys_calls,
        )?;
        account_set.validate_accounts(validate, sys_calls)?;
        Self::extra_validations(&mut account_set, &mut run, sys_calls)?;
        let ret = Self::run_instruction(run, program_id, &mut account_set, sys_calls)?;
        account_set.cleanup_accounts(cleanup, sys_calls)?;
        let mut return_data = vec![0u8; MAX_RETURN_DATA];
        let mut return_data_ref = &mut return_data[..];
        ret.to_bytes(&mut return_data_ref)?;
        if return_data_ref.len() != MAX_RETURN_DATA {
            let return_data_len = MAX_RETURN_DATA - return_data_ref.len();
            sys_calls.set_return_data(&return_data[..return_data_len]);
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::instruction::Instruction;
    use crate::prelude::SysCalls;
    use solana_program::account_info::AccountInfo;
    use solana_program::pubkey::Pubkey;
    use star_frame_proc::star_frame_instruction_set;

    #[allow(dead_code)]
    struct Ix1 {
        val: u8,
    }
    impl Instruction for Ix1 {
        type SelfData<'a> = ();

        fn data_from_bytes<'a>(_bytes: &mut &'a [u8]) -> anyhow::Result<Self::SelfData<'a>> {
            todo!()
        }

        fn run_ix_from_raw(
            _data: &Self::SelfData<'_>,
            _program_id: &Pubkey,
            _accounts: &[AccountInfo],
            _sys_calls: &mut impl SysCalls,
        ) -> anyhow::Result<()> {
            todo!()
        }
    }
    #[allow(dead_code)]
    struct Ix2 {
        val: u64,
    }
    impl Instruction for Ix2 {
        type SelfData<'a> = ();

        fn data_from_bytes<'a>(_bytes: &mut &'a [u8]) -> anyhow::Result<Self::SelfData<'a>> {
            todo!()
        }

        fn run_ix_from_raw(
            _data: &Self::SelfData<'_>,
            _program_id: &Pubkey,
            _accounts: &[AccountInfo],
            _sys_calls: &mut impl SysCalls,
        ) -> anyhow::Result<()> {
            todo!()
        }
    }

    #[star_frame_instruction_set]
    enum TestInstructionSet1 {
        Ix1(Ix1),
        Ix2(Ix2),
    }
}
