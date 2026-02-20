# Star Frame

A high-performance, trait-based Solana program framework for building **fast**, **reliable**, and **type-safe** programs.

[![crates.io](https://img.shields.io/crates/v/star_frame?logo=rust)](https://crates.io/crates/star_frame)
[![docs.rs](https://img.shields.io/docsrs/star_frame?logo=docsdotrs)](https://docs.rs/star_frame)

## Why Star Frame?

Star Frame is a modern alternative to Anchor for Solana program development. Key advantages:

- **60-93% fewer compute units** than Anchor (see [benchmarks](#benchmarks))
- **~50% smaller binaries** than equivalent Anchor programs
- **Full type safety** — traits and types all the way down, catching errors at compile time
- **Built on Pinocchio** — zero-copy account access, no deserialization overhead
- **Anchor-compatible discriminators** by default — easy migration path

## Quick Start

### Option 1: CLI (Recommended)

```bash
cargo install star_frame_cli
sf new my_program
cd my_program
```

### Option 2: Manual Setup

```bash
cargo init --lib my_program
cd my_program
cargo add star_frame bytemuck borsh
```

Add to your `Cargo.toml`:

```toml
[lib]
crate-type = ["cdylib", "lib"]

[features]
idl = ["star_frame/idl"]
test_helpers = ["star_frame/test_helpers"]
```

## Hello World: A Simple Counter

```rust
use star_frame::prelude::*;

// 1. Define your program
#[derive(StarFrameProgram)]
#[program(
    instruction_set = CounterInstructionSet,
    id = "YourProgramId11111111111111111111111111111111"
)]
pub struct CounterProgram;

// 2. Define instructions
#[derive(InstructionSet)]
pub enum CounterInstructionSet {
    Initialize(Initialize),
    Increment(Increment),
}

// 3. Define account state (zero-copy for performance)
#[zero_copy(pod)]
#[derive(ProgramAccount, Default, Debug, Eq, PartialEq)]
pub struct CounterAccount {
    pub authority: Pubkey,
    pub count: u64,
}

// 4. Define instruction data
#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct Initialize {
    #[ix_args(run)]
    pub start_at: u64,
}

// 5. Define account sets with validation
#[derive(AccountSet)]
pub struct InitializeAccounts {
    #[validate(funder)]
    pub authority: Signer<Mut<SystemAccount>>,
    #[validate(arg = Create(()))]
    pub counter: Init<Signer<Account<CounterAccount>>>,
    pub system_program: Program<System>,
}

// 6. Write instruction logic
#[star_frame_instruction]
fn Initialize(accounts: &mut InitializeAccounts, start_at: u64) -> Result<()> {
    **accounts.counter.data_mut()? = CounterAccount {
        authority: *accounts.authority.pubkey(),
        count: start_at,
    };
    Ok(())
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Copy, Clone, InstructionArgs)]
pub struct Increment;

#[derive(AccountSet, Debug)]
pub struct IncrementAccounts {
    pub authority: Signer,
    pub counter: Mut<Account<CounterAccount>>,
}

#[star_frame_instruction]
fn Increment(accounts: &mut IncrementAccounts) -> Result<()> {
    ensure!(
        *accounts.authority.pubkey() == accounts.counter.data()?.authority,
        ProgramError::IncorrectAuthority
    );
    let mut counter = accounts.counter.data_mut()?;
    counter.count += 1;
    Ok(())
}
```

## Building

```bash
# Build for Solana
cargo build-sbf

# Run tests
cargo test --features test_helpers

# Generate IDL
cargo test --features idl -- generate_idl
```

## Benchmarks

Compared to Anchor (Solana v2.1.0):

| Operation | Star Frame | vs Anchor |
|-----------|-----------|-----------|
| 1 AccountInfo | 166 CU | **-71% CU** |
| 8 AccountInfo | 277 CU | **-91% CU** |
| 1 Account Init | 1,984 CU | **-61% CU** |
| 8 Account Init | 13,178 CU | **-62% CU** |
| 1 Account Read | 191 CU | **-72% CU** |
| 8 Account Read | 480 CU | **-86% CU** |
| 1 Token Account | 215 CU | **-90% CU** |
| Binary Size | 528 KB | **-49%** |

## Project Structure

```
my_program/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Program, InstructionSet, accounts, instructions
│   └── instructions/       # (optional) split instructions into modules
│       ├── mod.rs
│       ├── initialize.rs
│       └── increment.rs
└── tests/
```

## Feature Flags

| Feature | Description |
|---------|-------------|
| `idl` | Enables Codama IDL generation |
| `test_helpers` | Testing utilities (required when tests use Star Frame's built-in test helpers, e.g. `MockContext`) |
| `cleanup_rent_warning` | Warns if account has excess lamports after cleanup |
| `aggressive_inline` | More aggressive inlining (may increase binary size) |

## Pre-Deploy Checklist

- [ ] All funders use `Signer<Mut<SystemAccount>>` (not just `Signer<SystemAccount>`)
- [ ] `Init` account sets include `Program<System>`
- [ ] No `bool` in zero-copy (`#[zero_copy(pod)]`) types — use `u8`
- [ ] PDA seeds produce unique addresses (no hardcoded/default seed values)
- [ ] `CloseAccount` cleanup has a `#[validate(recipient)]` tagged account
- [ ] No Edition 2024 transitive dependencies (check with `cargo tree`)
- [ ] Instruction variant names are stable (renaming changes discriminants)
- [ ] Unsized accounts use `#[cleanup(arg = NormalizeRent(()))]`

## Documentation

- [Architecture Guide](ARCHITECTURE.md) — How Star Frame works internally
- [Developer Guide](GUIDE.md) — Step-by-step tutorial with a complete example
- [API Reference](API_REFERENCE.md) — Traits, types, macros reference
- [Anchor Comparison](ANCHOR_COMPARISON.md) — Migration guide for Anchor developers
- [Production Learnings](LEARNINGS.md) — Practical lessons from production usage

## Links

- [API Docs (docs.rs)](https://docs.rs/star_frame)
- [GitHub Repository](https://github.com/staratlasmeta/star_frame)
- [Examples](https://github.com/staratlasmeta/star_frame/tree/main/example_programs)

## License

Apache-2.0
