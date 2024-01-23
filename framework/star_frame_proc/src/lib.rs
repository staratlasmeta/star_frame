mod account_set;
mod framework_instruction;
mod instruction_set;
mod solana_pubkey;
mod ty;
mod unit_enum_from_repr;
mod util;

use crate::unit_enum_from_repr::unit_enum_from_repr_impl;
use proc_macro2::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use proc_macro_error::{abort, proc_macro_error};
use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::token::Token;
use syn::{
    parenthesized, parse_macro_input, parse_quote, token, Data, DataStruct, DataUnion, DeriveInput,
    Fields, Ident, LitInt, Token,
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
#[proc_macro_derive(FrameworkInstruction)]
pub fn derive_framework_instruction(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let out = framework_instruction::derive_framework_instruction_impl(parse_macro_input!(
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

/// Derives `InstructionSet` for a valid type.
#[proc_macro_error]
#[proc_macro_derive(InstructionSet)]
pub fn derive_instruction_set(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let out =
        instruction_set::derive_instruction_set_impl(&parse_macro_input!(item as DeriveInput));
    #[cfg(feature = "debug_instruction_set")]
    {
        println!("HELLO FROM THE MACRO");
        println!("{out}");
    }
    out.into()
}

#[proc_macro_error]
#[proc_macro_derive(InstructionSetToIdl)]
pub fn derive_instruction_set_to_idl(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let out = instruction_set::derive_instruction_set_to_idl_impl(parse_macro_input!(
        item as DeriveInput
    ));
    out.into()
}

/// Derives `TypeToIdl` for a valid type.
#[cfg(feature = "idl")]
#[proc_macro_error]
#[proc_macro_derive(TypeToIdl, attributes(program))]
pub fn derive_type_to_idl(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let out = ty::derive_type_to_idl(&parse_macro_input!(item as DeriveInput));
    // #[cfg(feature = "debug_type_to_idl")]
    // {
    //     println!("HELLO FROM THE MACRO");
    //     println!("{out}");
    // }
    out.into()
}

// ---- Copied solana-program macros to use `star_frame::solana_program` path  ----
#[proc_macro]
pub fn declare_id(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    solana_pubkey::program_declare_id_impl(input)
}

#[proc_macro]
pub fn pubkey(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    solana_pubkey::pubkey_impl(input)
}
