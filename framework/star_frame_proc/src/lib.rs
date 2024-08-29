#![allow(clippy::let_and_return)]

use proc_macro2::TokenStream;
#[cfg(feature = "idl")]
mod account;
mod account_set;
mod hash;
mod instruction_set;
#[cfg(feature = "idl")]
mod instruction_set_to_idl;
mod program;
mod solana_pubkey;
mod star_frame_instruction;
#[cfg(feature = "idl")]
mod ty;
mod unit_enum_from_repr;
mod unsize;
mod util;

#[cfg(feature = "idl")]
use crate::account::derive_account_to_idl_impl;
use crate::unit_enum_from_repr::unit_enum_from_repr_impl;
use proc_macro_crate::{crate_name, FoundCrate};
use proc_macro_error::{abort, proc_macro_error};
use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::{Comma, Token};
use syn::{
    parenthesized, parse_macro_input, parse_quote, token, Data, DataStruct, DataUnion, DeriveInput,
    Fields, Ident, Item, ItemEnum, LitInt, LitStr, Token,
};

fn get_crate_name() -> TokenStream {
    let generator_crate = crate_name("star_frame").expect("Could not find `star_frame`");
    match generator_crate {
        FoundCrate::Itself => quote! { star_frame },
        FoundCrate::Name(name) => {
            let ident = format_ident!("{}", name);
            quote! { ::#ident }
        }
    }
}

#[proc_macro_error]
#[proc_macro_derive(InstructionToIdl)]
pub fn derive_star_frame_instruction(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let out = star_frame_instruction::derive_star_frame_instruction_impl(parse_macro_input!(
        input as DeriveInput
    ));
    out.into()
}

#[proc_macro_error]
#[proc_macro_derive(AccountSet, attributes(account_set, decode, validate, cleanup, idl))]
pub fn derive_account_set(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let out = account_set::derive_account_set_impl(parse_macro_input!(input as DeriveInput));
    out.into()
}

/// Derives the `GetSeeds` trait for a struct.
///
/// # Attributes
///
/// ## 1. `#[seed_const = <expr>]` (item level attribute)
///
/// ### syntax
///
/// Attribute takes an `Expr` which resolves to a `&[u8]` seed for the account.
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
/// #[derive(Debug, GetSeeds)]
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
/// #[derive(Debug, GetSeeds)]
/// #[seed_const(Cool::DISC)]
/// pub struct TestAccount {}
/// ```
///
/// ```
/// # use star_frame::prelude::*;
/// // `seed_const` here resolves to the byte string `b"TEST_CONST"`
/// // Resulting `account.seeds()` is `vec![b"TEST_CONST".as_ref(), account.key.seed()];`
/// #[derive(Debug, GetSeeds)]
/// #[seed_const(b"TEST_CONST")]
/// pub struct TestAccount {
///     key: Pubkey,
/// }
/// ```
#[proc_macro_error]
#[proc_macro_derive(GetSeeds, attributes(seed_const))]
pub fn derive_get_seeds(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let out = account_set::seeded_account::derive_get_seeds_impl(parse_macro_input!(
        input as DeriveInput
    ));
    out.into()
}

/// Similar to strum's `FromRepr` derive but includes a trait for generic implementations and does not support non-unit enums.
#[proc_macro_error]
#[proc_macro_derive(UnitEnumFromRepr)]
pub fn derive_unit_enum_from_repr(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let out = unit_enum_from_repr_impl(parse_macro_input!(input as DeriveInput));
    out.into()
}

struct IdentWithArgs<A> {
    ident: Ident,
    args: Option<IdentArg<A>>,
}
impl<A> Parse for IdentWithArgs<A>
where
    A: Parse + Token,
{
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            ident: input.parse()?,
            args: if input.peek(token::Paren) {
                Some(input.parse()?)
            } else {
                None
            },
        })
    }
}
impl<A> ToTokens for IdentWithArgs<A>
where
    A: ToTokens,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ident.to_tokens(tokens);
        self.args.to_tokens(tokens);
    }
}

struct IdentArg<A> {
    paren: token::Paren,
    arg: Option<A>,
}
impl<A> Parse for IdentArg<A>
where
    A: Parse + Token,
{
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;

        Ok(Self {
            paren: parenthesized!(content in input),
            arg: content.parse()?,
        })
    }
}
impl<A> ToTokens for IdentArg<A>
where
    A: ToTokens,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.paren.surround(tokens, |tokens| {
            self.arg.to_tokens(tokens);
        });
    }
}

/// Derives `Align1` for a valid type.
#[proc_macro_error]
#[proc_macro_derive(Align1)]
pub fn derive_align1(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let crate_name = get_crate_name();
    let derive_input = parse_macro_input!(item as DeriveInput);
    match derive_input.data.clone() {
        Data::Struct(DataStruct { fields, .. }) => {
            derive_align1_for_struct(fields, derive_input, &crate_name)
        }
        Data::Union(DataUnion { fields, .. }) => {
            derive_align1_for_struct(Fields::Named(fields), derive_input, &crate_name)
        }
        Data::Enum(e) => {
            // TODO: Derive for repr u8 and unit enums
            for variant in e.variants {
                if variant.fields != Fields::Unit {
                    abort!(variant.fields, "Align1 only supports unit enums");
                }
            }

            abort!(e.enum_token, "Align1 cannot be derived for enums");
        }
    }
}

fn derive_align1_for_struct(
    fields: Fields,
    derive_input: DeriveInput,
    crate_name: &TokenStream,
) -> proc_macro::TokenStream {
    let packed = derive_input.attrs.into_iter().any(|attr| {
        attr.path().is_ident("repr") && {
            let Ok(args) = attr.parse_args_with(|p: ParseStream| {
                p.parse_terminated(IdentWithArgs::<LitInt>::parse, Token![,])
            }) else {
                abort!(attr, "Repr invalid args")
            };
            // args.iter().any(|arg|arg.ident.to_string() == "packed" && {
            //     if let Some(num) = arg.args {
            //
            //     }
            // });
            for arg in args {
                let ident = arg.ident.to_string();
                let arg = arg.args.as_ref().and_then(|a| a.arg.as_ref());
                if &ident == "align" && arg.map_or(false, |align| &align.to_string() != "1") {
                    abort!(arg, "`align` argument must be 1 to implement `Align1`");
                }
                if &ident == "packed" {
                    if arg.map_or(false, |align| &align.to_string() != "1") {
                        abort!(
                            arg,
                            "`packed` argument must be 1 or not present to implement `Align1`"
                        );
                    } else {
                        return true;
                    }
                }
            }
            false
        }
    });

    let ident = derive_input.ident;

    let mut gen = derive_input.generics;
    let wc = gen.make_where_clause();
    if !packed {
        for field in fields {
            let ty = field.ty;
            wc.predicates
                .push(parse_quote!(#ty: #crate_name::align1::Align1));
        }
    }
    let (impl_gen, type_gen, where_clause) = gen.split_for_impl();

    (quote! {
        unsafe impl #impl_gen #crate_name::align1::Align1 for #ident #type_gen #where_clause {}
    })
    .into()
}

/// Implements the `InstructionSet` trait for an enum of instructions.
///
/// By default, it uses a discriminant type of `[u8; 8]`, and derives each item discriminant by taking
/// the first 8 bytes of the sha256 hash in a compatible way with Anchor.
///
/// This can be overridden by passing in a `#[star_frame_instruction_set(<type>)]`
/// attribute to the enum, in which case it will use the enum reprs as the discriminants. The type must
/// be a valid enum repr type.
///
/// # Examples
///
/// Default 8 byte hash discriminants:
/// ```
/// use star_frame::impl_blank_ix;
/// use star_frame::prelude::*;
///
/// #[star_frame_instruction_set]
/// pub enum CoolIxSet {
///     //
///     CoolInstruction(CoolIx) = 123,
/// }
///
/// // hash from anchor
/// const IX_DISCRIMINANT: [u8; 8] = [197, 46, 153, 154, 189, 74, 154, 10];
///
/// assert_eq!(CoolIx::DISCRIMINANT, IX_DISCRIMINANT);
///
///
/// // An example instruction (which implements `StarFrameInstruction`)
/// pub struct CoolIx {}
/// # impl_blank_ix!(CoolIx);
/// ```
///
/// Using enum reprs as discriminants:
/// ```
/// use star_frame::impl_blank_ix;
/// use star_frame::prelude::*;
///
/// // Example Instructions (which implement `StarFrameInstruction`)
/// pub struct CoolIx1 {}
/// pub struct CoolIx3 {}
/// pub struct CoolIx2 {}
///
/// #[star_frame_instruction_set(u8)]
/// pub enum CoolIxSetU8 {
///     CoolInstruction1(CoolIx1),
///     CoolInstruction2(CoolIx2),
///     CoolInstruction3(CoolIx3) = 100,
/// }
/// assert_eq!(<CoolIx1 as InstructionDiscriminant<CoolIxSetU8>>::DISCRIMINANT, 0u8);
/// assert_eq!(<CoolIx2 as InstructionDiscriminant<CoolIxSetU8>>::DISCRIMINANT, 1u8);
/// assert_eq!(<CoolIx3 as InstructionDiscriminant<CoolIxSetU8>>::DISCRIMINANT, 100u8);
///
/// // The same instructions can be used in multiple instruction sets, since the
/// // `InstructionDiscriminant` trait is generic over the instruction set.
/// #[star_frame_instruction_set(i32)]
/// pub enum CoolIxSetU32 {
///     CoolInstruction1(CoolIx1) = -999,
///     CoolInstruction2(CoolIx2),
///     CoolInstruction3(CoolIx3) = 9999,
/// }
/// assert_eq!(<CoolIx1 as InstructionDiscriminant<CoolIxSetU32>>::DISCRIMINANT, -999i32);
/// assert_eq!(<CoolIx2 as InstructionDiscriminant<CoolIxSetU32>>::DISCRIMINANT, -998i32);
/// assert_eq!(<CoolIx3 as InstructionDiscriminant<CoolIxSetU32>>::DISCRIMINANT, 9999i32);
///
/// # impl_blank_ix!(CoolIx1, CoolIx2, CoolIx3);
/// ```
#[proc_macro_error]
#[proc_macro_attribute]
pub fn star_frame_instruction_set(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let out =
        instruction_set::instruction_set_impl(parse_macro_input!(item as ItemEnum), args.into());
    out.into()
}

#[proc_macro_error]
#[proc_macro_derive(InstructionSetToIdl)]
#[cfg_attr(not(feature = "idl"), allow(unused_variables))]
pub fn derive_instruction_set_to_idl(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    #[cfg(feature = "idl")]
    let out = instruction_set_to_idl::derive_instruction_set_to_idl_impl(parse_macro_input!(
        item as DeriveInput
    ));
    #[cfg(not(feature = "idl"))]
    let out = TokenStream::default();
    // println!("{}", out);
    out.into()
}

/// Derives `TypeToIdl` for a valid type.
#[cfg(feature = "idl")]
#[proc_macro_error]
#[proc_macro_derive(TypeToIdl, attributes(program))]
pub fn derive_type_to_idl(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let out = {
        #[cfg(feature = "idl")]
        {
            ty::derive_type_to_idl(&parse_macro_input!(item as DeriveInput))
        }
        #[cfg(not(feature = "idl"))]
        {
            TokenStream::default()
        }
    };
    // #[cfg(feature = "debug_type_to_idl")]
    // {
    //     println!("HELLO FROM THE MACRO");
    //     println!("{out}");
    // }
    out.into()
}

#[proc_macro_error]
#[proc_macro_derive(AccountToIdl, attributes(program))]
#[cfg_attr(not(feature = "idl"), allow(unused_variables))]
pub fn derive_account_to_idl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    #[cfg(feature = "idl")]
    let out = derive_account_to_idl_impl(&parse_macro_input!(input as DeriveInput));
    #[cfg(not(feature = "idl"))]
    let out = TokenStream::default();
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
///     closed_account_discriminant = [u8::MAX; 8],
///     no_entrypoint,
///     no_setup,
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
///     no_setup
/// )]
/// ```
/// - `instruction_set` - The enum that implements `InstructionSet` for the program. If the instruction set has a
/// lifetime (which it will if implemented with the [`macro@star_frame_instruction_set`] macro), it should be
/// passed in as `'static`.
/// - `id` - The program id for the program. This can be either a literal string in base58 ("AABBCC42")
/// or an expression that resolves to a `Pubkey`
/// - `account_discriminant` - The `AccountDiscriminant` type used for the program. Defaults to `[u8; 8]` (similarly to Anchor)
/// - `closed_account_discriminant` - The `AccountDiscriminant` value used for closed accounts. Defaults to `[u8::MAX; 8]`
/// - `no_entrypoint` - If present, the macro will not generate an entrypoint for the program.
/// While the generated entrypoint is already feature gated, this may be useful in some cases where features aren't convenient.
/// - `no_setup` - If present, the macro will not call the `program_setup!` macro. This is useful in libraries that may contain multiple programs.
#[proc_macro_error]
#[proc_macro_derive(StarFrameProgram, attributes(program))]
pub fn program(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let out = program::program_impl(parse_macro_input!(input as DeriveInput));
    out.into()
}

// ---- Copied solana-program macros to use `star_frame::solana_program` path  ----
#[proc_macro]
pub fn pubkey(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    solana_pubkey::pubkey_impl(input)
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
