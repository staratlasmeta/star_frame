use crate::util::GetGenerics;
use easy_proc::ArgumentList;
use itertools::Itertools;
use proc_macro2::TokenStream;
use proc_macro_error2::abort;
use quote::ToTokens;
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    token::Bracket,
    Attribute, Expr, Item, Meta, Path, Token, Type,
};

mod account;
mod enum_impl;
mod impl_impl;
mod struct_impl;
pub use impl_impl::*;

#[derive(Debug, Clone, Default)]
pub struct UnsizedAttributeMetas {
    _bracket: Bracket,
    attributes: Punctuated<Meta, Token![,]>,
}

impl ToTokens for UnsizedAttributeMetas {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self._bracket
            .surround(tokens, |tokens| self.attributes.to_tokens(tokens));
    }
}

impl Parse for UnsizedAttributeMetas {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attributes;
        Ok(Self {
            _bracket: bracketed!(attributes in input),
            attributes: attributes.parse_terminated(Meta::parse, Token![,])?,
        })
    }
}

// todo: figure out what args this may need
// todo: derives for each new struct. allow disabling unnecessary default derives
#[derive(ArgumentList, Debug, Clone)]
pub struct UnsizedTypeArgs {
    #[argument(default)]
    pub owned_attributes: UnsizedAttributeMetas,
    pub owned_type: Option<Type>,
    pub owned_from_ref: Option<Path>,
    #[argument(default)]
    pub sized_attributes: UnsizedAttributeMetas,
    #[argument(presence)]
    pub program_account: bool,
    #[argument(presence)]
    pub skip_idl: bool,
    #[argument(presence)]
    pub skip_phantom_generics: bool,
    pub program: Option<Type>,
    pub seeds: Option<Type>,
    pub discriminant: Option<Expr>,
}

impl UnsizedTypeArgs {
    pub fn validate(&self) {
        if let Some(owned_type) = &self.owned_type {
            if !self.owned_attributes.attributes.is_empty() {
                abort!(
                    owned_type,
                    "owned_attributes cannot be used with a custom owned_type"
                )
            }
            if self.owned_from_ref.is_none() {
                abort!(
                    owned_type,
                    "owned_from_ref must be specified when using a custom owned_type"
                )
            }
        }
    }
}

pub fn reject_non_ty_gen(item: &impl GetGenerics) {
    let generics = item.get_generics();
    if !generics.lifetimes().collect_vec().is_empty() {
        abort!(generics, "Lifetimes are not allowed in unsized types")
    }

    if !generics.const_params().collect_vec().is_empty() {
        abort!(
            generics,
            "Const generics are not allowed in unsized types (yet)"
        )
    }
}

pub fn unsized_type_impl(item: Item, args: TokenStream) -> TokenStream {
    let args_attr: Attribute = parse_quote!(#[unsized_type(#args)]);
    let unsized_args = UnsizedTypeArgs::parse_arguments(&args_attr);
    unsized_args.validate();
    match item {
        Item::Struct(struct_item) => {
            reject_non_ty_gen(&struct_item);
            struct_impl::unsized_type_struct_impl(struct_item, unsized_args)
        }
        Item::Enum(enum_item) => {
            reject_non_ty_gen(&enum_item);
            enum_impl::unsized_type_enum_impl(enum_item, unsized_args)
        }
        _ => {
            abort!(
                args,
                "unsized_type can only be applied to structs and enums"
            )
        }
    }
}
