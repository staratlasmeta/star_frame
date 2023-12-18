use crate::state::{DuelAccount, GameAccount};
use framework_test::{
    AccountSet, DataAccount, FrameworkInstruction, SafeAccountInfo, Signer, SysCallInvoke, ToBytes,
};
use solana_program::entrypoint::ProgramResult;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;

#[derive(Debug)]
pub struct MakeMove {
    pub square_index: u8,
}

#[derive(Debug, AccountSet)]
#[validate(func = MakeMoveAccounts::validate_correct_turn)]
pub struct MakeMoveAccounts<'info> {
    pub game_account: DataAccount<'info, GameAccount>,
    // Only mut if the game is "over" in order to increment the score/game number on duel_account.
    // Sorta a contrived example of sometimes mutable accounts, but meh whatever.
    pub duel_account: DataAccount<'info, DuelAccount>,
    // constraint = game.turn == dual.player1_turn() { player == dual.player1 } else { player == dual.player2 }
    pub player: Signer<SafeAccountInfo<'info>>,
}

impl<'info> MakeMoveAccounts<'info> {
    pub fn validate_correct_turn(&self) -> ProgramResult {
        let game_account = self.game_account.data()?;
        let duel_account = self.duel_account.data()?;

        // If an even game turn Player 1 should start, otherwise Player 2 should start
        if game_account.turn == duel_account.player1_turn() {
            // Verify player is player1
            if self.player.key() != &duel_account.player1 {
                return Err(ProgramError::InvalidAccountData);
            }
        } else {
            // Verify player is player2
            if self.player.key() != &duel_account.player2 {
                return Err(ProgramError::InvalidAccountData);
            }
        }

        Ok(())
    }
}

impl ToBytes for MakeMove {
    fn to_bytes(&self, _output: &mut &mut [u8]) -> ProgramResult {
        todo!()
    }
}

impl<'a> FrameworkInstruction<'a> for MakeMove {
    type DecodeArg = ();
    type ValidateArg = ();
    type RunArg = u8;
    type CleanupArg = ();
    type ReturnType = ();
    type Accounts<'b, 'info> = MakeMoveAccounts<'info>;

    fn from_bytes(_bytes: &'a [u8]) -> Result<Self, ProgramError> {
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
        ((), (), self.square_index, ())
    }

    fn run_instruction(
        run_arg: Self::RunArg,
        _program_id: &Pubkey,
        account_set: &Self::Accounts<'_, '_>,
        _sys_calls: &mut impl SysCallInvoke,
    ) -> Result<Self::ReturnType, ProgramError> {
        let mut game_account = account_set.game_account.data_mut()?;
        let mut duel_account = account_set.duel_account.data_mut()?;
        if game_account.make_move(run_arg)? {
            duel_account.end_game(Some(game_account.turn));
        }
        if game_account.is_full() {
            duel_account.end_game(None);
        }
        Ok(())
    }
}
