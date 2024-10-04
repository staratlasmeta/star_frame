use crate::prelude::*;
use crate::syscalls::{SyscallInvoke, Syscalls};
use borsh::{to_vec, BorshDeserialize, BorshSerialize};
use bytemuck::Pod;
use derivative::Derivative;
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;
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

    /// Handles the input from the program entrypoint (along with the `syscalls`).
    /// This is called directly in [`StarFrameProgram::processor`].
    fn handle_ix<'info>(
        program_id: &Pubkey,
        accounts: &[AccountInfo<'info>],
        ix_bytes: &[u8],
        syscalls: &mut impl Syscalls<'info>,
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
    fn run_ix_from_raw<'info>(
        accounts: &[AccountInfo<'info>],
        data: &Self::SelfData<'_>,
        syscalls: &mut impl Syscalls<'info>,
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
pub struct IxArgs<'a, T: StarFrameInstruction + ?Sized> {
    pub decode: <T as StarFrameInstruction>::DecodeArg<'a>,
    pub validate: <T as StarFrameInstruction>::ValidateArg<'a>,
    pub run: <T as StarFrameInstruction>::RunArg<'a>,
    pub cleanup: <T as StarFrameInstruction>::CleanupArg<'a>,
}

impl<'a, T: StarFrameInstruction + ?Sized> IxArgs<'a, T> {
    pub fn decode<D>(decode: D) -> Self
    where
        T: StarFrameInstruction<
            DecodeArg<'a> = D,
            ValidateArg<'a> = (),
            CleanupArg<'a> = (),
            RunArg<'a> = (),
        >,
    {
        Self {
            decode,
            validate: (),
            run: (),
            cleanup: (),
        }
    }

    pub fn validate<V>(validate: V) -> Self
    where
        T: StarFrameInstruction<
            DecodeArg<'a> = (),
            ValidateArg<'a> = V,
            CleanupArg<'a> = (),
            RunArg<'a> = (),
        >,
    {
        Self {
            decode: (),
            validate,
            run: (),
            cleanup: (),
        }
    }

    pub fn run<R>(run: R) -> Self
    where
        T: StarFrameInstruction<
            DecodeArg<'a> = (),
            ValidateArg<'a> = (),
            CleanupArg<'a> = (),
            RunArg<'a> = R,
        >,
    {
        Self {
            decode: (),
            validate: (),
            run,
            cleanup: (),
        }
    }

    pub fn cleanup<C>(cleanup: C) -> Self
    where
        T: StarFrameInstruction<
            DecodeArg<'a> = (),
            ValidateArg<'a> = (),
            CleanupArg<'a> = C,
            RunArg<'a> = (),
        >,
    {
        Self {
            decode: (),
            validate: (),
            run: (),
            cleanup,
        }
    }
}

/// A `star_frame` defined instruction using [`AccountSet`] and other traits.
///
/// The steps are as follows:
/// 1. Split self into decode, validate, and run args using [`Self::split_to_args`].
/// 2. Decode the accounts using [`Self::Accounts::decode_accounts`](AccountSetDecode::decode_accounts).
/// 3. Run any extra instruction validations using [`Self::extra_validations`].
/// 4. Validate the accounts using [`Self::Accounts::validate_accounts`](AccountSetValidate::validate_accounts).
/// 5. Run the instruction using [`Self::run_instruction`].
/// 6. Set the solana return data using [`BorshSerialize`].
pub trait StarFrameInstruction: BorshDeserialize {
    /// The instruction data type used to decode accounts.
    type DecodeArg<'a>;
    /// The instruction data type used to validate accounts.
    type ValidateArg<'a>;
    /// The instruction data type used to run the instruction.
    type RunArg<'a>;
    /// The instruction data type used to cleanup accounts.
    type CleanupArg<'a>;

    /// The return type of this instruction.
    type ReturnType: BorshSerialize;

    /// The [`AccountSet`] used by this instruction.
    type Accounts<'b, 'c, 'info>: AccountSetDecode<'b, 'info, Self::DecodeArg<'c>>
        + AccountSetValidate<'info, Self::ValidateArg<'c>>
        + AccountSetCleanup<'info, Self::CleanupArg<'c>>;

    /// Splits self into decode, validate, and run args.
    fn split_to_args(r: &Self) -> IxArgs<Self>;

    /// Runs any extra validations on the accounts.
    #[allow(unused_variables)]
    fn extra_validations<'info>(
        account_set: &mut Self::Accounts<'_, '_, 'info>,
        run_arg: &mut Self::RunArg<'_>,
        syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()> {
        Ok(())
    }
    /// Runs the instruction.
    fn run_instruction<'info>(
        account_set: &mut Self::Accounts<'_, '_, 'info>,
        run_arg: Self::RunArg<'_>,
        syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<Self::ReturnType>;
}

impl<T> Instruction for T
where
    T: ?Sized + StarFrameInstruction,
{
    type SelfData<'a> = T;

    fn data_from_bytes<'a>(bytes: &mut &'a [u8]) -> Result<Self::SelfData<'a>> {
        <T as BorshDeserialize>::deserialize(bytes).map_err(Into::into)
    }

    fn run_ix_from_raw<'info>(
        mut accounts: &[AccountInfo<'info>],
        data: &Self::SelfData<'_>,
        syscalls: &mut impl Syscalls<'info>,
    ) -> Result<()> {
        let IxArgs {
            decode,
            validate,
            mut run,
            cleanup,
        } = Self::split_to_args(data);
        let mut account_set = <Self as StarFrameInstruction>::Accounts::decode_accounts(
            &mut accounts,
            decode,
            syscalls,
        )?;
        account_set.set_account_cache(syscalls);
        account_set.validate_accounts(validate, syscalls)?;
        Self::extra_validations(&mut account_set, &mut run, syscalls)?;
        let ret = Self::run_instruction(&mut account_set, run, syscalls)?;
        account_set.cleanup_accounts(cleanup, syscalls)?;
        let return_data = to_vec(&ret)?;
        if !return_data.is_empty() {
            syscalls.set_return_data(&return_data);
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
                impl $crate::prelude::Instruction for $ix {
                    type SelfData<'a> = ();
                    fn data_from_bytes<'a>(_bytes: &mut &'a [u8]) -> $crate::Result<Self::SelfData<'a>> {
                        todo!()
                    }

                    fn run_ix_from_raw<'info>(
                        _accounts: &[$crate::prelude::AccountInfo<'info>],
                        _data: &Self::SelfData<'_>,
                        _syscalls: &mut impl $crate::prelude::Syscalls<'info>,
                    ) -> $crate::Result<()> {
                        todo!()
                    }
                }
            )*
        };
    }
}

#[cfg(test)]
mod test {
    use crate::impl_blank_ix;
    use star_frame_proc::star_frame_instruction_set;
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

    #[star_frame_instruction_set(u8)]
    enum TestInstructionSet3 {
        Ix1(Ix1),
        Ix2(Ix2),
    }
}
