use clap::{Parser, Subcommand};
pub mod build_project;
pub mod generate_idl;
pub mod new_project;
use build_project::*;
use generate_idl::*;
use new_project::*;

#[derive(Subcommand, Debug)]
enum CliCommand {
    #[command(about = "Create new Solana program")]
    New(NewArgs),
    #[command(about = "Build the Solana program")]
    Build,
    #[command(about = "Generate the Solana program IDL")]
    Idl,
}

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: CliCommand,
}

fn main() -> eyre::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        CliCommand::New(args) => new_project(args),
        CliCommand::Build => new_build(),
        CliCommand::Idl => generate_idl(),
    }
}
