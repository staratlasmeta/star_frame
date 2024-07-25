use bytemuck::Pod;
use derivative::Derivative;
use solana_program::account_info::AccountInfo;
use solana_program::program::MAX_RETURN_DATA;
use solana_program::pubkey::Pubkey;

use crate::prelude::*;
use crate::sys_calls::{SysCallInvoke, SysCalls};
pub use star_frame_proc::star_frame_instruction_set;
pub use star_frame_proc::InstructionToIdl;

mod no_op;
pub mod un_callable;

/// A set of instructions that can be used as input to a program. This can be derived using the
/// [`star_frame_instruction_set`] macro on an enum. If implemented manually, [`Self::handle_ix`] should
/// probably match on each of its instructions discriminants and call the appropriate instruction on a match.
pub trait InstructionSet {
    /// The discriminant type used by this program's instructions.
    type Discriminant: Pod;

    /// Handles the input from the program entrypoint (along with the `sys_calls`).
    /// This is called directly in [`StarFrameProgram::processor`].
    fn handle_ix(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        ix_bytes: &[u8],
        sys_calls: &mut impl SysCalls,
    ) -> Result<()>;
}

/// A helper trait for the value of the instruction discriminant on an instruction. Since a single
/// instruction can be in multiple [`InstructionSet`]s, this trait is generic over it (with `IxSet`).
pub trait InstructionDiscriminant<IxSet>
where
    IxSet: InstructionSet,
{
    /// The actual value of the discriminant. For a single [`InstructionSet`], each member should
    /// have a unique discriminant.
    const DISCRIMINANT: <IxSet as InstructionSet>::Discriminant;
}

/// A callable instruction that can be used as input to a program.
pub trait Instruction {
    type SelfData<'a>;

    fn data_from_bytes<'a>(bytes: &mut &'a [u8]) -> Result<Self::SelfData<'a>>;
    /// Runs the instruction from a raw solana input.
    fn run_ix_from_raw(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        data: &Self::SelfData<'_>,
        sys_calls: &mut impl SysCalls,
    ) -> Result<()>;
}

/// Helper type for the return of [`StarFrameInstruction::split_to_args`].
#[derive(Derivative)]
#[derivative(
    Debug(bound = "<T as StarFrameInstruction>::DecodeArg<'a>: Debug,
            <T as StarFrameInstruction>::ValidateArg<'a>: Debug,
            <T as StarFrameInstruction>::RunArg<'a>: Debug,
            <T as StarFrameInstruction>::CleanupArg<'a>: Debug"),
    Default(bound = "<T as StarFrameInstruction>::DecodeArg<'a>: Default,
            <T as StarFrameInstruction>::ValidateArg<'a>: Default,
            <T as StarFrameInstruction>::RunArg<'a>: Default,
            <T as StarFrameInstruction>::CleanupArg<'a>: Default")
)]
pub struct SplitToArgsReturn<'a, T: StarFrameInstruction + ?Sized> {
    pub decode: <T as StarFrameInstruction>::DecodeArg<'a>,
    pub validate: <T as StarFrameInstruction>::ValidateArg<'a>,
    pub run: <T as StarFrameInstruction>::RunArg<'a>,
    pub cleanup: <T as StarFrameInstruction>::CleanupArg<'a>,
}

/// A `star_frame` defined instruction using [`AccountSet`] and other traits.
///
/// The steps are as follows:
/// 1. Split self into decode, validate, and run args using [`Self::split_to_args`].
/// 2. Decode the accounts using [`Self::Accounts::decode_accounts`](AccountSetDecode::decode_accounts).
/// 3. Run any extra instruction validations using [`Self::extra_validations`].
/// 4. Validate the accounts using [`Self::Accounts::validate_accounts`](AccountSetValidate::validate_accounts).
/// 5. Run the instruction using [`Self::run_instruction`].
/// 6. Set the solana return data using [`StarFrameSerialize::to_bytes`].
pub trait StarFrameInstruction {
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
    type ReturnType: StarFrameSerialize;

    /// The [`AccountSet`] used by this instruction.
    type Accounts<'b, 'c, 'info>: AccountSetDecode<'b, 'info, Self::DecodeArg<'c>>
        + AccountSetValidate<'info, Self::ValidateArg<'c>>
        + AccountSetCleanup<'info, Self::CleanupArg<'c>>
    where
        'info: 'b;

    fn data_from_bytes<'a>(bytes: &mut &'a [u8]) -> Result<Self::SelfData<'a>>;

    /// Splits self into decode, validate, and run args.
    fn split_to_args<'a>(r: &'a Self::SelfData<'_>) -> SplitToArgsReturn<'a, Self>;

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
    T: ?Sized + StarFrameInstruction,
{
    type SelfData<'a> = <Self as StarFrameInstruction>::SelfData<'a>;

    fn data_from_bytes<'a>(bytes: &mut &'a [u8]) -> Result<Self::SelfData<'a>> {
        <T as StarFrameInstruction>::data_from_bytes(bytes)
    }

    fn run_ix_from_raw(
        program_id: &Pubkey,
        mut accounts: &[AccountInfo],
        data: &Self::SelfData<'_>,
        sys_calls: &mut impl SysCalls,
    ) -> Result<()> {
        let SplitToArgsReturn {
            decode,
            validate,
            mut run,
            cleanup,
        } = Self::split_to_args(data);
        let mut account_set = <Self as StarFrameInstruction>::Accounts::decode_accounts(
            &mut accounts,
            decode,
            sys_calls,
        )?;
        account_set.validate_accounts(validate, sys_calls)?;
        Self::extra_validations(&mut account_set, &mut run, sys_calls)?;
        let ret = Self::run_instruction(run, program_id, &mut account_set, sys_calls)?;
        account_set.cleanup_accounts(cleanup, sys_calls)?;
        // todo: handle return data better
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

#[cfg(feature = "test_helpers")]
mod test_helpers {
    /// A helper macro for implementing blank instructions for testing.
    #[macro_export]
    macro_rules! impl_blank_ix {
        ($($ix:ident),*) => {
            $(
                impl Instruction for $ix {
                    type SelfData<'a> = ();
                    fn data_from_bytes<'a>(_bytes: &mut &'a [u8]) -> anyhow::Result<Self::SelfData<'a>> {
                        todo!()
                    }

                    fn run_ix_from_raw(
                        _program_id: &Pubkey,
                        _accounts: &[AccountInfo],
                        _data: &Self::SelfData<'_>,
                        _sys_calls: &mut impl SysCalls,
                    ) -> anyhow::Result<()> {
                        todo!()
                    }
                }
            )*
        };
    }
}

#[cfg(test)]
mod test {
    use solana_program::account_info::AccountInfo;
    use solana_program::pubkey::Pubkey;

    use crate::impl_blank_ix;
    use crate::instruction::Instruction;
    use crate::prelude::SysCalls;
    use star_frame_proc::star_frame_instruction_set;
    // todo: better testing here

    #[allow(dead_code)]
    struct Ix1 {
        val: u8,
    }
    #[allow(dead_code)]
    struct Ix2 {
        val: u64,
    }

    impl_blank_ix!(Ix1, Ix2);

    #[star_frame_instruction_set(u8)]
    enum TestInstructionSet3 {
        Ix1(Ix1),
        Ix2(Ix2),
    }
}
