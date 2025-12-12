use std::process::Stdio;

pub fn new_build() -> eyre::Result<()> {
    let exit = std::process::Command::new("cargo")
        .arg("build-sbf")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| eyre::format_err!("{}", e.to_string()))?;
    if !exit.status.success() {
        std::process::exit(exit.status.code().unwrap_or(1));
    }
    Ok(())
}
