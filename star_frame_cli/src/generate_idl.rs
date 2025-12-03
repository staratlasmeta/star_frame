use std::process::Stdio;

pub fn generate_idl() -> eyre::Result<()> {
    println!("Generating IDL...");

    let exit = std::process::Command::new("cargo")
        .args(&["test", "--quiet", "--features", "idl", "--", "generate_idl"])
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| eyre::format_err!("Failed to run IDL generation: {}", e))?;

    if !exit.status.success() {
        eprintln!("IDL generation failed.");
        std::process::exit(exit.status.code().unwrap_or(1));
    }

    println!("IDL generated successfully!");
    Ok(())
}
