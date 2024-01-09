use crate::{HandleCommand, StarFrameArgs};
use clap::{Args, Subcommand};
use generate::IdlGenerateArgs;

mod generate;

#[derive(Args)]
pub struct IdlArgs {
    #[clap(subcommand)]
    pub command: IdlCommand,
}

#[derive(Subcommand)]
pub enum IdlCommand {
    /// Generate IDLs from source.
    Generate(IdlGenerateArgs),
}

impl HandleCommand for IdlArgs {
    type Super<'a> = &'a StarFrameArgs;

    fn handle(&self, super_command: Self::Super<'_>) -> anyhow::Result<()> {
        match &self.command {
            IdlCommand::Generate(generate) => generate.handle((self, super_command)),
        }
    }
}
