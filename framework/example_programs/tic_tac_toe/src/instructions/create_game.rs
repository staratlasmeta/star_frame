use crate::state::{DuelAccount, GameAccount};
use framework_test::borsh::{BorshDeserialize, BorshSerialize};
use framework_test::{borsh, SingleAccountSet};
use framework_test::{
    AccountSet, DataAccount, DataAccountCleanup, FrameworkInstruction, InitAccount, InitArgs,
    SafeAccountInfo, Signer, SysCallInvoke, SystemAccount, SystemProgram, ToBytes,
};
use solana_program::entrypoint::ProgramResult;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;

#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct CreateGame {}

#[derive(Debug, AccountSet)]
#[validate(func = Self::validate_game)]
pub struct CreateGameAccounts<'info> {
    pub funder: SystemAccount<'info>,
    // TODO: Confirm that there are no other open games
    pub duel_account: DataAccount<'info, DuelAccount>,
    #[cleanup(arg = self.cleanup_args())]
    // TODO: Create Init macro which handles creating InitArgs - should be able to check if the system program is provided
    // #[init(arg = ())]
    #[validate(ty = InitArgs<()>, arg = self.init_args())]
    pub new_game_account: InitAccount<'info, GameAccount>,
    pub player: Signer<SafeAccountInfo<'info>>,
    pub system_program: SystemProgram<'info>,
}

impl<'info> CreateGameAccounts<'info> {
    fn init_args(&self) -> InitArgs<'_, 'info, ()> {
        // Init args is empty because a create game account is always initialize the same way
        InitArgs {
            system_program: &self.system_program,
            init: (),
        }
    }

    fn cleanup_args(&self) -> DataAccountCleanup<'_, 'info> {
        DataAccountCleanup {
            funder: &self.funder,
            system_program: &self.system_program,
            seeds: None,
        }
    }

    // TODO: Can you have multiple validation functions?
    fn validate_game(&self) -> ProgramResult {
        let duel_account_data = self.duel_account.data()?;

        // Player creating the game must be a participant in the duel
        if self.player.key() != &duel_account_data.player1
            && self.player.key() != &duel_account_data.player2
        {
            return Err(ProgramError::InvalidAccountData);
        }

        // Must not be an existing active game
        if duel_account_data.active_game.is_some() {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(())
    }
}

impl ToBytes for CreateGame {
    fn to_bytes(&self, output: &mut &mut [u8]) -> ProgramResult {
        BorshSerialize::serialize(&self, output)
            .map_err(|_| ProgramError::BorshIoError("Could not deserialize CreateGame".to_string()))
    }
}

impl<'a> FrameworkInstruction<'a> for CreateGame {
    type DecodeArg = ();
    type ValidateArg = ();
    type RunArg = ();
    type CleanupArg = ();
    type ReturnType = ();
    type Accounts<'b, 'info> = CreateGameAccounts<'info>;

    fn from_bytes(bytes: &'a [u8]) -> Result<Self, ProgramError> {
        BorshDeserialize::try_from_slice(bytes)
            .map_err(|_| ProgramError::BorshIoError("Could not deserialize CreateGame".to_string()))
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
        account_set: &Self::Accounts<'_, '_>,
        _syscalls: &mut impl SysCallInvoke,
    ) -> Result<Self::ReturnType, ProgramError> {
        // Increment game count in duel account
        let mut duel_account_data = account_set.duel_account.data_mut()?;
        duel_account_data.current_game += 1;

        // Set active game
        duel_account_data.active_game = Some(*account_set.new_game_account.key());

        Ok(())
    }
}
