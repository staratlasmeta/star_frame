use crate::state::{Combatants, DuelAccount};
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use star_frame::account_set::signer::Signer;
use star_frame::account_set::system_account::SystemAccount;
use star_frame::account_set::AccountSet;
use star_frame::instruction::{FrameworkInstruction, ToBytes};
use star_frame::program::system_program::SystemProgram;
use star_frame::syscalls::SysCallInvoke;

// #[instruction]
// mod instruction_module {
//     #[instruction_data]
//     pub struct InstructionData {
//         #[framework(decode, validate = clone, run = as_ref)]
//         pub single_arg: DecodeArgType,
//     }
//
//     #[run_instruction]
//     fn run_instruction<'info>(
//         run_arg: RunArgType,
//         program_id: &Pubkey,
//         account_set: AccountsType<'info>,
//         syscalls: &mut impl SyscallInvoke<'_>,
//     ) -> Result<InstructionReturnType, ProgramError> {
//     }
// }

#[derive(Debug)]
pub struct CreateDuel {
    pub best_of: u8,
}

#[derive(Debug, AccountSet)]
#[validate(arg = InitArgs<Combatants>)]
pub struct CreateDuelAccounts<'info> {
    pub player1: Signer<AccountInfo<'info>>,
    pub player2: AccountInfo<'info>,
    pub funder: SystemAccount<'info>,
    // TODO: Try to get Init account working? also use seeds there?
    #[validate(arg = self.init_args())]
    #[cleanup(arg = self.cleanup_args())]
    pub duel: InitAccount<'info, DuelAccount>,
    pub system_program: SystemProgram,
}
impl<'info> CreateDuelAccounts<'info> {
    fn init_args(&self) -> InitArgs<'_, 'info, Combatants> {
        InitArgs {
            system_program: &self.system_program,
            init: Combatants {
                player1: *self.player1.key(),
                player2: *self.player2.key(),
            },
        }
    }

    fn cleanup_args(&self) -> DataAccountCleanup<'_, 'info> {
        DataAccountCleanup {
            funder: &self.funder,
            system_program: &self.system_program,
            seeds: None,
        }
    }
}

impl ToBytes for CreateDuel {
    fn to_bytes(&self, _output: &mut &mut [u8]) -> ProgramResult {
        todo!()
    }
}

impl<'a> FrameworkInstruction<'a> for CreateDuel {
    type DecodeArg = ();
    type ValidateArg = ();
    type RunArg = ();
    type CleanupArg = ();
    type ReturnType = ();
    type Accounts<'b, 'info> = CreateDuelAccounts<'info>;

    fn from_bytes_framework(_bytes: &'a [u8]) -> Result<Self, ProgramError> {
        todo!()
    }

    fn split_to_args(
        self,
    ) -> (
        Self::DecodeArg,
        Self::ValidateArg,
        Self::RunArg,
        Self::CleanupArg,
    ) {
        todo!()
    }

    fn run_instruction(
        _run_arg: Self::RunArg,
        _program_id: &Pubkey,
        _account_set: &Self::Accounts<'_, '_>,
        _syscalls: &mut impl SyscallInvoke<'_>,
    ) -> Result<Self::ReturnType, ProgramError> {
        Ok(())
    }
}
