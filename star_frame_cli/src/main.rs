use clap::{command, Parser, Subcommand};
pub mod new_project;
use new_project::*;

#[derive(Subcommand, Debug)]
enum CliCommand {
    #[command(about = "Create new Solana program")]
    New(NewArgs),
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
    }
}
