#![allow(clippy::let_and_return)]
mod account_set;
mod align1;
mod get_seeds;
mod hash;
mod idl;
mod instruction_set;
mod program;
mod program_account;
mod solana_pubkey;
mod unsize;
mod util;

use proc_macro_error::proc_macro_error;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{parse_macro_input, DeriveInput, Item, ItemEnum, LitStr};

#[proc_macro_error]
#[proc_macro_derive(
    AccountSet,
    attributes(account_set, decode, validate, cleanup, idl, single_account_set)
)]
pub fn derive_account_set(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let out = account_set::derive_account_set_impl(parse_macro_input!(input as DeriveInput));
    out.into()
}

/// Derives the `GetSeeds` trait for a struct.
///
/// # Attributes
///
/// ## 1. `#[get_seeds(seed_const = <expr>, skip_idl)]` (item level attribute)
///
/// ### syntax
///
/// Attribute takes an `Expr` which resolves to a `&[u8]` seed for the account.
/// If `skip_idl` is present, the `SeedsToIdl` trait and the `IdlFindSeed` struct will not be derived.
///
/// ### usage
///
/// Attribute is optional. If the attribute is present, the seed for the account will be the concatenation
/// of the seed provided in the attribute and the seeds of the fields of the account.
///
/// ```
/// # use star_frame::prelude::*;
/// // `seed_const` is not present
/// // Resulting `account.seeds()` is `vec![account.key.seed(), account.number.seed()];`
///
/// #[derive(Debug, GetSeeds, Clone)]
/// pub struct TestAccount {
///     key: Pubkey,
///     number: u64,
/// }
///
/// let account = TestAccount {
///     key: Pubkey::new_unique(),
///     number: 42,
/// };
/// ```
///
/// ```
/// # use star_frame::prelude::*;
/// // `seed_const` here resolves to the `DISC` constant of the `Cool` struct
/// // Resulting `account.seeds()` is `vec![b"TEST_CONST".as_ref()];`
/// pub struct Cool {}
///
/// impl Cool {
///     const DISC: &'static [u8] = b"TEST_CONST";
/// }
///
/// #[derive(Debug, GetSeeds, Clone)]
/// #[get_seeds(seed_const = Cool::DISC)]
/// pub struct TestAccount {}
/// ```
///
/// ```
/// # use star_frame::prelude::*;
/// // `seed_const` here resolves to the byte string `b"TEST_CONST"`
/// // Resulting `account.seeds()` is `vec![b"TEST_CONST".as_ref(), account.key.seed()];`
/// #[derive(Debug, GetSeeds, Clone)]
/// #[get_seeds(seed_const = b"TEST_CONST")]
/// pub struct TestAccount {
///     key: Pubkey,
/// }
/// ```
#[proc_macro_error]
#[proc_macro_derive(GetSeeds, attributes(get_seeds))]
pub fn derive_get_seeds(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let out = get_seeds::derive_get_seeds_impl(parse_macro_input!(input as DeriveInput));
    out.into()
}

/// Derives `Align1` for a valid type.
#[proc_macro_error]
#[proc_macro_derive(Align1)]
pub fn derive_align1(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    align1::derive_align1_impl(parse_macro_input!(item as DeriveInput)).into()
}

/// Derives the `InstructionSet` trait for an enum of instructions.
///
/// It uses a discriminant type of `[u8; 8]`, and derives each item discriminant by taking
/// the first 8 bytes of the sha256 hash in a compatible way with Anchor.
///
/// # Example
///
/// ```
/// use star_frame::impl_blank_ix;
/// use star_frame::prelude::*;
///
/// #[derive(InstructionSet)]
/// #[ix_set(skip_idl)]
/// pub enum CoolIxSet {
///     CoolInstruction(CoolIx),
/// }
///
/// // hash from anchor
/// const IX_DISCRIMINANT: [u8; 8] = [197, 46, 153, 154, 189, 74, 154, 10];
///
/// assert_eq!(CoolIx::DISCRIMINANT, IX_DISCRIMINANT);
///
///
/// // An example instruction (which implements `StarFrameInstruction`)
/// pub struct CoolIx;
/// # impl_blank_ix!(CoolIx);
/// ```
// todo: add this back once custom reprs are supported
// todo: add docs for idl stuff
// Using enum reprs as discriminants:
// ```
// use star_frame::impl_blank_ix;
// use star_frame::prelude::*;
//
// // Example Instructions (which implement `StarFrameInstruction`)
// pub struct CoolIx1 {}
// pub struct CoolIx3 {}
// pub struct CoolIx2 {}
//
// #[star_frame_instruction_set(u8)]
// pub enum CoolIxSetU8 {
//     CoolInstruction1(CoolIx1),
//     CoolInstruction2(CoolIx2),
//     CoolInstruction3(CoolIx3) = 100,
// }
// assert_eq!(<CoolIx1 as InstructionDiscriminant<CoolIxSetU8>>::DISCRIMINANT, 0u8);
// assert_eq!(<CoolIx2 as InstructionDiscriminant<CoolIxSetU8>>::DISCRIMINANT, 1u8);
// assert_eq!(<CoolIx3 as InstructionDiscriminant<CoolIxSetU8>>::DISCRIMINANT, 100u8);
//
// // The same instructions can be used in multiple instruction sets, since the
// // `InstructionDiscriminant` trait is generic over the instruction set.
// #[star_frame_instruction_set(i32)]
// pub enum CoolIxSetU32 {
//     CoolInstruction1(CoolIx1) = -999,
//     CoolInstruction2(CoolIx2),
//     CoolInstruction3(CoolIx3) = 9999,
// }
// assert_eq!(<CoolIx1 as InstructionDiscriminant<CoolIxSetU32>>::DISCRIMINANT, -999i32);
// assert_eq!(<CoolIx2 as InstructionDiscriminant<CoolIxSetU32>>::DISCRIMINANT, -998i32);
// assert_eq!(<CoolIx3 as InstructionDiscriminant<CoolIxSetU32>>::DISCRIMINANT, 9999i32);
//
// # impl_blank_ix!(CoolIx1, CoolIx2, CoolIx3);
// ```
#[proc_macro_error]
#[proc_macro_derive(InstructionSet, attributes(ix_set))]
pub fn star_frame_instruction_set(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let out = instruction_set::instruction_set_impl(parse_macro_input!(item as ItemEnum));
    out.into()
}

// todo: docs
#[proc_macro_error]
#[proc_macro_derive(ProgramAccount, attributes(program_account, type_to_idl))]
pub fn program_account(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let out = program_account::program_account_impl(parse_macro_input!(input as DeriveInput));
    out.into()
}

/// Derives `StarFrameProgram` and sets up the entrypoint and useful items for a program. This should be placed at the root of the crate.
///
/// ## Additional code generated:
/// - Solana entrypoint - This will call the `star_frame_entrypoint` macro with the program struct.
/// - `StarFrameDeclaredProgram` - This is a type alias around the struct that is used in other `star_frame` macros. This
/// derive should be placed at the root of the crate, or be re-exported there.
/// - `declare_id!` - It also generates the `crate::ID` and `id()` constants like how the `solana_program::declare_id` macro works.
///
/// Both the `ID`s and `StarFrameDeclaredProgram` items are generated with the `star_frame::program_setup` macro.
///
/// # Example
/// ```
/// # fn main() {}
/// use star_frame::prelude::*;
///
/// type MyInstructionSet<'a> = ();
///
/// #[derive(StarFrameProgram)]
/// #[program(
///     instruction_set = MyInstructionSet<'static>,
///     id = Pubkey::new_from_array([0; 32]),
///     account_discriminant = [u8; 8],
///     no_entrypoint,
///     no_setup,
///     skip_idl
/// )]
/// struct MyProgram;
/// ```
/// The arguments can be split up into multiple attributes for conditional compilation:
/// ```
/// # fn main() {}
/// use star_frame::prelude::*;
///
/// #[derive(StarFrameProgram)]
/// #[program(instruction_set = ())]
/// #[cfg_attr(feature = "prod", program(id = "11111111111111111111111111111111"))]
/// #[cfg_attr(not(feature = "prod"), program(id = SystemProgram::PROGRAM_ID))]
/// struct MyOtherProgram;
/// ```
///
/// # Arguments
/// ```ignore
/// #[program(
///     instruction_set = <ty>,
///     id = <expr>,
///     account_discriminant = <ty>,
///     closed_account_discriminant = <expr>,
///     no_entrypoint,
///     no_setup,
///     skip_idl
/// )]
/// ```
/// - `instruction_set` - The enum that implements `InstructionSet` for the program. If the instruction set has a
/// lifetime, it should be passed in as `'static`.
/// - `id` - The program id for the program. This can be either a literal string in base58 ("AABBCC42")
/// or an expression that resolves to a `Pubkey`
/// - `account_discriminant` - The `AccountDiscriminant` type used for the program. Defaults to `[u8; 8]` (similarly to Anchor)
/// - `closed_account_discriminant` - The `AccountDiscriminant` value used for closed accounts. Defaults to `[u8::MAX; 8]`
/// - `no_entrypoint` - If present, the macro will not generate an entrypoint for the program.
/// While the generated entrypoint is already feature gated, this may be useful in some cases where features aren't convenient.
/// - `no_setup` - If present, the macro will not call the `program_setup!` macro. This is useful in libraries that may contain multiple programs.
/// - `skip_idl` - If present, the macro will not generate a `ProgramToIdl` implementation for the program.
#[proc_macro_error]
#[proc_macro_derive(StarFrameProgram, attributes(program))]
pub fn program(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let out = program::program_impl(parse_macro_input!(input as DeriveInput));
    out.into()
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn unsized_type(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let out = unsize::unsized_type_impl(parse_macro_input!(item as Item), args.into());
    out.into()
}

/// Derives `TypeToIdl` for a valid type.
// todo: docs
#[proc_macro_error]
#[proc_macro_derive(TypeToIdl, attributes(type_to_idl))]
pub fn derive_type_to_idl(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let out = idl::derive_type_to_idl(parse_macro_input!(item as DeriveInput));
    out.into()
}

// todo: docs
#[proc_macro_error]
#[proc_macro_derive(InstructionToIdl, attributes(instruction_to_idl, type_to_idl))]
pub fn derive_instruction_to_idl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let out = idl::derive_instruction_to_idl(parse_macro_input!(input as DeriveInput));
    out.into()
}

/// Takes in multiple string literals and returns the first 8 bytes of its sha256 hash.
/// The strings will be concatenated with a `:` separator prior to hashing if multiple are passed in.
///
/// # Example
/// ```
/// use star_frame_proc::sighash;
/// // hash of "Hello World!"
/// const HELLO_WORLD: [u8; 8] = [0x7f, 0x83, 0xb1, 0x65, 0x7f, 0xf1, 0xfc, 0x53];
/// assert_eq!(sighash!("Hello World!"), HELLO_WORLD);
///
/// const NAMESPACE_HASH: [u8; 8] = [0x76, 0x03, 0x6f, 0xcc, 0x93, 0xdd, 0x73, 0x10];
/// assert_eq!(sighash!("global", "other_stuff"), NAMESPACE_HASH);
/// ```
#[proc_macro]
pub fn sighash(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    hash::sighash_impl(parse_macro_input!(input with Punctuated::<LitStr, Comma>::parse_terminated))
        .into()
}

// ---- Copied solana-program macros to use `star_frame::solana_program` path  ----
#[proc_macro]
pub fn pubkey(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    solana_pubkey::pubkey_impl(input)
}
