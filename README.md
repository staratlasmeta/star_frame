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

- **Type Safety**: Zero-cost abstractions that catch errors at compile time
- **Performance**: Optimized for Solana's compute unit constraints by utilizing Pinocchio and our unsized_type system
- **Developer Experience**: Intuitive APIs with comprehensive compile-time validation
- **Modularity**: Composable components for account validation, CPI calls, and program logic

## Getting Help

Star Frame is in active development (and improving our docs is a main priority now!). If you need help:

- Check out the [API documentation](https://docs.rs/star_frame)
- Browse the [examples](example_programs/) in this repository
- Open an [issue](https://github.com/staratlasmeta/star_frame/issues) for bug reports or feature requests
- Join our [Star Atlas Discord](https://discord.gg/gahmBHsc) and chat in our `#community-developers` channel

## Getting Started

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

#[derive(Align1, Pod, Zeroable, Default, Copy, Clone, Debug, Eq, PartialEq, ProgramAccount)]
#[program_account(seeds = CounterSeeds)]
#[repr(C, packed)]
pub struct CounterAccount {
    pub authority: Pubkey,
    pub count: u64,
}

#[derive(Debug, GetSeeds, Clone)]
#[get_seeds(seed_const = b"COUNTER")]
pub struct CounterSeeds {
    pub authority: Pubkey,
}

impl AccountValidate<&Pubkey> for CounterAccount {
    fn validate(self_ref: &Self::Ref<'_>, arg: &Pubkey) -> Result<()> {
        ensure!(arg == &self_ref.authority, "Incorrect authority");
        Ok(())
    }
}

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

impl StarFrameInstruction for Initialize {
    type ReturnType = ();
    type Accounts<'b, 'c> = InitializeAccounts;

    fn process(
        accounts: &mut Self::Accounts<'_, '_>,
        start_at: &Option<u64>,
        _ctx: &mut Context,
    ) -> Result<Self::ReturnType> {
        **accounts.counter.data_mut()? = CounterAccount {
            authority: *accounts.authority.pubkey(),
            count: start_at.unwrap_or(0),
        };
        Ok(())
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Copy, Clone, InstructionArgs)]
pub struct Increment;

#[derive(AccountSet, Debug)]
pub struct IncrementAccounts {
    pub authority: Signer,
    #[validate(arg = self.authority.pubkey())]
    pub counter: Mut<ValidatedAccount<CounterAccount>>,
}

impl StarFrameInstruction for Increment {
    type ReturnType = ();
    type Accounts<'b, 'c> = IncrementAccounts;

    fn process(
        accounts: &mut Self::Accounts<'_, '_>,
        _run_arg: Self::RunArg<'_>,
        _ctx: &mut Context,
    ) -> Result<Self::ReturnType> {
        let mut counter = accounts.counter.data_mut()?;
        counter.count += 1;
        Ok(())
    }
}
```

## Supported Rust Versions

Star Frame is built against the latest stable Solana Rust Release: https://github.com/anza-xyz/rust. The minimum supported version is currently **1.84.1**.

## License

This project is licensed under the [Apache-2.0](LICENSE) license.
