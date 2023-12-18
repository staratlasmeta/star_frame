#![feature(const_convert)]
#![feature(const_trait_impl)]

use framework_test::{this_program_id, Program};
use solana_program::pubkey::Pubkey;

mod instructions;
mod state;

pub struct TicTacToeProgram;

#[cfg(target_os = "solana")]
framework_test::framework_entrypoint!(TicTacToeProgram);

impl Program for TicTacToeProgram {
    type InstructionSet<'a> = instructions::TicTacToeInstruction;
    type DiscriminantType = u8;

    fn program_id() -> &'static Pubkey {
        this_program_id()
    }
}
