//! Processing and handling of instructions from a [`StarFrameProgram::entrypoint`].

use crate::{
    account_set::{AccountSetCleanup, AccountSetDecode, AccountSetValidate},
    prelude::*,
};
use anyhow::Context as _;
use borsh::to_vec;
use bytemuck::{bytes_of, Pod};
use derive_more::From;
use pinocchio::cpi::set_return_data;

pub use star_frame_proc::{
    star_frame_instruction, InstructionArgs, InstructionSet, InstructionToIdl,
};

mod no_op;
mod un_callable;
pub use un_callable::UnCallable;

/// A set of instructions that can be used as input to a program.
///
/// This can be derived using the [`derive@InstructionSet`] macro on an enum.
pub trait InstructionSet {
    /// The discriminant type used by this program's instructions.
    type Discriminant: Pod;

    /// Dispatches the instruction data from the program entrypoint and then
    /// calls the appropriate [`Instruction::process_from_raw`] method.
    ///
    /// This is called directly by [`StarFrameProgram::entrypoint`].
    fn dispatch(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
        ctx: &mut Context,
    ) -> Result<()>;
}

/// A helper trait for the value of the instruction discriminant on an instruction.
///
/// Since a single instruction can be in multiple [`InstructionSet`]s, this trait is generic over it
/// (with `IxSet`).
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
    /// Runs the instruction from a raw solana input.
    ///
    /// This is called from [`InstructionSet::dispatch`] after the discriminant is parsed and matched on.
    fn process_from_raw(
        accounts: &[AccountInfo],
        instruction_data: &[u8],
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

#[doc(hidden)]
#[diagnostic::on_unimplemented(
    message = "`StarFrameInstruction` requires the return type to be `Result<()> or Result<ReturnData<T>>`"
)]
/// A helper trait to get the inner T of a [`Result`] from a [`StarFrameInstruction::process`] declaration. This is used internally in the [`star_frame_instruction`] macro.
pub trait IxReturnType {
    type ReturnType;
}
impl<T, E> IxReturnType for Result<T, E> {
    type ReturnType = T;
}

mod sealed {
    use super::*;

    #[doc(hidden)]
    pub trait Sealed {}
    impl Sealed for () {}
    impl<T: BorshSerialize> Sealed for ReturnData<T> {}
}
/// A marker trait for valid return types for a [`StarFrameInstruction`].
#[diagnostic::on_unimplemented(
    message = "`StarFrameInstruction` requires the return type to be `Result<()> or Result<ReturnData<T>>`"
)]
pub trait InstructionReturn: sealed::Sealed {
    fn handle_return(&self) -> Result<()>;
}

impl<T: BorshSerialize> InstructionReturn for ReturnData<T> {
    #[inline]
    fn handle_return(&self) -> Result<()> {
        let return_data = to_vec(&self.0).context("Failed to serialize return data")?;
        if !return_data.is_empty() {
            set_return_data(&return_data);
        }
        Ok(())
    }
}
impl InstructionReturn for () {
    #[inline]
    fn handle_return(&self) -> Result<()> {
        Ok(())
    }
}

/// An opinionated (and recommended) [`Instruction`] using [`AccountSet`] and other traits. Can be derived using the [`star_frame_instruction`] macro.
///
/// The steps for how this implements [`Instruction::process_from_raw`] are as follows:
/// 1. Decode Self from bytes using [`BorshDeserialize`].
/// 2. Split Self into decode, validate, run, and cleanup args using [`InstructionArgs::split_to_args`].
/// 3. Decode the accounts using [`Self::Accounts::decode_accounts`](AccountSetDecode::decode_accounts).
/// 4. Validate the accounts using [`Self::Accounts::validate_accounts`](AccountSetValidate::validate_accounts).
/// 5. Process the instruction using [`Self::process`].
/// 6. Cleanup the accounts using [`Self::Accounts::cleanup_accounts`](AccountSetCleanup::cleanup_accounts).
/// 7. Set the solana return data using [`BorshSerialize`] if it is not empty.
pub trait StarFrameInstruction: BorshDeserialize + InstructionArgs {
    /// The return type of this instruction.
    type ReturnType: InstructionReturn;

    /// The [`AccountSet`] used by this instruction.
    type Accounts<'decode, 'arg>: AccountSetDecode<'decode, Self::DecodeArg<'arg>>
        + AccountSetValidate<Self::ValidateArg<'arg>>
        + AccountSetCleanup<Self::CleanupArg<'arg>>;

    /// Processes the instruction.
    fn process(
        accounts: &mut Self::Accounts<'_, '_>,
        run_arg: Self::RunArg<'_>,
        ctx: &mut Context,
    ) -> Result<Self::ReturnType>;
}

#[derive(Debug, From)]
#[repr(transparent)]
pub struct ReturnData<T>(pub T);

impl<T> Instruction for T
where
    T: StarFrameInstruction,
{
    fn process_from_raw(
        mut accounts: &[AccountInfo],
        mut data: &[u8],
        ctx: &mut Context,
    ) -> Result<()> {
        let mut data = <T as BorshDeserialize>::deserialize(&mut data)
            .context("Failed to deserialize instruction data")?;
        let IxArgs {
            decode,
            validate,
            run,
            cleanup,
        } = Self::split_to_args(&mut data);
        let mut account_set =
            <Self as StarFrameInstruction>::Accounts::decode_accounts(&mut accounts, decode, ctx)
                .context("Failed to decode accounts")?;
        account_set
            .validate_accounts(validate, ctx)
            .context("Failed to validate accounts")?;
        let ret: <T as StarFrameInstruction>::ReturnType =
            Self::process(&mut account_set, run, ctx).context("Failed to run instruction")?;
        account_set
            .cleanup_accounts(cleanup, ctx)
            .context("Failed to cleanup accounts")?;
        ret.handle_return()?;
        Ok(())
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! empty_star_frame_instruction {
    ($ix:ident, $accounts:ident) => {
        impl $crate::instruction::StarFrameInstruction for $ix {
            type ReturnType = ();
            type Accounts<'decode, 'arg> = $accounts;

            fn process(
                _accounts: &mut Self::Accounts<'_, '_>,
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
            impl $crate::instruction::Instruction for $ix {
                fn process_from_raw(
                    _accounts: &[$crate::prelude::AccountInfo],
                    _data: &[u8],
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
