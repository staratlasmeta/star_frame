use std::{
    fs, io,
    path::{Path, PathBuf},
    process,
    time::{SystemTime, UNIX_EPOCH},
};

use clap::Parser;
use colored::*;
use convert_case::{Case, Casing};
use eyre::{bail, eyre, WrapErr};
use solana_keypair::{write_keypair_file, Keypair};
use solana_signer::Signer;

#[derive(Parser, Debug)]
pub struct NewArgs {
    /// The name of the program
    #[arg(value_name = "NAME")]
    pub name: String,
}

pub fn new_project(args: NewArgs) -> eyre::Result<()> {
    new_project_in(Path::new("."), args)
}

fn new_project_in(output_dir: &Path, args: NewArgs) -> eyre::Result<()> {
    let project_name = validate_program_name(args.name.trim())?;
    let destination = output_dir.join(&project_name);

    scaffold_project(&destination, &project_name)?;
    let keypair_path = program_keypair_relative_path(&project_name);

    println!(
        "{}",
        format!("{} program initialized", project_name)
            .green()
            .bold()
    );
    println!("Next steps:");
    println!("  cd {}", project_name);
    println!("  cargo build");
    println!("  cargo test");
    println!("  cargo build-sbf");
    println!("  cargo test-sbf");
    println!(
        "  cargo test --features idl  # writes target/idl/{}.json",
        project_name
    );
    println!("  # Program keypair: {}", keypair_path.display());

    Ok(())
}

fn scaffold_project(destination: &Path, project_name: &str) -> eyre::Result<()> {
    if destination.exists() {
        bail!(
            "Target path `{}` already exists. Choose a different name or remove the existing path.",
            destination.display()
        );
    }

    let staging_dir = staging_directory_for(destination)?;
    let program_keypair = Keypair::new();
    let values = TemplateValues::new(project_name, program_keypair.pubkey().to_string());

    let scaffold_result = (|| -> eyre::Result<()> {
        create_project_directories(&staging_dir).wrap_err_with(|| {
            format!(
                "Failed to create scaffold directories in `{}`",
                staging_dir.display()
            )
        })?;
        write_project_files(&staging_dir, &values).wrap_err_with(|| {
            format!(
                "Failed to write scaffold files in `{}`",
                staging_dir.display()
            )
        })?;
        write_program_keypair(&staging_dir, project_name, &program_keypair)?;
        Ok(())
    })();

    if let Err(err) = scaffold_result {
        let _ = fs::remove_dir_all(&staging_dir);
        return Err(err);
    }

    fs::rename(&staging_dir, destination)
        .inspect_err(|_err| {
            let _ = fs::remove_dir_all(&staging_dir);
        })
        .wrap_err_with(|| {
            format!(
                "Failed to move scaffold from `{}` to `{}`",
                staging_dir.display(),
                destination.display()
            )
        })?;

    Ok(())
}

fn staging_directory_for(destination: &Path) -> eyre::Result<PathBuf> {
    let parent = destination.parent().unwrap_or_else(|| Path::new("."));
    let name = destination
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| eyre!("Invalid destination path `{}`", destination.display()))?;
    let seed = unix_timestamp_nanos()?;

    for attempt in 0_u32..256 {
        let candidate = parent.join(format!(
            ".{}.sf-new-{}-{}",
            name,
            process::id(),
            seed + u128::from(attempt)
        ));
        match fs::create_dir(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(err) => {
                return Err(err).wrap_err_with(|| {
                    format!(
                        "Failed to create staging directory `{}`",
                        candidate.display()
                    )
                });
            }
        }
    }

    bail!(
        "Unable to allocate a temporary staging directory for `{}`",
        destination.display()
    )
}

fn create_project_directories(base: &Path) -> io::Result<()> {
    let cargo = base.join(".cargo");
    let src = base.join("src");
    let ixs = src.join("instructions");
    let tests = src.join("tests");

    for directory in [
        cargo.as_path(),
        src.as_path(),
        ixs.as_path(),
        tests.as_path(),
    ] {
        fs::create_dir_all(directory)?;
    }

    Ok(())
}

fn program_keypair_relative_path(project_name: &str) -> PathBuf {
    let artifact_name = project_name.replace('-', "_");
    Path::new("target")
        .join("deploy")
        .join(format!("{artifact_name}-keypair.json"))
}

fn write_program_keypair(base: &Path, project_name: &str, keypair: &Keypair) -> eyre::Result<()> {
    let keypair_path = base.join(program_keypair_relative_path(project_name));
    let keypair_directory = keypair_path
        .parent()
        .ok_or_else(|| eyre!("Invalid keypair path `{}`", keypair_path.display()))?;
    fs::create_dir_all(keypair_directory).wrap_err_with(|| {
        format!(
            "Failed to create keypair directory `{}`",
            keypair_directory.display()
        )
    })?;
    write_keypair_file(keypair, &keypair_path)
        .map(|_json| ())
        .map_err(|err| {
            eyre!(
                "Failed to write program keypair `{}`: {err}",
                keypair_path.display()
            )
        })?;

    Ok(())
}

fn write_project_files(base: &Path, values: &TemplateValues) -> io::Result<()> {
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

    let files = [
        (CARGO_TOML, base.join("Cargo.toml")),
        (GITIGNORE, base.join(".gitignore")),
        (README_MD, base.join("README.md")),
        (CONFIG_TOML, base.join(".cargo/config.toml")),
        (LIB_RS, base.join("src/lib.rs")),
        (STATES_RS, base.join("src/states.rs")),
        (INCREMENT_RS, base.join("src/instructions/increment.rs")),
        (INITIALIZE_RS, base.join("src/instructions/initialize.rs")),
        (INSTRUCTION_MOD_RS, base.join("src/instructions/mod.rs")),
        (TEST_RS, base.join("src/tests/counter.rs")),
        (TEST_MOD_RS, base.join("src/tests/mod.rs")),
    ];

    for (template, path) in files {
        write_template_file(template, &path, values)?;
    }

    Ok(())
}

fn write_template_file(template: &str, path: &Path, values: &TemplateValues) -> io::Result<()> {
    fs::write(path, render_template(template, values))
}

fn render_template(template: &str, values: &TemplateValues) -> String {
    template
        .replace("{name_lowercase}", &values.name_lowercase)
        .replace(
            "{name_lowercase_underscore}",
            &values.name_lowercase_underscore,
        )
        .replace("{name_uppercase}", &values.name_uppercase)
        .replace("{name_pascalcase}", &values.name_pascalcase)
        .replace("{pubkey}", &values.pubkey)
}

/// Validates a strict crate/module-safe subset of names:
/// `[a-z][a-z0-9_-]*`, no repeated or trailing separators, and not a Rust keyword
/// after normalizing `-` to `_`.
fn validate_program_name(name: &str) -> eyre::Result<String> {
    if name.is_empty() {
        return Err(invalid_name(name, "name cannot be empty"));
    }

    let mut chars = name.chars();
    let first_char = chars
        .next()
        .ok_or_else(|| invalid_name(name, "name cannot be empty"))?;

    if !first_char.is_ascii_lowercase() {
        return Err(invalid_name(
            name,
            "must start with a lowercase ASCII letter",
        ));
    }

    let mut previous_separator = false;
    for character in chars {
        match character {
            'a'..='z' | '0'..='9' => previous_separator = false,
            '-' | '_' => {
                if previous_separator {
                    return Err(invalid_name(
                        name,
                        "cannot contain consecutive '-' or '_' separators",
                    ));
                }
                previous_separator = true;
            }
            _ => {
                return Err(invalid_name(
                    name,
                    "can only include lowercase letters, digits, '-' or '_'",
                ));
            }
        }
    }

    if previous_separator {
        return Err(invalid_name(name, "cannot end with '-' or '_'"));
    }

    let module_name = name.replace('-', "_");
    if is_rust_keyword(&module_name) {
        return Err(invalid_name(
            name,
            "cannot be a Rust keyword once '-' is normalized to '_'",
        ));
    }

    Ok(name.to_owned())
}

fn invalid_name(name: &str, reason: &str) -> eyre::Report {
    let display_name = if name.is_empty() { "<empty>" } else { name };
    eyre!(
        "Invalid program name `{display_name}`: {reason}. Use a crate-safe name like `counter_program` or `counter-program`."
    )
}

fn is_rust_keyword(value: &str) -> bool {
    matches!(
        value,
        "as" | "break"
            | "const"
            | "continue"
            | "crate"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "async"
            | "await"
            | "dyn"
            | "abstract"
            | "become"
            | "box"
            | "do"
            | "final"
            | "macro"
            | "override"
            | "priv"
            | "typeof"
            | "unsized"
            | "virtual"
            | "yield"
            | "try"
    )
}

fn unix_timestamp_nanos() -> eyre::Result<u128> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| eyre!("System clock is before UNIX_EPOCH: {err}"))?
        .as_nanos())
}

struct TemplateValues {
    name_lowercase: String,
    name_lowercase_underscore: String,
    name_uppercase: String,
    name_pascalcase: String,
    pubkey: String,
}

impl TemplateValues {
    fn new(project_name: &str, pubkey: String) -> Self {
        Self {
            name_lowercase: project_name.to_owned(),
            name_lowercase_underscore: project_name.replace('-', "_"),
            name_uppercase: project_name.to_ascii_uppercase(),
            name_pascalcase: project_name.to_case(Case::Pascal),
            pubkey,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_keypair::read_keypair_file;
    use solana_signer::Signer;

    #[test]
    fn validates_expected_program_names() {
        for name in ["counter", "counter_program", "counter-program", "counter2"] {
            assert_eq!(validate_program_name(name).unwrap(), name);
        }
    }

    #[test]
    fn rejects_invalid_program_names() {
        for invalid in [
            "",
            "Counter",
            "9counter",
            "counter--program",
            "counter__program",
            "counter-",
            "counter_",
            "counter!",
            "fn",
            "self",
            "counter program",
        ] {
            let err = validate_program_name(invalid).unwrap_err();
            assert!(
                err.to_string().contains("Invalid program name"),
                "missing error context for `{invalid}`: {err:#}"
            );
        }
    }

    #[test]
    fn rejects_existing_destination() {
        let temp_dir = TestDir::new("sf-new-existing");
        let existing = temp_dir.path().join("counter");
        fs::create_dir(&existing).unwrap();

        let err = new_project_in(
            temp_dir.path(),
            NewArgs {
                name: "counter".to_owned(),
            },
        )
        .unwrap_err();

        assert!(
            err.to_string().contains("already exists"),
            "unexpected error: {err:#}"
        );
    }

    #[test]
    fn scaffolds_project_with_expected_templates() {
        let temp_dir = TestDir::new("sf-new-scaffold");

        new_project_in(
            temp_dir.path(),
            NewArgs {
                name: "counter-program".to_owned(),
            },
        )
        .unwrap();

        let project_dir = temp_dir.path().join("counter-program");
        assert!(project_dir.join("Cargo.toml").exists());
        assert!(project_dir.join("src/lib.rs").exists());
        assert!(project_dir.join("src/tests/counter.rs").exists());
        let keypair_path = project_dir.join(program_keypair_relative_path("counter-program"));
        assert!(keypair_path.exists());

        let cargo_toml = fs::read_to_string(project_dir.join("Cargo.toml")).unwrap();
        assert!(cargo_toml.contains("crate-type = [\"cdylib\", \"lib\"]"));
        assert!(cargo_toml.contains("az = \"=1.2.1\""));
        assert!(cargo_toml
            .contains("TODO(star_frame): remove this pin once build-sbf toolchains support Cargo"));
        assert!(cargo_toml.contains("mollusk-svm = \"=0.7.0\""));
        assert!(cargo_toml.contains("solana-account = \"3.0.0\""));
        assert!(!cargo_toml.contains("codama-nodes"));

        let counter_test = fs::read_to_string(project_dir.join("src/tests/counter.rs")).unwrap();
        assert!(!counter_test.contains("SF_RUN_SBF_TESTS"));
        assert!(counter_test.contains("let authority = Pubkey::new_unique();"));
        assert!(counter_test.contains("Run `cargo build-sbf` first"));
        assert!(counter_test.contains("target_dir.join(\"idl\")"));
        assert!(!counter_test.contains("std::fs::write(\"idl.json\""));

        let lib_rs = fs::read_to_string(project_dir.join("src/lib.rs")).unwrap();
        let id = extract_program_id(&lib_rs).expect("missing generated program id");
        let keypair = read_keypair_file(keypair_path).unwrap();
        assert_eq!(id, keypair.pubkey().to_string());
    }

    #[test]
    fn generates_distinct_program_ids() {
        let temp_dir = TestDir::new("sf-new-distinct-ids");

        new_project_in(
            temp_dir.path(),
            NewArgs {
                name: "alpha".to_owned(),
            },
        )
        .unwrap();
        new_project_in(
            temp_dir.path(),
            NewArgs {
                name: "beta".to_owned(),
            },
        )
        .unwrap();

        let alpha_id = read_program_id(&temp_dir.path().join("alpha"));
        let beta_id = read_program_id(&temp_dir.path().join("beta"));
        assert_ne!(alpha_id, beta_id);
    }

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new(prefix: &str) -> Self {
            let seed = unix_timestamp_nanos().unwrap();
            for attempt in 0_u32..256 {
                let candidate = std::env::temp_dir().join(format!(
                    "{}-{}-{}-{}",
                    prefix,
                    process::id(),
                    seed,
                    attempt
                ));
                if fs::create_dir(&candidate).is_ok() {
                    return Self { path: candidate };
                }
            }

            panic!("failed to allocate temp test dir for prefix `{prefix}`");
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn read_program_id(project_dir: &Path) -> String {
        let lib_rs = fs::read_to_string(project_dir.join("src/lib.rs")).unwrap();
        extract_program_id(&lib_rs)
            .unwrap_or_else(|| panic!("missing program id in `{}`", project_dir.display()))
    }

    fn extract_program_id(lib_rs: &str) -> Option<String> {
        lib_rs.lines().find_map(|line| {
            let trimmed = line.trim();
            trimmed
                .strip_prefix("id = \"")
                .and_then(|value| value.strip_suffix('"'))
                .map(str::to_owned)
        })
    }
}
