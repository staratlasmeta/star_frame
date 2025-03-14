use crate::idl::IdlArgs;
use crate::{HandleCommand, StarFrameArgs};
use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct IdlGenerateArgs {
    /// Directory to output IDLs to.
    #[clap(long, short)]
    pub out_dir: Option<PathBuf>,

    /// Directory of the cargo manifest to build.
    #[clap(long)]
    pub manifest_dir: Option<PathBuf>,

    /// Args to pass to cargo.
    #[clap(long, short)]
    pub cargo_args: Option<String>,
}

impl HandleCommand for IdlGenerateArgs {
    type Super<'a> = (&'a IdlArgs, &'a StarFrameArgs);

    fn handle(&self, _super_command: Self::Super<'_>) -> anyhow::Result<()> {
        println!("Self: {:?}", self);
        Ok(())
    }
}
