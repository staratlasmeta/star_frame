use std::process::{Command, Stdio};

use clap::{command, Parser, Subcommand};
pub mod new_project;
use new_project::*;

#[derive(Subcommand, Debug)]
enum CliCommand {
    #[command(about = "Create new Solana program")]
    New(NewArgs),

    #[command(about = "Compile/build program")]
    Build,

    #[command(about = "Execute all tests")]
    Test(TestArgs),
}

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Parser, Debug)]
pub struct TestArgs {
    /// Show test output (don’t capture stdout/stderr)
    #[arg(long)]
    pub nocapture: bool,
}

pub fn test_project(args: TestArgs) -> anyhow::Result<()> {
    let mut command = Command::new("cargo");
    command.arg("test-sbf");

    if args.nocapture {
        command.arg("--").arg("--nocapture");
    }

    command
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("Failed to execute command");
    Ok(())
}

pub fn build_project() -> anyhow::Result<()> {
    Command::new("cargo")
        .arg("build-sbf")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("Failed to execute command");
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        CliCommand::New(args) => new_project(args),
        CliCommand::Build => build_project(),
        CliCommand::Test(args) => test_project(args),
    }
}
