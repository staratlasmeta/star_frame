mod create_game;
pub use create_game::*;
mod create_duel;
pub use create_duel::*;
mod make_move;
pub use make_move::*;

use crate::instructions::create_duel::CreateDuel;
use solana_program::entrypoint::ProgramResult;
use star_frame::instruction::{InstructionSet, ToBytes};
use std::fmt::Debug;

use advance::AdvanceArray;

#[repr(u8)]
#[derive(InstructionSet, Debug)]
pub enum TicTacToeInstruction {
    CreateDuel(CreateDuel) = 0,
    CreateGame(CreateGame) = 1,
    MakeMove(MakeMove) = 2,
    // EndGame(()) = 3,
    // EndMatch(()) = 4,
}

impl ToBytes for TicTacToeInstruction {
    fn to_bytes(&self, _output: &mut &mut [u8]) -> ProgramResult {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        // let mut bytes = vec![];
        // let instruction = TicTacToeInstruction::CreateGame();
        // instruction.serialize(&mut bytes).unwrap();
    }
}
