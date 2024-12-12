use easy_proc::ArgumentList;
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Bracket;
use syn::{bracketed, Expr, Item, Meta, Token, Type};

mod account;
mod struct_impl;

#[derive(Debug, Clone, Default)]
pub struct UnsizedAttributeMetas {
    _bracket: Bracket,
    attributes: Punctuated<Meta, Token![,]>,
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
    #[argument(presence)]
    pub program_account: bool,
    #[argument(presence)]
    pub skip_idl: bool,
    pub program: Option<Type>,
    pub seeds: Option<Type>,
    pub discriminant: Option<Expr>,
}

pub fn unsized_type_impl(item: Item, args: TokenStream) -> TokenStream {
    match item {
        Item::Struct(struct_item) => struct_impl::unsized_type_struct_impl(struct_item, args),
        Item::Enum(_enum_item) => {
            abort!(
                args,
                "unsized_type cannot be applied to enums yet. It will be supported in the future. (soonTM)"
            )
        }
        _ => {
            abort!(
                args,
                "unsized_type can only be applied to structs and enums"
            )
        }
    }
}
