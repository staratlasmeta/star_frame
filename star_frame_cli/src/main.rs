mod idl;
mod solana;

use crate::idl::IdlArgs;
use crate::solana::SolanaArgs;
use anyhow::Result;
use clap::{Parser, Subcommand};
use lazy_static::lazy_static;

pub trait HandleCommand {
    type Super<'a>;

    fn handle(&self, super_command: Self::Super<'_>) -> Result<()>;
}

#[derive(Parser)]
pub struct StarFrameArgs {
    #[clap(subcommand)]
    pub command: Command,
}
impl HandleCommand for StarFrameArgs {
    type Super<'a> = ();

    fn handle(&self, _super_command: Self::Super<'_>) -> Result<()> {
        match &self.command {
            Command::Idl(idl_args) => idl_args.handle(self),
            Command::Solana(solana_args) => solana_args.handle(self),
        }
    }
}

#[derive(Subcommand)]
pub enum Command {
    /// Functions interacting with IDLs.
    Idl(IdlArgs),
    /// Interact with solana cli
    Solana(SolanaArgs),
}

lazy_static! {
    static ref ARGS: StarFrameArgs = StarFrameArgs::parse();
}

fn main() -> Result<()> {
    ARGS.handle(())
}
