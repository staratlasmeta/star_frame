//! A high-performance, trait-based Solana program framework for building **fast**,
//! **reliable**, and **type-safe** programs.
//!
//! Star Frame provides a modern approach to Solana program development with zero-cost
//! abstractions, comprehensive compile-time validation, and an intuitive API design
//! optimized for Solana's compute unit constraints.
//!
//! ## Key Features
//!
//! - **Type Safety**: Catch errors at compile time with zero-cost abstractions
//! - **Performance**: Optimized for Solana's compute unit constraints
//! - **Developer Experience**: Intuitive APIs with comprehensive validation
//! - **Modularity**: Composable components for accounts, instructions, and program logic
//!
//! # Getting Started
//!
//! Add `star_frame` and `bytemuck` to your `Cargo.toml`:
//!
//! ```shell
//! cargo add star_frame bytemuck
//! ```
//!
//! # Lifecycle of a Star Frame Transaction
//!
//! Here's what happens when you call a into a Star Frame program:
//!
//! ## Program Entrypoint
//!
//! The [`StarFrameProgram`](crate::program::StarFrameProgram) derive macro generates the program entrypoint:
//!
//! ```
//! # fn main() {}
//! use star_frame::prelude::*;
//! # type CounterInstructionSet = ();
//! #[derive(StarFrameProgram)]
//! #[program(
//!     instruction_set = CounterInstructionSet,
//!     id = "Coux9zxTFKZpRdFpE4F7Fs5RZ6FdaURdckwS61BUTMG",
//! )]
//! pub struct CounterProgram;
//! ```
//!
//! The [`StarFrameProgram::entrypoint`](crate::program::StarFrameProgram::entrypoint)'s default implementation calls in to the
//! [`InstructionSet::dispatch`](crate::instruction::InstructionSet::dispatch) method.
//!
//! ## Instruction Set Dispatch
//!
//! The [`InstructionSet`](crate::instruction::InstructionSet) derive macro defines instruction discriminants for each instruction variant
//! (by default it is compatible with Anchor's sighashes, but you can override it), and generates dispatch logic:
//!
//! ```
//! # fn main() {}
//! use star_frame::prelude::*;
//!
//! #[derive(InstructionSet)]
//! # #[ix_set(skip_idl)]
//! pub enum CounterInstructionSet {
//!     Initialize(Initialize),
//!     Increment(Increment),
//! }
//!
//! # #[derive(Debug)]
//! # pub struct Initialize;
//! # #[derive(Debug)]
//! # pub struct Increment;
//! # star_frame::impl_blank_ix!(Initialize, Increment);
//! ```
//!
//! The derived [`InstructionSet::dispatch`](crate::instruction::InstructionSet::dispatch) method matches on the instruction discriminant
//!  from the instruction data, and calls [`Instruction::process_from_raw`](crate::instruction::Instruction::process_from_raw) for the matched instruction.
//!
//! ## Instruction Processing
//!
//! The [`Instruction`](crate::instruction::Instruction) trait provides the low-level interface for instruction processing,
//! but it's rough and requires manual handling of raw account data and instruction bytes. In most cases, you should
//! implement the [`StarFrameInstruction`](crate::instruction::StarFrameInstruction) and [`InstructionArgs`](crate::instruction::InstructionArgs)
//! traits instead. The `Instruction` trait is implemented generically for all instructions that implement `StarFrameInstruction`.
//!
//! Check the docs on [`StarFrameInstruction`](crate::instruction::StarFrameInstruction) for how that implementation works.
//!
//! ## Instruction Data Parsing
//!
//! Instructions implement [`BorshDeserialize`](borsh::BorshDeserialize) (to parse the instruction data), and [`InstructionArgs`](crate::instruction::InstructionArgs)
//! (to split the data into `AccountSet` lifecycle arguments). See the [`InstructionArgs`](crate::instruction::InstructionArgs) trait for more information.
//!
//! ```
//! use star_frame::prelude::*;
//! # fn main() {}
//! #[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
//! # #[instruction_args(skip_idl)]
//! pub struct Initialize {
//!     #[ix_args(&run)]
//!     pub start_at: Option<u64>,
//! }
//! ```
//!
//! ## Defining Program Accounts
//!
//! Star Frame provides multiple ways to define program accounts for different use cases:
//!
//! ### Basic Account with Standard Derive
//!
//! For statically sized accounts, you can use the [`ProgramAccount`](derive@crate::account_set::ProgramAccount) derive.
//! For the best performance, you can use [`bytemuck`] (with the [`zero_copy`] macro for convenience) with [`Account`](crate::account_set::account::Account):
//!
//! ```
//! # fn main() {}
//! # #[derive(StarFrameProgram)]
//! # #[program(instruction_set = (), id = System::ID, no_entrypoint)]
//! # pub struct MyProgram;
//! #
//! use star_frame::prelude::*;
//!
//! #[zero_copy(pod)]
//! #[derive(Default, Debug, Eq, PartialEq, ProgramAccount)]
//! #[program_account(seeds = CounterSeeds)]
//! pub struct CounterAccount {
//!     pub authority: Pubkey,
//!     pub count: u64,
//! }
//!
//! // Strongly typed seeds can be defined too!
//! #[derive(Debug, GetSeeds, Clone)]
//! #[get_seeds(seed_const = b"COUNTER")]
//! pub struct CounterSeeds {
//!     pub authority: Pubkey,
//! }
//! ```
//!
//! You can also use [`borsh`] with [`BorshAccount`](crate::account_set::borsh_account::BorshAccount) if you don't ~~like~~ need performance.
//!
//! ### Unsized Accounts with the Unsized Type system
//!
//! For accounts with variable-size data like vectors or dynamic strings, use [`unsized_type`](crate::unsize::unsized_type).
//!
//! ```
//! # fn main() {}
//! use star_frame::prelude::*;
//! #
//! # #[derive(StarFrameProgram)]
//! # #[program(instruction_set = (), id = System::ID, no_entrypoint)]
//! # pub struct MyProgram;
//! #
//! #[derive(Debug, GetSeeds, Clone)]
//! # pub struct CounterSeeds;
//!
//! #[unsized_type(program_account, seeds = CounterSeeds)]
//! pub struct CounterAccount {
//!     pub authority: Pubkey,
//!     #[unsized_start]
//!     pub count_tracker: UnsizedMap<Pubkey, PackedValue<u64>>,
//! }
//! ```
//! Check out the [`unsize`] module for more details.
//!
//! ## Account Set Validation
//!
//! Accounts are validated through [`AccountSet`](crate::account_set::AccountSet) traits with compile-time and runtime checks:
//!
//! ```
//! # fn main() {}
//! use star_frame::prelude::*;
//! #[derive(AccountSet)]
//! # #[account_set(skip_default_idl)]
//! pub struct InitializeAccounts {
//!     #[validate(funder)]
//!     pub authority: Signer<Mut<SystemAccount>>,
//!     #[validate(arg = (
//!         Create(()),
//!         Seeds(CounterSeeds { authority: *self.authority.pubkey() }),
//!     ))]
//!     pub counter: Init<Seeded<Account<CounterAccount>>>,
//!     pub system_program: Program<System>,
//! }
//! #
//! # #[derive(StarFrameProgram)]
//! # #[program(instruction_set = (), id = System::ID, no_entrypoint)]
//! # pub struct MyProgram;
//! #
//! #[derive(Align1, Pod, Zeroable, Default, Copy, Clone, Debug, Eq, PartialEq, ProgramAccount)]
//! # #[program_account(seeds = CounterSeeds)]
//! # #[repr(C, packed)]
//! # pub struct CounterAccount {
//! #   authority: Pubkey,
//! # }
//! #
//! # #[derive(Debug, GetSeeds, Clone)]
//! # #[get_seeds(seed_const = b"COUNTER")]
//! # pub struct CounterSeeds {
//! #     pub authority: Pubkey,
//! # }
//!
//! ```
//!
//! ## Instruction Processing
//!
//! Finally, [`StarFrameInstruction::process`](crate::instruction::StarFrameInstruction::process) executes the instruction logic:
//!
//! ```
//! use star_frame::prelude::*;
//! # fn main() {}
//! # #[derive(StarFrameProgram)]
//! # #[program(instruction_set = (), id = System::ID, no_entrypoint)]
//! # pub struct MyProgram;
//! #
//! # #[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
//! # #[instruction_args(skip_idl)]
//! # pub struct Initialize {
//! #    #[ix_args(&run)]
//! #    pub start_at: Option<u64>,
//! # }
//! #
//! # #[derive(Align1, Pod, Zeroable, Default, Copy, Clone, Debug, Eq, PartialEq, ProgramAccount)]
//! # #[repr(C, packed)]
//! # pub struct CounterAccount {
//! #     pub authority: Pubkey,
//! #     pub count: u64,
//! # }
//! # #[derive(AccountSet, Debug)]
//! # #[account_set(skip_default_idl)]
//! # pub struct InitializeAccounts {
//! #     pub counter: Mut<Account<CounterAccount>>,
//! #     pub authority: AccountInfo,
//! # }
//! use star_frame::prelude::*;
//!
//! impl StarFrameInstruction for Initialize {
//!     type Accounts<'decode, 'arg> = InitializeAccounts;
//!     type ReturnType = ();
//!
//!     fn process(
//!         accounts: &mut Self::Accounts<'_, '_>,
//!         start_at: &Option<u64>,
//!         _ctx: &mut Context,
//!     ) -> Result<()> {
//!         **accounts.counter.data_mut()? = CounterAccount {
//!             authority: *accounts.authority.pubkey(),
//!             count: start_at.unwrap_or(0),
//!         };
//!         Ok(())
//!     }
//! }
//! ```
//!
//! You can directly implement [`StarFrameInstruction`](crate::instruction::StarFrameInstruction) with the process function using the
//! [`star_frame_instruction`](crate::instruction::star_frame_instruction) macro.
//! ```
//! # fn main() {}
//! # #[derive(StarFrameProgram)]
//! # #[program(instruction_set = (), id = System::ID, no_entrypoint)]
//! # pub struct MyProgram;
//! #
//! # #[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
//! # #[instruction_args(skip_idl)]
//! # pub struct Initialize {
//! #    #[ix_args(&run)]
//! #    pub start_at: Option<u64>,
//! # }
//! #
//! # #[derive(Align1, Pod, Zeroable, Default, Copy, Clone, Debug, Eq, PartialEq, ProgramAccount)]
//! # #[repr(C, packed)]
//! # pub struct CounterAccount {
//! #     pub authority: Pubkey,
//! #     pub count: u64,
//! # }
//! # #[derive(AccountSet, Debug)]
//! # #[account_set(skip_default_idl)]
//! # pub struct InitializeAccounts {
//! #     pub counter: Mut<Account<CounterAccount>>,
//! #     pub authority: AccountInfo,
//! # }
//! use star_frame::prelude::*;
//!
//! #[star_frame_instruction]
//! fn Initialize(accounts: &mut InitializeAccounts, start_at: &Option<u64>) -> Result<()> {
//!     **accounts.counter.data_mut()? = CounterAccount {
//!         authority: *accounts.authority.pubkey(),
//!         count: start_at.unwrap_or(0),
//!     };
//!     Ok(())
//! }
//! ```
//!
//! ## Putting it all together
//!
//! You can check out our [example programs](https://github.com/staratlasmeta/star_frame/tree/main/example_programs) for more information,
//! and the [simple counter example](https://github.com/staratlasmeta/star_frame/blob/main/example_programs/simple_counter/src/lib.rs) for how these steps are
//! put together.
//!
//! # Generating IDLs
//!
//! Star Frame can automatically generate Codama IDL files for your programs when the `idl` feature flag is enabled.
//! IDLs are JSON files that describe your program's interface, making it easier for clients to interact with your program.
//! Check out the [Codama](https://github.com/codama-idl/codama) for more information on generating clients and using the IDL.
//!
//! ## Enabling IDL Generation
//!
//! Add the `idl` feature to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! star_frame = { version = "*", features = ["idl"] }
//! ```
//!
//! ## Generating an IDL
//!
//! Programs that derive [`StarFrameProgram`](crate::program::StarFrameProgram) automatically implement
//! [`ProgramToIdl`](crate::idl::ProgramToIdl). You can create a test to generate the IDL:
//!
//! ```ignore
//! #[cfg(feature = "idl")]
//! #[test]
//! fn generate_idl() -> Result<()> {
//!     use star_frame::prelude::*;
//!     let idl = CounterProgram::program_to_idl()?;
//!     let codama_idl: ProgramNode = idl.try_into()?;
//!     let idl_json = codama_idl.to_json()?;
//!     std::fs::write("idl.json", &idl_json)?;
//!     Ok(())
//! }
//! ```
//! And run it with:
//! ```shell
//! cargo test --features idl -- generate_idl
//! ```
//!
//! # Feature Flags
//!
//! Star Frame provides several feature flags to customize functionality:
//! - `idl` - Enables IDL generation for client libraries
//! - `test_helpers` - Provides utilities for testing programs and the unsized type system
//! - `cleanup_rent_warning` - Emits a warning message if the account has more lamports than required by rent on cleanup
//! - `aggressive_inline` - Adds `#[inline(always)]` to more functions. Can be beneficial in some cases, but will likely increase binary size and may even reduce performance.
//!   This should only be used when you have thorough benchmarks and are confident in the performance impact.
#![warn(
    clippy::pedantic,
    missing_copy_implementations,
    missing_debug_implementations,
    unsafe_op_in_unsafe_fn
)]
#![allow(
    unexpected_cfgs,
    clippy::non_canonical_clone_impl,
    clippy::default_trait_access,
    clippy::manual_string_new,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::mut_mut,
    clippy::wildcard_imports,
    clippy::expl_impl_clone_on_copy,
    clippy::non_canonical_partial_ord_impl
)]

pub extern crate advancer;
pub extern crate borsh;
pub extern crate bytemuck;
pub extern crate derive_more;
pub extern crate derive_where;
pub extern crate eyre;
pub extern crate fixed;
pub extern crate itertools;
pub extern crate num_traits;
pub extern crate paste;
pub extern crate pinocchio;
pub extern crate self as star_frame;
pub extern crate serde;
#[cfg(all(feature = "idl", not(target_os = "solana")))]
pub extern crate serde_json;
pub extern crate solana_instruction;
pub extern crate solana_pubkey;
#[cfg(all(feature = "idl", not(target_os = "solana")))]
pub extern crate star_frame_idl;
pub extern crate static_assertions;
pub extern crate typenum;

pub mod account_set;
pub mod align1;
pub mod client;
pub mod cpi;
pub mod data_types;
mod entrypoint;
mod errors;

pub mod context;
#[cfg(all(feature = "idl", not(target_os = "solana")))]
pub mod idl;
pub mod instruction;
pub mod prelude;
pub mod program;
pub mod unsize;
pub mod util;

/// Internal paths mainly for use in macros. DO NOT USE MANUALLY. NOT PART OF THE PUBLIC API.
#[doc(hidden)]
pub mod __private;

pub use eyre::Result;
#[doc(hidden)]
pub use solana_instruction::Instruction as SolanaInstruction;
pub use star_frame_proc::{pubkey, sighash, zero_copy};

#[allow(unused_imports)]
#[cfg(test)]
use tests::StarFrameDeclaredProgram;

#[cfg(all(not(feature = "test_helpers"), any(doctest, test)))]
compile_error!("You must enable the `test_helpers` feature for running tests!");

#[cfg(all(test, feature = "test_helpers"))]
mod tests {
    use super::*;
    use crate::program::StarFrameProgram;
    use solana_pubkey::Pubkey;

    #[derive(StarFrameProgram)]
    #[program(
        instruction_set = (),
        id = Pubkey::new_from_array([0; 32]),
        no_entrypoint,
    )]
    pub struct MyProgram;

    #[cfg(all(feature = "idl", not(target_os = "solana")))]
    #[test]
    fn test_idl() {
        use crate::idl::ProgramToIdl;
        let idl = MyProgram::program_to_idl().unwrap();
        println!("{}", serde_json::to_string_pretty(&idl).unwrap());
    }
}
