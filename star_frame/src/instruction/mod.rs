use crate::prelude::*;
use borsh::to_vec;
use bytemuck::{bytes_of, Pod};
use pinocchio::cpi::set_return_data;

pub use star_frame_proc::{InstructionArgs, InstructionSet, InstructionToIdl};

mod no_op;
pub mod un_callable;

/// A set of instructions that can be used as input to a program. This can be derived using the
/// [`star_frame_proc::InstructionSet`] macro on an enum. If implemented manually, [`InstructionSet::handle_ix`] should
/// probably match on each of its instructions discriminants and call the appropriate instruction on a match.
pub trait InstructionSet {
    /// The discriminant type used by this program's instructions.
    type Discriminant: Pod;

    /// Processes the input from the program entrypoint (along with the [`Context`]).
    /// This is called directly in [`StarFrameProgram::entrypoint`].
    fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
        ctx: &mut Context,
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

    #[must_use]
    fn discriminant_bytes() -> Vec<u8> {
        bytes_of(&Self::DISCRIMINANT).into()
    }
}

/// A callable instruction that can be used as input to a program.
pub trait Instruction {
    type ParsedData<'a>;

    fn parse_instruction_data<'a>(instruction_data: &mut &'a [u8]) -> Result<Self::ParsedData<'a>>;

    /// Runs the instruction from a raw solana input.
    fn process_from_parsed(
        accounts: &[AccountInfo],
        data: &mut Self::ParsedData<'_>,
        ctx: &mut Context,
    ) -> Result<()>;

    /// Runs the instruction from a raw solana input. This is called from [`InstructionSet::process_instruction`] after the discriminant is parsed
    /// and matched on.
    fn process_from_raw(
        accounts: &[AccountInfo],
        mut instruction_data: &[u8],
        ctx: &mut Context,
    ) -> Result<()> {
        let mut data = Self::parse_instruction_data(&mut instruction_data)
            .context("Failed to parse instruction data")?;
        Self::process_from_parsed(accounts, &mut data, ctx).context("Failed to process instruction")
    }
}

/// Helper type for the return of [`InstructionArgs::split_to_args`].
#[derive(derive_where::DeriveWhere)]
#[derive_where(
    Default, Debug;
    <T as InstructionArgs>::DecodeArg<'a>,
    <T as InstructionArgs>::ValidateArg<'a>,
    <T as InstructionArgs>::RunArg<'a>,
    <T as InstructionArgs>::CleanupArg<'a>
)]
pub struct IxArgs<'a, T: InstructionArgs> {
    pub decode: <T as InstructionArgs>::DecodeArg<'a>,
    pub validate: <T as InstructionArgs>::ValidateArg<'a>,
    pub run: <T as InstructionArgs>::RunArg<'a>,
    pub cleanup: <T as InstructionArgs>::CleanupArg<'a>,
}

/// A helper trait to split a struct into arguments for a [`StarFrameInstruction`].
///
/// Derivable via [`derive@InstructionArgs`].
pub trait InstructionArgs: Sized {
    /// The instruction data type used to decode accounts.
    type DecodeArg<'a>;
    /// The instruction data type used to validate accounts.
    type ValidateArg<'a>;
    /// The instruction data type used to run the instruction.
    type RunArg<'a>;
    /// The instruction data type used to cleanup accounts.
    type CleanupArg<'a>;
    /// Splits self into decode, validate, cleanup, and run args.
    fn split_to_args(r: &mut Self) -> IxArgs<'_, Self>;
}

/// A `star_frame` defined instruction using [`AccountSet`] and other traits.
///
/// The steps are as follows:
/// 1. Decode self from bytes using [`BorshDeserialize`].
/// 2. Split self into decode, validate, run, and cleanup args using [`InstructionArgs::split_to_args`].
/// 3. Decode the accounts using [`Self::Accounts::decode_accounts`](AccountSetDecode::decode_accounts).
/// 4. Validate the accounts using [`Self::Accounts::validate_accounts`](AccountSetValidate::validate_accounts).
/// 5. Process the instruction using [`Self::process`].
/// 6. Set the solana return data using [`BorshSerialize`] if it is not empty.
pub trait StarFrameInstruction: BorshDeserialize + InstructionArgs {
    /// The return type of this instruction.
    type ReturnType: BorshSerialize;

    /// The [`AccountSet`] used by this instruction.
    type Accounts<'b, 'c>: AccountSetDecode<'b, Self::DecodeArg<'c>>
        + AccountSetValidate<Self::ValidateArg<'c>>
        + AccountSetCleanup<Self::CleanupArg<'c>>;

    /// Processes the instruction.
    fn process(
        accounts: &mut Self::Accounts<'_, '_>,
        run_arg: Self::RunArg<'_>,
        ctx: &mut Context,
    ) -> Result<Self::ReturnType>;
}

impl<T> Instruction for T
where
    T: StarFrameInstruction,
{
    type ParsedData<'a> = T;

    fn parse_instruction_data<'a>(instruction_data: &mut &'a [u8]) -> Result<Self::ParsedData<'a>> {
        <T as BorshDeserialize>::deserialize(instruction_data).map_err(Into::into)
    }

    fn process_from_parsed(
        mut accounts: &[AccountInfo],
        data: &mut Self::ParsedData<'_>,
        ctx: &mut Context,
    ) -> Result<()> {
        let IxArgs {
            decode,
            validate,
            run,
            cleanup,
        } = Self::split_to_args(data);
        let mut account_set =
            <Self as StarFrameInstruction>::Accounts::decode_accounts(&mut accounts, decode, ctx)
                .context("Failed to decode accounts")?;
        account_set
            .validate_accounts(validate, ctx)
            .context("Failed to validate accounts")?;
        let ret = Self::process(&mut account_set, run, ctx).context("Failed to run instruction")?;
        account_set
            .cleanup_accounts(cleanup, ctx)
            .context("Failed to cleanup accounts")?;
        let return_data = to_vec(&ret).context("Failed to serialize return data")?;
        if !return_data.is_empty() {
            set_return_data(&return_data);
        }
        Ok(())
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! empty_star_frame_instruction {
    ($ix:ident, $accounts:ident) => {
        impl $crate::instruction::StarFrameInstruction for $ix {
            type ReturnType = ();
            type Accounts<'b, 'c> = $accounts;

            fn process(
                _account_set: &mut Self::Accounts<'_, '_>,
                _run_arg: Self::RunArg<'_>,
                _ctx: &mut $crate::context::Context,
            ) -> $crate::Result<Self::ReturnType> {
                Ok(())
            }
        }
    };
}

/// A helper macro for implementing blank instructions for testing.
#[doc(hidden)]
#[macro_export]
macro_rules! impl_blank_ix {
    ($($ix:ident),*) => {
        $(
            impl $crate::prelude::Instruction for $ix {
                type ParsedData<'a> = ();
                fn parse_instruction_data<'a>(_instruction_data: &mut &'a [u8]) -> $crate::Result<Self::ParsedData<'a>> {
                    todo!()
                }

                fn process_from_parsed(
                    _accounts: &[$crate::prelude::AccountInfo],
                    _data: &mut Self::ParsedData<'_>,
                    _ctx: &mut $crate::prelude::Context,
                ) -> $crate::Result<()> {
                    todo!()
                }
            }
        )*
    };
}

#[cfg(test)]
mod test {
    use crate::impl_blank_ix;
    use star_frame_proc::InstructionSet;
    // todo: better testing here!

    #[allow(dead_code)]
    struct Ix1 {
        val: u8,
    }
    #[allow(dead_code)]
    struct Ix2 {
        val: u64,
    }

    impl_blank_ix!(Ix1, Ix2);

    #[allow(dead_code)]
    #[derive(InstructionSet)]
    #[ix_set(skip_idl)]
    enum TestInstructionSet3 {
        Ix1(Ix1),
        Ix2(Ix2),
    }
}
