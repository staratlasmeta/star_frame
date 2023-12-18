use crate::TicTacToeProgram;
use framework_test::{
    AccountData, AccountDataInit, AccountDataValidate, AccountInfoDataSection, ProgramAccountEntry,
};
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;

#[derive(Debug, AccountData)]
// TODO: Remove
#[allow(dead_code)]
pub struct DuelAccount {
    pub player1: Pubkey,
    pub player2: Pubkey,
    pub current_score: i8,
    pub current_game: u8,
    pub active_game: Option<Pubkey>,
}
pub struct Combatants {
    pub player1: Pubkey,
    pub player2: Pubkey,
}

impl DuelAccount {
    pub fn player1_turn(&self) -> Turn {
        if self.current_game % 2 == 0 {
            Turn::X
        } else {
            Turn::O
        }
    }

    pub fn end_game(&mut self, win: Option<Turn>) {
        let score_change = win
            .map(|turn| turn as i8 * self.player1_turn() as i8)
            .unwrap_or_default();
        self.current_game += 1;
        self.current_score += score_change;
        self.active_game = None;
    }
}

// account_data_shell!(duelAccount, TicTacToeProgram, 0);

impl AccountDataValidate<()> for DuelAccount {
    fn validate(_data: &Self::NewestVersion, _arg: ()) -> Result<(), ProgramError> {
        todo!()
    }
}
use crate::state::Turn;

impl AccountData for DuelAccount {
    type Program = TicTacToeProgram;
    type NewestVersion = Self;
    type UpgradeArg<'a> = ();
    const REQUIRES_SEEDS: bool = false;

    fn from_bytes<'a>(_bytes: &mut &'a [u8]) -> Result<&'a Self::NewestVersion, ProgramError> {
        todo!()
    }

    fn from_bytes_mut<'a>(
        _bytes: &mut &'a mut [u8],
    ) -> Result<&'a mut Self::NewestVersion, ProgramError> {
        todo!()
    }

    fn upgrade_if_needed<'a>(
        _bytes: &mut &'a mut [u8],
        _arg: Self::UpgradeArg<'_>,
    ) -> Result<&'a mut Self::NewestVersion, ProgramError> {
        todo!()
    }
}
impl ProgramAccountEntry<DuelAccount> for TicTacToeProgram {
    const DISCRIMINANT: u64 = 2;
}

impl AccountDataInit<Combatants> for DuelAccount {
    fn init(bytes: &mut AccountInfoDataSection, init_arg: Combatants) -> Result<(), ProgramError> {
        let duel_account = DuelAccount::from_bytes_mut(&mut bytes.data_bytes_mut())?;

        *duel_account = DuelAccount {
            player1: init_arg.player1,
            player2: init_arg.player2,
            current_score: 0,
            current_game: 0,
            active_game: None,
        };

        Ok(())
    }
}
