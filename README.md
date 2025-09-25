<h1 align="center">
  <code>star_frame</code>
</h1>
<p align="center">
  A high-performance, trait-based Solana program framework for building <strong>fast</strong>, <strong>reliable</strong>, and <strong>type-safe</strong> programs.
</p>

<p align="center">
  <a href="https://crates.io/crates/star_frame"><img src="https://img.shields.io/crates/v/star_frame?logo=rust" /></a>
  <a href="https://docs.rs/star_frame"><img src="https://img.shields.io/docsrs/star_frame?logo=docsdotrs" /></a>
  <a href="https://github.com/staratlasmeta/star_frame/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/staratlasmeta/star_frame/ci.yml?logo=GitHub" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache%202.0-blue" /></a>
</p>

## Overview

Star Frame is a modern Solana program framework designed to make developing on-chain programs more ergonomic, safe, and performant. Built with a trait-based architecture, it provides:

- **Performance**: Optimized for Solana's compute unit constraints by utilizing Pinocchio and our `unsized_type` system (check out the [Compute Units](example_programs/bench/COMPUTE_UNITS.md) benchmark vs Anchor).
- **Developer Experience**: Intuitive APIs with comprehensive compile-time validation (traits and types all the way down!).
- **Modularity**: Everything is a trait or a type, so you can use what you need when you need it. For example, the entrypoint is a method on the `StarFrameProgram` trait, and client/cpi account sets are associated types of the `ClientAccountSet` and `CpiAccountSet` traits.

## Getting Help

Star Frame is in active development (and improving our docs is a main priority now!). If you need help:

- Check out the [API documentation](https://docs.rs/star_frame)
- Browse the [examples](example_programs/) in this repository
- Open an [issue](https://github.com/staratlasmeta/star_frame/issues) for bug reports or feature requests
- Join our [Star Atlas Discord](https://discord.gg/gahmBHsc) and chat in our `#community-developers` channel

## Getting Started

### Create a new project using the CLI

```bash
cargo install star_frame_cli
sf --help
```

```bash
sf new <PROJECT-NAME>
```

### Update an existing project to use Star Frame

Add `star_frame` and `bytemuck` to your `Cargo.toml`:

```shell
cargo add star_frame bytemuck
```

Use the `prelude` to import the most commonly used traits and macros:

```rs
use star_frame::prelude::*;
```

## Example

Below is a simple counter program demonstrating the basic features of Star Frame. In this example, only the designated authority can increment the counter.

```rust
use star_frame::{anyhow::ensure, prelude::*};

#[derive(StarFrameProgram)]
#[program(
    instruction_set = CounterInstructionSet,
    id = "Coux9zxTFKZpRdFpE4F7Fs5RZ6FdaURdckwS61BUTMG"
)]
pub struct CounterProgram;

#[derive(InstructionSet)]
pub enum CounterInstructionSet {
    Initialize(Initialize),
    Increment(Increment),
}

#[zero_copy(pod)]
#[derive(ProgramAccount, Default, Debug, Eq, PartialEq)]
#[program_account(seeds = CounterSeeds)]
pub struct CounterAccount {
    pub authority: Pubkey,
    pub count: u64,
}

#[derive(Debug, GetSeeds, Clone)]
#[get_seeds(seed_const = b"COUNTER")]
pub struct CounterSeeds {
    pub authority: Pubkey,
}

#[derive(Debug)]
pub struct Authority(Pubkey);

impl AccountValidate<Authority> for CounterAccount {
    fn validate_account(self_ref: &Self::Ref<'_>, arg: Authority) -> Result<()> {
        ensure!(arg.0 == self_ref.authority, "Incorrect authority", ProgramError::IncorrectAuthority);
        Ok(())
    }
}

/// Initialize the counter
#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct Initialize {
    #[ix_args(&run)]
    pub start_at: Option<u64>,
}

#[derive(AccountSet)]
pub struct InitializeAccounts {
    #[validate(funder)]
    pub authority: Signer<Mut<SystemAccount>>,
    #[validate(arg = (
        Create(()),
        Seeds(CounterSeeds { authority: *self.authority.pubkey() }),
    ))]
    #[idl(arg = Seeds(FindCounterSeeds { authority: seed_path("authority") }))]
    pub counter: Init<Seeded<Account<CounterAccount>>>,
    pub system_program: Program<System>,
}

#[star_frame_instruction]
fn Initialize(account_set: &mut InitializeAccounts, start_at: &Option<u64>) -> Result<()> {
    **account_set.counter.data_mut()? = CounterAccount {
        authority: *account_set.authority.pubkey(),
        count: start_at.unwrap_or(0),
    };
    Ok(())
}

/// Increment the counter by 1
#[derive(BorshSerialize, BorshDeserialize, Debug, Copy, Clone, InstructionArgs)]
pub struct Increment;

#[derive(AccountSet, Debug)]
pub struct IncrementAccounts {
    pub authority: Signer,
    #[validate(arg = Authority(*self.authority.pubkey()))]
    pub counter: Mut<ValidatedAccount<CounterAccount>>,
}

#[star_frame_instruction]
fn Increment(account_set: &mut IncrementAccounts) -> Result<()> {
    let mut counter = account_set.counter.data_mut()?;
    counter.count += 1;
    Ok(())
}
```

## Supported Rust Versions

Star Frame is built against the latest stable Solana Rust Release: https://github.com/anza-xyz/rust. The minimum supported version is currently **1.84.1**.

## License

This project is licensed under the [Apache-2.0](LICENSE) license.
