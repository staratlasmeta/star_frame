#![allow(clippy::let_and_return)]
mod account_set;
mod align1;
mod get_seeds;
mod hash;
mod idl;
mod instruction_args;
mod instruction_set;
mod program;
mod program_account;
mod solana_pubkey;
mod star_frame_instruction;
mod unsize;
mod util;
mod zero_copy;

use proc_macro_error2::proc_macro_error;
use syn::{
    parse::Nothing, parse_macro_input, punctuated::Punctuated, token::Comma, DeriveInput, Item,
    ItemEnum, ItemFn, ItemImpl, LitStr,
};

/// Derives `AccountSet` lifecycle traits and `AccountSetToIdl` for a struct.
///
/// The `AccountSet` proc macro generates implementations for the three core traits:
/// - `AccountSetDecode` - Decodes accounts from `&[AccountInfo]` arrays
/// - `AccountSetValidate` - Validates decoded accounts  
/// - `AccountSetCleanup` - Performs cleanup operations after instruction execution
///
/// It also generates client-side implementations:
/// - `CpiAccountSet` - Cross-program invocation account handling
/// - `ClientAccountSet` - Client-side account metadata generation
/// - `AccountSetToIdl
///
/// This macro creates a comprehensive account management system that handles account validation,
/// decoding from account info arrays, cleanup operations, and IDL generation for Solana programs.
///
/// # Integration with StarFrameInstruction
///
/// When using AccountSet with `StarFrameInstruction`, the argument types specified in field-level
/// attributes must correspond to the argument types from `InstructionArgs`. The `arg` parameter
/// in field attributes should match the types available from the instruction's decode, validate,
/// run, and cleanup argument types.
///
/// # Struct-level Attributes
///
/// ## `#[account_set(skip_client_account_set, skip_cpi_account_set, skip_default_decode, skip_default_validate, skip_default_cleanup, skip_default_idl)]`
///
/// Controls which implementations are generated:
/// - `skip_client_account_set` - Skips generating `ClientAccountSet` implementation
/// - `skip_cpi_account_set` - Skips generating `CpiAccountSet` implementation  
/// - `skip_default_decode` - Skips generating default `AccountSetDecode` implementation
/// - `skip_default_validate` - Skips generating default `AccountSetValidate` implementation
/// - `skip_default_cleanup` - Skips generating default `AccountSetCleanup` implementation
/// - `skip_default_idl` - Skips generating default IDL implementations
///
/// ## `#[decode(id = <str>, arg = <type>, generics = <generics>, inline_always)]`
///
/// Define custom decode implementations with specific arguments:
/// - `id = <str>` - Unique identifier for this decode variant (optional, defaults to no id)
/// - `arg = <type>` - Type of argument passed to decode functions
/// - `generics = <generics>` - Additional generic parameters for this decode implementation
/// - `inline_always` - Whether to add `#[inline(always)]` to the decode implementation (by default `#[inline]` is added)
///
/// ## `#[validate(id = <str>, arg = <type>, generics = <generics>, before_validation = <expr>, extra_validation = <expr>, inline_always)]`
///
/// Define custom validation implementations:
/// - `id = <str>` - Unique identifier for this validate variant (optional, defaults to no id)
/// - `arg = <type>` - Type of argument passed to validate functions
/// - `generics = <generics>` - Additional generic parameters for this validate implementation
/// - `before_validation = <expr>` - Expression to execute before field validation
/// - `extra_validation = <expr>` - Expression to execute after field validation
/// - `inline_always` - Whether to add `#[inline(always)]` to the validate implementation (by default `#[inline]` is added)
///
/// ## `#[cleanup(id = <str>, generics = <generics>, arg = <type>, extra_cleanup = <expr>, inline_always)]`
///
/// Define custom cleanup implementations:
/// - `id = <str>` - Unique identifier for this cleanup variant
/// - `generics = <generics>` - Generic parameters for this cleanup implementation
/// - `arg = <type>` - Type of argument passed to cleanup functions
/// - `extra_cleanup = <expr>` - Cleanup expression to execute after field cleanup
/// - `inline_always` - Whether to add `#[inline(always)]` to the cleanup implementation (by default `#[inline]` is added)
///
/// ## `#[idl(id = <str>, arg = <type>, generics = <generics>)]`
///
/// Define custom IDL generation implementations:
/// - `id = <str>` - Unique identifier for this IDL variant (optional, defaults to no id)
/// - `arg = <type>` - Type of argument passed to IDL functions
/// - `generics = <generics>` - Additional generic parameters for this IDL implementation
///
/// # Field-level Attributes
///
/// ## `#[account_set(skip = <TokenStream>)]`
///
/// Skip this field during account set processing. The field will be initialized with the provided default value.
///
/// ## `#[single_account_set(signer, writable, meta = <expr>, skip_*)]`
///
/// Mark a field as a single account set. This indicates that the AccountSet contains only one account
/// and all account set traits should be passed through to this flagged field. Only one field can have this attribute.
///
/// Options:
/// - `signer` - Mark this account as a signer
/// - `writable` - Mark this account as writable  
/// - `meta = <expr>` - Custom metadata expression
/// - `skip_signed_account` - Skip `SignedAccount` trait implementation
/// - `skip_writable_account` - Skip `WritableAccount` trait implementation
/// - `skip_has_inner_type` - Skip `HasInnerType` trait implementation
/// - `skip_has_owner_program` - Skip `HasOwnerProgram` trait implementation
/// - `skip_has_seeds` - Skip `HasSeeds` trait implementation
/// - `skip_can_init_seeds` - Skip `CanInitSeeds` trait implementation
/// - `skip_can_init_account` - Skip `CanInitAccount` trait implementation
///
/// When a field is marked with `#[single_account_set]`, the generated AccountSet implementation will:
/// - Implement `SingleAccountSet` and delegate to the marked field
/// - Pass through `CpiAccountSet` and `ClientAccountSet` implementations
/// - Forward trait implementations like `SignedAccount`, `WritableAccount`, `HasSeeds`, etc.
///
/// ## `#[validate(id = <str>, funder, recipient, skip, requires = [<field>, ...], arg = <expr>, temp = <expr>, arg_ty = <type>, address = <expr>)]`
///
/// Pass arguments to field validation:
/// - `id = <str>` - Which validate variant this field participates in, to enable multiple `AccountSetValidate` implementations
/// - `funder` - Mark this field as the funder for the Context cache (only one field can be marked as funder)
/// - `recipient` - Mark this field as the recipient for the Context cache (only one field can be marked as recipient)
/// - `skip` - Skip validation for this field
/// - `requires = [<field>, ...]` - List of fields that must be validated before this field
/// - `arg = <expr>` - Argument to pass to the field's `AccountSetValidate`` function
/// - `temp = <expr>` - Temporary variable expression to use with `arg` (requires `arg` to be specified)
/// - `arg_ty = <type>` - Type of the validation argument. Usually inferred, but can be specified to get better error messages
/// - `address = <expr>` - Check that the field's key matches this address, expr must return a `&Pubkey`
///
/// ## `#[decode(id = <str>, arg = <expr>)]`
///
/// Pass arguments to field decoding:
/// - `id = <str>` - Which decode variant this field participates in, to enable multiple `AccountSetDecode` implementations
/// - `arg = <expr>` - Argument to pass to the field's `AccountSetDecode` function
///
/// ## `#[cleanup(id = <str>, arg = <expr>)]`
///
/// Pass arguments to field cleanup:
/// - `id = <str>` - Which cleanup variant this field participates in, to enable multiple `AccountSetCleanup` implementations
/// - `arg = <expr>` - Argument to pass to the field's `AccountSetCleanup` function
///
/// ## `#[idl(id = <str>, arg = <expr>, address = <expr>)]`
///
/// Pass arguments to IDL generation:
/// - `id = <str>` - Which IDL variant this field participates in, to enable multiple `AccountSetToIdl` implementations
/// - `arg = <expr>` - Argument to pass to the field's `AccountSetToIdl` function for IDL generation
/// - `address = <expr>` - Address expression for single account IDL generation, expr must return a `Pubkey`
///
/// # Examples
///
/// ## Basic Account Set
///
/// ```
/// # fn main() {}
/// use star_frame::prelude::*;
///
/// #[derive(AccountSet)]
/// pub struct BasicAccounts {
///     pub authority: Signer,
///     pub account: Mut<SystemAccount>,
///     pub system_program: Program<System>,
/// }
/// ```
///
/// ## Account Set with Custom Arguments
///
/// ```
/// # fn main() {}
/// use star_frame::prelude::*;
///
/// #[derive(AccountSet)]
/// #[decode(arg = usize)]
/// #[validate(arg = String, extra_validation = self.check_authority(arg))]
/// pub struct CustomAccounts {
///     pub authority: Signer,
///     // One of Vec's AccountSetDecode implementations takes in a usize to specify the number of accounts to decode, so it will try to decode `arg * 2` (for some reason?) accounts
///     // which will be passed to the `AccountSetDecode` function from StarFrameInstruction as the decode arg
///     #[decode(arg = arg * 2)]
///     pub accounts: Vec<SystemAccount>,
/// }
///
/// impl CustomAccounts {
///     fn check_authority(&self, arg: String) -> Result<()> {
///         todo!("check stuff")
///     }
/// }
/// ```
///
/// By setting the decode arg to usize, and validate to String, any `StarFrameInstruction` using this set must have an `InstructionArgs` implementation that returns those types.
///
/// ## Single Account Set Newtype
///
/// ```
/// # fn main() {}
/// use star_frame::{prelude::*, derive_more};
/// # #[derive(StarFrameProgram)]
/// # #[program(instruction_set = (), id = System::ID, no_entrypoint)]
/// # pub struct MyProgram;
/// # #[zero_copy(pod)]
/// # #[derive(ProgramAccount, Debug, Default)]
/// # pub struct CounterAccount { pub count: u64 }
///
/// #[derive(AccountSet, derive_more::Deref, derive_more::DerefMut, Debug)]
/// pub struct WrappedCounter(#[single_account_set] Account<CounterAccount>);
/// ```
///
/// This creates a newtype wrapper that implements `AccountSet` and passes through all account
/// traits to the inner `Account<CounterAccount>`. The `signer` and `writable` flags modify
/// the account's metadata for CPI and client usage. This will propagate all of the `account_set::modifier`
/// marker traits from the inner account to the newtype.
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
/// ### Syntax
///
/// Attribute takes an `Expr` which resolves to a `&[u8]` seed for the account.
/// If `skip_idl` is present, the `SeedsToIdl` trait and the `IdlFindSeed` struct will not be derived.
///
/// ### Usage
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

/// Derives `ProgramAccount` for a struct.
///
/// This macro generates implementations for account-related traits and optionally `AccountToIdl` and `TypeToIdl`.
///
/// # Attributes
///
/// ## `#[program_account(skip_idl, program = <ty>, seeds = <ty>, discriminant = <expr>)]` (item level attribute)
///
/// ### Arguments
/// - `skip_idl` (presence) - If present, skips generating IDL implementations for this account
/// - `program` (optional `Type`) - Specifies the program that owns this account type. Defaults to StarFrameDeclaredProgram at root of your crate
///    (Defined by the `#[derive(StarFrameProgram)]` macro)
/// - `seeds` (optional `Type`) - Specifies the seed type used to generate PDAs for this account
/// - `discriminant` (optional `Expr`) - Custom discriminant value for the account type, overriding the Anchor style sighash
///
/// ### Usage
/// ```
/// # fn main() {}
/// use star_frame::prelude::*;
///
/// # #[derive(StarFrameProgram)]
/// # #[program(instruction_set = (), id = System::ID, no_entrypoint)]
/// # pub struct MyProgram;
///
/// #[zero_copy(pod)]
/// #[derive(ProgramAccount, Debug)]
/// #[program_account(seeds = MyAccountSeeds)]
/// pub struct MyAccount {
///     pub data: u64,
/// }
///
/// #[derive(GetSeeds, Debug, Clone)]
/// pub struct MyAccountSeeds {
///     pub key: Pubkey,
/// }
/// ```
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
/// - `declare_id!` - It also generates the `crate::ID` and `id()` constants like how the `pinocchio::declare_id` macro works.
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
/// #[cfg_attr(not(feature = "prod"), program(id = System::ID))]
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

/// Generates unsized type wrappers for dynamic-length structs and enums in a way that emulates normal rust syntax.
///
/// This attribute macro creates wrapper structs and their `UnsizedType` implementations for handling variable-length data
/// on-chain, such as dynamic lists or maps.
///
/// # Arguments
/// ```ignore
/// #[unsized_type(
///     owned_attributes = [derive(...)],
///     owned_type = <ty>,
///     owned_from_ref = <path>,
///     sized_attributes = [derive(...)],
///     program_account,
///     skip_idl,
///     skip_phantom_generics,
///     skip_init_struct,
///     program = <ty>,
///     seeds = <ty>,
///     discriminant = <expr>
/// )]
/// ```
/// - `owned_attributes` - Additional attributes to apply to the `UnsizedType::Owned` variant
/// - `sized_attributes` - Additional attributes to apply to the generated Sized portion of the struct
/// - `owned_type` - Override the type for the `UnsizedType::Owned` variant
/// - `owned_from_ref` - Override the function to convert from reference to `UnsizedType::Owned`
/// - `program_account` - Mark as a program account, deriving the `ProgramAccount` and `AccountToIdl` traits
/// - `skip_idl` - Skips `TypeToIdl`/`AccountToIdl` generation
/// - `skip_phantom_generics` - Skip phantom generic parameters in the generated Sized struct
/// - `skip_init_struct` - Skip generating initialization struct for `UnsizedInit<MyStructInit>`
/// - `program` - Override the program that owns this account type
/// - `seeds` - Seed type for HasSeeds. Requires `program_account` to be present.
/// - `discriminant` - Custom discriminant value, overrides the Anchor style sighash
///
/// # Example Struct
///
/// ```
/// use star_frame::prelude::*;
///
/// # #[derive(StarFrameProgram)]
/// # #[program(instruction_set = (), id = System::ID, no_entrypoint)]
/// # pub struct MyProgram;
///
/// #[unsized_type(program_account)]
/// pub struct MyAccount {
///     pub sized_field: u64,
///     pub another_sized_field: bool,
///     #[unsized_start]
///     pub bytes: List<u8>,
///     pub map: Map<Pubkey, [u8; 10]>,
/// }
///
/// # fn main() -> Result<()> {
/// let account = TestByteSet::<MyAccount>::new_default()?; // Get from program entrypoint or use `TestByteSet`
/// let mut data = account.data_mut()?;
/// data.map().insert(Pubkey::new_unique(), [0; 10]); // To resize a field, access it with the method() version of the field name
/// data.bytes().push(10);
/// data.bytes[0] = 10;
/// let len = data.bytes().len(); // To just read or write to the field without resizing, you can access the field like a normal struct.
/// # Ok(())
/// # }
/// ```
///
/// # Example Enum
///
/// ```
/// use star_frame::prelude::*;
///
/// # #[derive(StarFrameProgram)]
/// # #[program(instruction_set = (), id = System::ID, no_entrypoint)]
/// # pub struct MyProgram;
///
/// #[unsized_type]
/// pub struct MyUnsizedType {
///     pub sized: u64,
///     #[unsized_start]
///     pub map: Map<Pubkey, u8>,
/// }
///
/// #[unsized_type]
/// #[repr(u8)]
/// pub enum MyEnum {
///     #[default_init]
///     UnitVariant,
///     SizedPubkey(Pubkey),
///     Unsized(MyUnsizedType),
/// }
///
/// # fn main() -> Result<()> {
/// let account = TestByteSet::<MyEnum>::new_default()?;
/// let mut data = account.data_mut()?;
/// assert!(matches!(**data, MyEnumMut::UnitVariant));
/// let new_key = Pubkey::new_unique();
/// let mut the_key = data.set_sized_pubkey(new_key)?;
/// assert!(matches!(**the_key, new_key));
///
/// let _unsized_inner = data.set_unsized(DefaultInit)?;
///
/// // You can also call `.get()` to get an exclusive wrapper version of the inner.
/// let MyEnumExclusive::Unsized(mut unsized_inner) = data.get() else {
///     panic!("Expected Unsized variant");
/// };
/// unsized_inner.map().insert(new_key, 10);
///
/// # Ok(())
/// # }
///
/// ```
#[proc_macro_error]
#[proc_macro_attribute]
pub fn unsized_type(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let out = unsize::unsized_type_impl(parse_macro_input!(item as Item), args.into());
    out.into()
}

/// Generates an impl block for an unsized type using standard rust syntax.
///
/// This generates the neccesary boilerplate to work with `UnsizedType`s.
///
/// # Arguments
/// ```ignore
/// #[unsized_impl(tag = <str>, ref_ident = <ident>, mut_ident = <ident>)]
/// ```
/// - `tag` - The tag for the impl block. This is used to avoid trait name collisions when using `#[exclusive]`
/// - `ref_ident` - Overrides the identifier for the `UnsizedType::Ref` type. Defaults to `<SelfTypeName>Ref`
/// - `mut_ident` - Overrides the identifier for the `UnsizedType::Mut` type. Defaults to `<SelfTypeName>Mut`
///
/// # Usage
///
/// All methods must be inherent impls on `&self` or `&mut self`.
///
/// For methods that need to resize the data, take in `&mut self` and add `#[exclusive]` to the method name. This turns &mut self into an `ExclusiveWrapper`
/// around the impl type. This generates a trait that is implemented on the ExclusiveWrapper. For multiple separate impl blocks, add `#[unsized_impl(tag = <str>)]`
/// to avoid trait name collisions. The trait name for public methods is `<SelfTypeName>ExclusiveImpl<optional_tag>` and for private methods is `<SelfTypeName>ExclusiveImplPrivate<optional_tag>`.
///
/// Non-exclusive methods will be called on the `UnsizedType::Ref` or `UnsizedType::Mut` types, with `&self` methods being generated for both.
/// To skip generating a shared impl on Mut, add `#[skip_mut]` to the method.
///
/// # Example
/// ```
/// # fn main() {}
/// use star_frame::prelude::*;
///
/// # #[derive(StarFrameProgram)]
/// # #[program(instruction_set = (), id = System::ID, no_entrypoint)]
/// # pub struct MyProgram;
///
/// #[unsized_type]
/// pub struct MyStruct {
///     pub sized_field: u64,
///     #[unsized_start]
///     pub items: List<u8>,
/// }
///
/// #[unsized_impl]
/// impl MyStruct {
///     // Implemented on both Ref and Mut
///     pub fn len(&self) -> usize {
///         self.items.len()
///     }
///
///     // Implemented on Ref only
///     #[skip_mut]
///     fn len_shared_only(&self) -> usize {
///         self.items.len()
///     }
///
///     // Implemented on Mut only
///     pub fn set_sized(&mut self, item: u64) {
///         self.sized_field = item;
///     }
///
///     #[exclusive]
///     fn push(&mut self, item: u8) -> Result<()> {
///         self.sized_field += item as u64;
///         // This is a method that needs to resize the data, so to access the field we use the method() version of the field name.
///         // This requires being called on an ExclusiveWrapper, which is created by adding #[exclusive] to the method.
///         self.items().push(item)?;
///         Ok(())
///     }
/// }
/// ```
#[proc_macro_error]
#[proc_macro_attribute]
pub fn unsized_impl(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let out = unsize::unsized_impl_impl(parse_macro_input!(item as ItemImpl), args.into());
    out.into()
}

/// Derives `InstructionArgs` on a struct.
///
/// # Attributes
///
/// ## `#[ix_args(decode, validate, run, cleanup)]` (item and field level attribute)
///
/// ### Syntax
///
/// Attribute takes an optional list of the following arguments: `decode`, `validate`, `run`, `cleanup`.
/// Each argument can be optionally preceded by `&` or `&mut` to specify that argument should be borrowed from the struct.
///
/// If an argument type is provided multiple times, the type will be a tuple of the combined types, starting with the top level argument and in order of appearance.
///
/// If an argument type is not provided, the type will default to `()`.
///
/// ## `#[instruction_args(skip_idl)]` (item level attribute)
///
/// If present, the macro will not generate a `InstructionToIdl` implementation for the type.
///
/// # Example
/// ```
/// use star_frame::prelude::*;
/// use star_frame::static_assertions::assert_type_eq_all;
/// #[derive(Copy, Clone, InstructionArgs, Default)]
/// #[instruction_args(skip_idl)]
/// #[ix_args(decode)]
/// pub struct Ix1 {
///     #[ix_args(&mut validate)]
///     pub validate: u64,
///     #[ix_args(run)]
///     pub run: u32,
///     #[ix_args(&cleanup)]
///     pub cleanup: u8,
/// }
///
/// assert_type_eq_all!(
///     <Ix1 as InstructionArgs>::DecodeArg<'static>,
///     Ix1
/// );
/// assert_type_eq_all!(
///     <Ix1 as InstructionArgs>::ValidateArg<'static>,
///     &mut u64
/// );
/// assert_type_eq_all!(
///     <Ix1 as InstructionArgs>::RunArg<'static>,
///     u32
/// );
/// assert_type_eq_all!(
///     <Ix1 as InstructionArgs>::CleanupArg<'static>,
///     &u8
/// );
/// ```
///
/// A single field can be used in multiple args:
/// ```
/// use star_frame::prelude::*;
/// use star_frame::static_assertions::assert_type_eq_all;
/// #[derive(Copy, Clone, Default, InstructionArgs)]
/// #[ix_args(&decode, &validate, cleanup, run)]
/// pub struct Ix2 {
///     pub ignored: u64,
/// }
///
/// assert_type_eq_all!(
///     <Ix2 as InstructionArgs>::DecodeArg<'static>,
///     <Ix2 as InstructionArgs>::ValidateArg<'static>,
///     &Ix2
/// );
/// assert_type_eq_all!(
///     <Ix2 as InstructionArgs>::RunArg<'static>,
///     <Ix2 as InstructionArgs>::CleanupArg<'static>,
///     Ix2
/// );
/// ```
///
/// You can pick multiple fields to turn into a tuple of arguments:
/// ```
/// use star_frame::prelude::*;
/// use star_frame::static_assertions::assert_type_eq_all;
///
/// #[derive(Copy, Clone, Default, InstructionArgs)]
/// #[ix_args(decode)]
/// pub struct Ix3 {
///     #[ix_args(&mut decode)]
///     pub field1: u64,
///     #[ix_args(&decode)]
///     pub field2: u32,
///     #[ix_args(decode)]
///     pub field3: u8,
/// }
///
/// assert_type_eq_all!(
///     <Ix3 as InstructionArgs>::DecodeArg<'static>,
///     (Ix3, &mut u64, &u32, u8)
/// );
/// // None of these args are provided, so the default is `()`
/// assert_type_eq_all!(
///     <Ix3 as InstructionArgs>::ValidateArg<'static>,
///     <Ix3 as InstructionArgs>::RunArg<'static>,
///     <Ix3 as InstructionArgs>::CleanupArg<'static>,
///     ()
/// );
/// ```
#[proc_macro_error]
#[proc_macro_derive(InstructionArgs, attributes(ix_args, type_to_idl, instruction_args))]
pub fn derive_instruction_args(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let out =
        instruction_args::derive_instruction_args_impl(parse_macro_input!(input as DeriveInput));
    out.into()
}

/// Derives `TypeToIdl` for a valid type.
///
/// This macro generates `TypeToIdl` for a type.
///
/// # Attributes
///
/// ## `#[type_to_idl(skip)]` (item level attribute)
///
/// If present, this field and all remaining fields will be skipped in the IDL definition.
///
/// # Example
/// ```
/// # fn main() {}
/// use star_frame::prelude::*;
///
/// # #[derive(StarFrameProgram)]
/// # #[program(instruction_set = (), id = System::ID, no_entrypoint)]
/// # pub struct MyProgram;
///
/// #[derive(TypeToIdl)]
/// pub struct MyData {
///     pub value: u64,
///     pub name: String,
/// }
/// ```
#[proc_macro_error]
#[proc_macro_derive(TypeToIdl, attributes(type_to_idl))]
pub fn derive_type_to_idl(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let out = idl::derive_type_to_idl(&parse_macro_input!(item as DeriveInput));
    out.into()
}

/// Derives `InstructionToIdl` for instruction types.
///
/// This macro generates `InstructionToIdl` for a type. The trait requires `TypeToIdl` to be implemented as well.
///
/// # Example
/// ```
/// use star_frame::prelude::*;
///
/// # #[derive(StarFrameProgram)]
/// # #[program(instruction_set = (), id = System::ID, no_entrypoint)]
/// # pub struct MyProgram;
///
/// #[derive(InstructionToIdl)]
/// #[derive(TypeToIdl)]
/// pub struct MyInstruction {
///     pub amount: u64,
///     pub recipient: Pubkey,
/// }
/// ```
#[proc_macro_error]
#[proc_macro_derive(InstructionToIdl)]
pub fn derive_instruction_to_idl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let out = idl::derive_instruction_to_idl(&parse_macro_input!(input as DeriveInput));
    out.into()
}

/// Shorthand to implement the `StarFrameInstruction` trait through a `StarFrameInstruction::process` function declaration.
///
/// # Function Signature
/// The function ident should be the same as the instruction ident being implemented on (PascalCase and all).
///
/// ## Arguments
/// - `accounts: &mut <the account set>` (required) - The mutable reference to the account set to be set as `StarFrameInstruction::Accounts<'decode, 'arg>`
/// - `run_arg: <the run argument type>` (optional) - The run argument for the instruction. Defaults to `_run_arg: Self::RunArg<'_>`
/// - `ctx: &mut Context` (optional) - The context for the instruction. Defaults to `_ctx: &mut Context`
///
/// ## Return Type
/// - `Result<T>` (required) - The return type of the instruction. `T` will be set as `StarFrameInstruction::ReturnType`
///
/// # Example
/// ```
/// use star_frame::prelude::*;
/// # fn main() {}
/// #
/// # #[derive(StarFrameProgram)]
/// # #[program(instruction_set = (), id = System::ID, no_entrypoint)]
/// # pub struct MyProgram;
/// #
/// # #[zero_copy(pod)]
/// # #[derive(ProgramAccount)]
/// # pub struct CounterAccount {
/// #     pub authority: Pubkey,
/// #     pub count: u64,
/// # }
/// #
/// # #[derive(AccountSet)]
/// # pub struct InitializeAccounts {
/// #     pub counter: Mut<Account<CounterAccount>>,
/// #     pub authority: AccountInfo,
/// # }
///
/// #[derive(InstructionArgs, BorshDeserialize)]
/// # #[borsh(crate = "star_frame::borsh")]
/// pub struct Initialize {
///     #[ix_args(&mut run)]
///     pub start_at: Option<u64>,
/// }
///
/// // The second and third arguments are optional, so you don't need to include
/// // it if you aren't using them.
/// //
/// // If you hover over the function name ident, you can view the `StarFrameInstruction`
/// // trait documentation (after your instruction struct documentation).
/// #[star_frame_instruction]
/// fn Initialize(initialize_accounts: &mut InitializeAccounts, start_at: &mut Option<u64>) -> Result<()> {
///     **initialize_accounts.counter.data_mut()? = CounterAccount {
///         authority: *initialize_accounts.authority.pubkey(),
///         count: start_at.unwrap_or(0),
///     };
///     Ok(())
/// }
/// ```
#[proc_macro_error]
#[proc_macro_attribute]
pub fn star_frame_instruction(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    parse_macro_input!(args as Nothing);

    let out =
        star_frame_instruction::star_frame_instruction_impl(parse_macro_input!(item as ItemFn));
    out.into()
}

/// Compile time hashing of string literals.
///
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

/// Convenience wrapper around the common `bytemuck` derives and `repr` attribute.
///
/// Works with structs and enums, but omits the `repr` attribute on enums. This should be the first attribute on the item.
///
/// # Attributes
///
/// ## `#[zero_copy(pod, skip_packed)]` (item level attribute)
///
/// ### Syntax
///
/// - `pod` - (struct only) derives `Pod` instead of `CheckedBitPattern` and `NoUninit`
/// - `skip_packed` - (struct only) skips the `packed` attribute. We still add the `Align1` derive,
/// so all fields must be `Align1` if used.
///
/// # Example
/// ```
/// # use star_frame::prelude::*;
/// #[zero_copy]
/// struct MyStruct {
///     pub field: u64,
/// }
/// ```
///
/// is equivalent to:
///
/// ```
/// # use star_frame::prelude::*;
/// #[derive(Copy, Clone, Align1, Zeroable, NoUninit, CheckedBitPattern)]
/// #[repr(C, packed)]
/// struct MyStruct {
///     pub field: u64,
/// }
/// ```
#[proc_macro_error]
#[proc_macro_attribute]
pub fn zero_copy(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    zero_copy::zero_copy_impl(parse_macro_input!(input as DeriveInput), args.into()).into()
}

/// Compile time generation of a `Pubkey` from a base58 string literal.
// ---- Copied solana-program macros to use `star_frame::solana_program` path  ----
#[proc_macro]
pub fn pubkey(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    solana_pubkey::pubkey_impl(input)
}
