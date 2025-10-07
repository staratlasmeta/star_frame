use std::{
    fs, io,
    path::{Path, PathBuf},
};

use clap::{arg, Parser};
use colored::*;
use convert_case::{Case, Casing};
use solana_address::Address;

#[derive(Parser, Debug)]
pub struct NewArgs {
    ///The name of the program
    #[arg(value_name = "NAME")]
    pub name: String,
}
pub fn new_project(args: NewArgs) -> eyre::Result<()> {
    let project_name = args.name.trim().to_ascii_lowercase();

    let base = Path::new(&project_name); //Base path
    let cargo = base.join(".cargo");
    let src = base.join("src");
    let ixs = src.join("instructions");
    let tests = src.join("tests");

    for dir in [&base.to_path_buf(), &cargo, &src, &ixs, &tests] {
        fs::create_dir_all(dir)?;
    }

    // Embed templates
    const CARGO_TOML: &str = include_str!("template/cargo_toml");
    const GITIGNORE: &str = include_str!("template/gitignore");
    const README_MD: &str = include_str!("template/readme_md");
    const CONFIG_TOML: &str = include_str!("template/config_toml");
    const LIB_RS: &str = include_str!("template/lib_rs");
    const STATES_RS: &str = include_str!("template/states_rs");
    const INCREMENT_RS: &str = include_str!("template/increment_rs");
    const INITIALIZE_RS: &str = include_str!("template/initialize_rs");
    const INSTRUCTION_MOD_RS: &str = include_str!("template/instruction_mod_rs");
    const TEST_RS: &str = include_str!("template/counter_test_rs");
    const TEST_MOD_RS: &str = include_str!("template/test_mod_rs");

    // Batch-render & write all files
    let files = [
        (CARGO_TOML, base.join("Cargo.toml")),
        (GITIGNORE, base.join(".gitignore")),
        (README_MD, base.join("README.md")),
        (CONFIG_TOML, cargo.join("config.toml")),
        (LIB_RS, src.join("lib.rs")),
        (STATES_RS, src.join("states.rs")),
        (INCREMENT_RS, ixs.join("increment.rs")),
        (INITIALIZE_RS, ixs.join("initialize.rs")),
        (INSTRUCTION_MOD_RS, ixs.join("mod.rs")),
        (TEST_RS, tests.join("counter.rs")),
        (TEST_MOD_RS, tests.join("mod.rs")),
    ];

    for (template, relative_path) in files {
        stub_file(template, &relative_path, &project_name)?;
    }

    println!(
        "{}",
        format!("{} program initialized", project_name)
            .green()
            .bold()
    );
    Ok(())
}

fn stub_file(template: &str, path: &PathBuf, project_name: &String) -> io::Result<()> {
    let content = template
        .replace("{name_lowercase}", &project_name.to_ascii_lowercase())
        .replace("{name_uppercase}", &project_name.to_ascii_uppercase())
        .replace("{name_pascalcase}", &project_name.to_case(Case::Pascal))
        .replace("{pubkey}", &Address::new_unique().to_string());
    fs::write(path, content)?;
    Ok(())
}
