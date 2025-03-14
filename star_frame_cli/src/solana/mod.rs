mod install;

use crate::solana::install::SolanaInstallArgs;
use crate::{HandleCommand, StarFrameArgs};
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct SolanaArgs {
    #[clap(subcommand)]
    pub command: SolanaCommand,
}

#[derive(Subcommand)]
pub enum SolanaCommand {
    /// Installs a version of solana.
    Install(SolanaInstallArgs),
}

impl HandleCommand for SolanaArgs {
    type Super<'a> = &'a StarFrameArgs;

    fn handle(&self, super_command: Self::Super<'_>) -> anyhow::Result<()> {
        match &self.command {
            SolanaCommand::Install(install_args) => install_args.handle((self, super_command)),
        }
    }
}
