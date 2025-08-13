use crate::context::Context;
use crate::prelude::*;
use anyhow::Context as _;
use borsh::{to_vec, BorshDeserialize, BorshSerialize};
use bytemuck::{bytes_of, Pod};
use pinocchio::{account_info::AccountInfo, cpi::set_return_data};
use solana_pubkey::Pubkey;
pub use star_frame_proc::{InstructionArgs, InstructionSet, InstructionToIdl};

mod no_op;
pub mod un_callable;

/// A set of instructions that can be used as input to a program. This can be derived using the
/// [`star_frame_proc::InstructionSet`] macro on an enum. If implemented manually, [`Self::handle_ix`] should
/// probably match on each of its instructions discriminants and call the appropriate instruction on a match.
pub trait InstructionSet {
    /// The discriminant type used by this program's instructions.
    type Discriminant: Pod;

    /// Handles the input from the program entrypoint (along with the `context`).
    /// This is called directly in [`StarFrameProgram::processor`].
    fn handle_ix(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        ix_bytes: &[u8],
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
    type SelfData<'a>;

    fn data_from_bytes<'a>(bytes: &mut &'a [u8]) -> Result<Self::SelfData<'a>>;
    /// Runs the instruction from a raw solana input.
    fn run_ix_from_raw(
        accounts: &[AccountInfo],
        data: &mut Self::SelfData<'_>,
        ctx: &mut Context,
    ) -> Result<()>;
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
/// 1. Split self into decode, validate, and run args using [`InstructionArgs::split_to_args`].
/// 2. Decode the accounts using [`Self::Accounts::decode_accounts`](AccountSetDecode::decode_accounts).
/// 3. Run any extra instruction validations using [`Self::extra_validations`].
/// 4. Validate the accounts using [`Self::Accounts::validate_accounts`](AccountSetValidate::validate_accounts).
/// 5. Run the instruction using [`Self::run_instruction`].
/// 6. Set the solana return data using [`BorshSerialize`].
pub trait StarFrameInstruction: BorshDeserialize + InstructionArgs {
    /// The return type of this instruction.
    type ReturnType: BorshSerialize;

    /// The [`AccountSet`] used by this instruction.
    type Accounts<'b, 'c>: AccountSetDecode<'b, Self::DecodeArg<'c>>
        + AccountSetValidate<Self::ValidateArg<'c>>
        + AccountSetCleanup<Self::CleanupArg<'c>>;

    /// Runs any extra validations on the accounts.
    #[allow(unused_variables)]
    fn extra_validations(
        account_set: &mut Self::Accounts<'_, '_>,
        run_arg: &mut Self::RunArg<'_>,
        ctx: &mut Context,
    ) -> Result<()> {
        Ok(())
    }
    /// Runs the instruction.
    fn run_instruction(
        account_set: &mut Self::Accounts<'_, '_>,
        run_arg: Self::RunArg<'_>,
        ctx: &mut Context,
    ) -> Result<Self::ReturnType>;
}

impl<T> Instruction for T
where
    T: StarFrameInstruction,
{
    type SelfData<'a> = T;

    fn data_from_bytes<'a>(bytes: &mut &'a [u8]) -> Result<Self::SelfData<'a>> {
        <T as BorshDeserialize>::deserialize(bytes).map_err(Into::into)
    }

    fn run_ix_from_raw(
        mut accounts: &[AccountInfo],
        data: &mut Self,
        ctx: &mut Context,
    ) -> Result<()> {
        let IxArgs {
            decode,
            validate,
            mut run,
            cleanup,
        } = Self::split_to_args(data);
        // SAFETY: .validate_accounts is called after .decode_accounts
        let mut account_set = unsafe {
            <Self as StarFrameInstruction>::Accounts::decode_accounts(&mut accounts, decode, ctx)
        }
        .context("Failed to decode accounts")?;
        account_set
            .validate_accounts(validate, ctx)
            .context("Failed to validate accounts")?;
        Self::extra_validations(&mut account_set, &mut run, ctx)
            .context("Failed in extra validations accounts")?;
        let ret = Self::run_instruction(&mut account_set, run, ctx)
            .context("Failed to run instruction")?;
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

#[macro_export]
macro_rules! empty_star_frame_instruction {
    ($ix:ident, $accounts:ident) => {
        impl $crate::instruction::StarFrameInstruction for $ix {
            type ReturnType = ();
            type Accounts<'b, 'c> = $accounts;

            fn run_instruction(
                _account_set: &mut Self::Accounts<'_, '_>,
                _run_args: Self::RunArg<'_>,
                _ctx: &mut $crate::context::Context,
            ) -> $crate::Result<Self::ReturnType> {
                Ok(())
            }
        }
    };
}

/// A helper macro for implementing blank instructions for testing.
#[macro_export]
macro_rules! impl_blank_ix {
    ($($ix:ident),*) => {
        $(
            impl $crate::prelude::Instruction for $ix {
                type SelfData<'a> = ();
                fn data_from_bytes<'a>(_bytes: &mut &'a [u8]) -> $crate::Result<Self::SelfData<'a>> {
                    todo!()
                }

                fn run_ix_from_raw(
                    _accounts: &[$crate::prelude::AccountInfo],
                    _data: &mut Self::SelfData<'_>,
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
