use bytemuck::Pod;
use solana_program::account_info::AccountInfo;
use solana_program::program::MAX_RETURN_DATA;
use solana_program::pubkey::Pubkey;

use star_frame::serialize::StarFrameSerialize;
pub use star_frame_proc::star_frame_instruction_set;
pub use star_frame_proc::InstructionToIdl;

use crate::account_set::{AccountSetCleanup, AccountSetDecode, AccountSetValidate};
use crate::sys_calls::{SysCallInvoke, SysCalls};
use crate::Result;

mod no_op;
pub mod un_callable;

/// A set of instructions that can be used as input to a program. This can be derived using the
/// [`star_frame_instruction_set`] macro on an enum. If implemented manually, [`Self::handle_ix`] should
/// probably match on each of its instructions discriminants and call the appropriate instruction on a match.
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
        data: &Self::SelfData<'_>,
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        sys_calls: &mut impl SysCalls,
    ) -> Result<()>;
}

/// A `star_frame` defined instruction using [`AccountSet`] and other traits.
///
/// The steps are as follows:
/// 1. Split self into decode, validate, and run args using [`Instruction::split_to_args`].
/// 2. Decode the accounts using [`Instruction::Accounts::decode_accounts`](AccountSetDecode::decode_accounts).
/// 3. Run any extra instruction validations using [`Instruction::extra_validations`].
/// 4. Validate the accounts using [`Instruction::Accounts::validate_accounts`](AccountSetValidate::validate_accounts).
/// 5. Run the instruction using [`Instruction::run_instruction`].
/// 6. Set the solana return data using [`Instruction::ReturnType::to_bytes`].
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
    T: ?Sized + StarFrameInstruction,
{
    type SelfData<'a> = <Self as StarFrameInstruction>::SelfData<'a>;

    fn data_from_bytes<'a>(bytes: &mut &'a [u8]) -> Result<Self::SelfData<'a>> {
        <T as StarFrameInstruction>::data_from_bytes(bytes)
    }

    fn run_ix_from_raw(
        data: &Self::SelfData<'_>,
        program_id: &Pubkey,
        mut accounts: &[AccountInfo],
        sys_calls: &mut impl SysCalls,
    ) -> Result<()> {
        let (decode, validate, mut run, cleanup) = Self::split_to_args(data);
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

#[cfg(test)]
mod test {
    use solana_program::account_info::AccountInfo;
    use solana_program::pubkey::Pubkey;

    use star_frame_proc::star_frame_instruction_set;

    use crate::instruction::{Instruction, InstructionDiscriminant};
    use crate::prelude::SysCalls;

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

    #[star_frame_instruction_set(u8)]
    enum TestInstructionSet3 {
        Ix1(Ix1),
        Ix2(Ix2),
    }

    // fn stuff() {
    //     let thingy = TestInstructionSet1::Ix1(());
    //     let thingy_disc = <Ix1 as InstructionDiscriminant<TestInstructionSet2>>::DISCRIMINANT;
    // }
    //
    // impl<'__a> star_frame::instruction::InstructionSet for TestInstructionSet1<'__a> {
    //     type Discriminant = u8;
    //     fn handle_ix(
    //         mut ix_bytes: &[u8],
    //         program_id: &star_frame::solana_program::pubkey::Pubkey,
    //         accounts: &[star_frame::solana_program::account_info::AccountInfo],
    //         sys_calls: &mut impl star_frame::sys_calls::SysCalls,
    //     ) -> star_frame::Result<()> {
    //         const DISC0: u8 = 0;
    //         const DISC1: u8 = (0) + 1;
    //         let discriminant = u8::from_le_bytes(
    //             *star_frame::advance::AdvanceArray::try_advance_array(&mut ix_bytes)?,
    //         );
    //         match discriminant {
    //             DISC0 => {
    //                 let data = <Ix1 as star_frame::instruction::Instruction>::data_from_bytes(
    //                     &mut ix_bytes,
    //                 )?;
    //                 <Ix1 as star_frame::instruction::Instruction>::run_ix_from_raw(
    //                     &data, program_id, accounts, sys_calls,
    //                 )
    //             }
    //             DISC1 => {
    //                 let data = <Ix2 as star_frame::instruction::Instruction>::data_from_bytes(
    //                     &mut ix_bytes,
    //                 )?;
    //                 <Ix2 as star_frame::instruction::Instruction>::run_ix_from_raw(
    //                     &data, program_id, accounts, sys_calls,
    //                 )
    //             }
    //             x => Err(star_frame::anyhow::anyhow!(
    //                 "Invalid ix discriminant: {}",
    //                 x
    //             )),
    //         }
    //     }
}
