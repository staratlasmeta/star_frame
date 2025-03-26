use crate::solana::SolanaArgs;
use crate::{HandleCommand, StarFrameArgs};
use anyhow::anyhow;
use clap::Args;
use std::process::Command;

#[derive(Args)]
pub struct SolanaInstallArgs {
    /// Version of solana to install.
    pub version: String,
    /// Automatically agree to install, no user interaction required.
    #[clap(short = 'y', default_value_t = false)]
    pub agree: bool,
}
impl HandleCommand for SolanaInstallArgs {
    type Super<'a> = (&'a SolanaArgs, &'a StarFrameArgs);

    fn handle(&self, _super_command: Self::Super<'_>) -> anyhow::Result<()> {
        println!("Installing solana version {}", &self.version);
        let command = || {
            let mut solana_install_command = Command::new("solana-install");
            solana_install_command.arg("init").arg(&self.version);
            solana_install_command
        };

        let status = command().status();
        let status = match status {
            Ok(status) => status,
            Err(e) => {
                if e.kind() != std::io::ErrorKind::NotFound {
                    return Err(anyhow!("Failed to run `solana-install`: {}", e));
                }
                if self.agree {
                    println!("`solana-install` not found. Installing...");
                } else {
                    println!("`solana-install` not found. Install? [y/n]");
                    let mut input = String::new();
                    loop {
                        input.clear();
                        std::io::stdin().read_line(&mut input)?;
                        match input.trim() {
                            "y" => break,
                            "n" => return Ok(()),
                            _ => println!("Invalid input. Install? [y/n]"),
                        }
                    }
                }
                #[cfg(target_os = "windows")]
                {
                    return Err(anyhow!("Windows is not supported for install"));
                }
                #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
                {
                    return Err(anyhow!("Unsupported OS for install"));
                }
                #[cfg(any(target_os = "linux", target_os = "macos"))]
                {
                    let mut install_solana_command: Command;
                    install_solana_command = Command::new("sh");
                    install_solana_command.arg("-c").arg(format!(
                        "$(curl -sSfL https://release.solana.com/v{}/install)",
                        self.version
                    ));
                    println!("Running command {:?}", install_solana_command);
                    let status = install_solana_command
                        .status()
                        .map_err(|e| anyhow!("Failed to install solana tools: {}", e))?;
                    if status.success() {
                        println!("Successfully installed solana version {}", &self.version);
                    } else {
                        return Err(anyhow!("Failed to install solana tools"));
                    }
                    command().status().map_err(|e| {
                        anyhow!("Failed to run `solana-install` after install: {}", e)
                    })?
                }
            }
        };

        if status.success() {
            println!("Successfully installed solana version {}", &self.version);
        } else {
            println!("Failed to install solana version {}", &self.version);
        }

        Ok(())
    }
}
