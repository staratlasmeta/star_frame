use crate::prelude::*;
use borsh::BorshDeserialize;
use solana_program::system_program;
use system_instruction_impl::SystemInstructionSet;

/// Solana's system program.
#[derive(Debug, Copy, Clone, Align1)]
pub struct SystemProgram;
impl StarFrameProgram for SystemProgram {
    type InstructionSet = SystemInstructionSet;
    type AccountDiscriminant = ();
    const CLOSED_ACCOUNT_DISCRIMINANT: Self::AccountDiscriminant = ();
    const PROGRAM_ID: Pubkey = system_program::ID;
}

#[cfg(feature = "idl")]
impl ProgramToIdl for SystemProgram {
    fn version() -> star_frame_idl::Version {
        star_frame_idl::Version::new(1, 18, 10)
    }
}

mod system_instruction_impl {
    use super::*;

    // todo: support custom discriminants
    #[derive(Copy, Debug, Clone, PartialEq, Eq, InstructionSet)]
    pub enum SystemInstructionSet {
        CreateAccount(CreateAccountIx),
    }

    /// Accounts for the [`CreateAccountIx`] instruction.
    #[derive(Debug, AccountSet)]
    pub struct CreateAccountSet<'info> {
        /// The account that pays the rent for the `new_account`
        pub funder: Mut<Signer<AccountInfo<'info>>>,
        pub new_account: Mut<Signer<AccountInfo<'info>>>,
    }

    /// Creates a new account and assigns ownership to the `owner` program.
    #[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionToIdl, BorshDeserialize)]
    #[instruction_to_idl(program = SystemProgram)]
    pub struct CreateAccountIx {
        /// The number of lamports to transfer to the new account. 1 SOL = 10^9 lamports
        pub lamports: u64,
        pub space: u64,
        pub owner: Pubkey,
    }
    impl StarFrameInstruction for CreateAccountIx {
        type DecodeArg<'a> = ();
        type ValidateArg<'a> = ();
        type RunArg<'a> = ();
        type CleanupArg<'a> = ();
        type ReturnType = ();
        type Accounts<'b, 'c, 'info> = CreateAccountSet<'info>;

        fn split_to_args(_r: &Self) -> IxArgs<Self> {
            unimplemented!()
        }

        fn run_instruction<'info>(
            _account_set: &mut Self::Accounts<'_, '_, 'info>,
            _run_arg: Self::RunArg<'_>,
            _syscalls: &mut impl SyscallInvoke<'info>,
        ) -> Result<Self::ReturnType> {
            unimplemented!()
        }
    }

    #[cfg(feature = "idl")]
    #[test]
    fn check_idl() {
        use star_frame_idl::item_source;
        use star_frame_idl::ty::IdlTypeDef;

        let idl = SystemProgram::program_to_idl().unwrap();
        let ix_set_source = item_source::<CreateAccountSet>();
        let ix_source = item_source::<CreateAccountIx>();
        assert!(idl.instructions.contains_key(&ix_source));
        assert!(idl.account_sets.contains_key(&ix_set_source));
        assert!(idl.types.contains_key(&ix_source));
        let create_account_data = idl.types.get(&ix_source).unwrap();
        assert!(matches!(
            create_account_data.type_def,
            IdlTypeDef::Struct(_)
        ));
    }
}
