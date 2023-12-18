use crate::TicTacToeProgram;
use framework_test::{
    AccountData, AccountDataInit, AccountDataValidate, AccountInfoDataSection, ProgramAccountEntry,
};
use solana_program::program_error::ProgramError;

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub enum Square {
    Empty,
    X,
    O,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd)]
#[repr(i8)]
pub enum Turn {
    X = 1,
    O = -1,
}

impl Turn {
    pub fn other(&self) -> Self {
        match self {
            Self::X => Self::O,
            Self::O => Self::X,
        }
    }
}

impl From<Turn> for Square {
    fn from(value: Turn) -> Square {
        match value {
            Turn::O => Square::O,
            Turn::X => Square::X,
        }
    }
}

// probably has duel account + game index as seeds or something
#[derive(Debug)]
pub struct GameAccount {
    pub board: [[Square; 3]; 3],
    pub turn: Turn,
    pub game_over: bool,
}

//  0 | 1 | 2
// ---|---|---
//  3 | 4 | 5
// ---|---|---
//  6 | 7 | 8
pub fn index_to_row_col(square_index: u8) -> (usize, usize) {
    let row = (square_index / 3) as usize;
    let col = (square_index % 3) as usize;
    (row, col)
}

impl GameAccount {
    pub fn is_full(&self) -> bool {
        self.board.iter().flatten().all(|sq| sq != &Square::Empty)
    }

    pub fn make_move(&mut self, square_index: u8) -> Result<bool, ProgramError> {
        let (row, col) = index_to_row_col(square_index);
        if self.game_over {
            return Err(ProgramError::InvalidArgument);
        }
        if self.board[row][col] != Square::Empty {
            return Err(ProgramError::InvalidArgument);
        }
        let new_square = self.turn.into();
        self.board[row][col] = new_square;

        let (mut row_count, mut col_count) = (0, 0);
        for i in 0..3 {
            if self.board[row][i] == new_square {
                row_count += 1;
            }
            if self.board[i][col] == new_square {
                col_count += 1;
            }
        }
        if row_count == 3 || col_count == 3 {
            self.game_over = true;
            return Ok(true);
        }

        // on a diagonal
        if square_index % 2 == 0 {
            let (mut r_diag, mut l_diag) = (0, 0);
            for i in 0..3 {
                if self.board[i][i] == new_square {
                    r_diag += 1;
                }
                if self.board[i][2 - i] == new_square {
                    l_diag += 1;
                }
            }
            if r_diag == 3 || l_diag == 3 {
                self.game_over = true;
                return Ok(true);
            }
        }
        self.turn = self.turn.other();
        Ok(false)
    }
}

// account_data_shell!(GameAccount, TicTacToeProgram, 1);

impl AccountDataValidate<()> for GameAccount {
    fn validate(_data: &Self::NewestVersion, _arg: ()) -> Result<(), ProgramError> {
        todo!()
    }
}

impl AccountDataInit<()> for GameAccount {
    fn init(bytes: &mut AccountInfoDataSection, init_arg: ()) -> Result<(), ProgramError> {
        let duel_account = GameAccount::from_bytes_mut(&mut bytes.data_bytes_mut())?;

        *duel_account = GameAccount {
            board: [[Square::Empty; 3]; 3],
            turn: Turn::X,
            game_over: false,
        };

        Ok(())
    }
}

impl AccountData for GameAccount {
    type NewestVersion = Self;
    type UpgradeArg<'a> = ();
    type Program = TicTacToeProgram;

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
impl ProgramAccountEntry<GameAccount> for TicTacToeProgram {
    const DISCRIMINANT: u64 = 1;
}
