use crate::get_crate_name;
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::{format_ident, quote};
use std::ops::{Deref, DerefMut};
use syn::parse::{Parse, ParseStream};
use syn::{bracketed, token, Attribute, Expr, ExprLit, Generics, Ident, Lit, Meta, MetaNameValue};

pub struct Paths {
    pub crate_name: TokenStream,

    pub account_info: TokenStream,
    pub result: TokenStream,
    pub account_set: TokenStream,
    pub account_set_decode: TokenStream,
    pub account_set_validate: TokenStream,
    pub account_set_cleanup: TokenStream,
    pub sys_call_invoke: TokenStream,
    #[cfg(feature = "idl")]
    pub account_set_to_idl: TokenStream,
    #[cfg(feature = "idl")]
    pub idl_definition: TokenStream,
    pub idl_account_set_def: TokenStream,
    pub idl_account_set: TokenStream,
    pub idl_account_set_struct_field: TokenStream,
    pub account_set_id: TokenStream,

    pub account_set_ident: Ident,
    pub decode_ident: Ident,
    pub validate_ident: Ident,
    pub cleanup_ident: Ident,
}
impl Default for Paths {
    fn default() -> Self {
        let crate_name = get_crate_name();
        Self {
            account_info: quote! { #crate_name::solana_program::account_info::AccountInfo },
            result: quote! { #crate_name::Result },
            account_set: quote! { #crate_name::account_set::AccountSet },
            account_set_decode: quote! { #crate_name::account_set::AccountSetDecode },
            account_set_validate: quote! { #crate_name::account_set::AccountSetValidate },
            account_set_cleanup: quote! { #crate_name::account_set::AccountSetCleanup },
            sys_call_invoke: quote! { #crate_name::sys_calls::SysCallInvoke },
            #[cfg(feature = "idl")]
            account_set_to_idl: quote! { #crate_name::idl::AccountSetToIdl },
            #[cfg(feature = "idl")]
            idl_definition: quote! { #crate_name::star_frame_idl::IdlDefinition },
            idl_account_set_def: quote! { #crate_name::star_frame_idl::account_set::IdlAccountSetDef },
            idl_account_set: quote! { #crate_name::star_frame_idl::account_set::IdlAccountSet },
            idl_account_set_struct_field: quote! { #crate_name::star_frame_idl::account_set::IdlAccountSetStructField },
            account_set_id: quote! { #crate_name::star_frame_idl::account_set::AccountSetId },

            account_set_ident: format_ident!("account_set"),
            decode_ident: format_ident!("decode"),
            validate_ident: format_ident!("validate"),
            cleanup_ident: format_ident!("cleanup"),

            crate_name,
        }
    }
}

pub struct BracketedGenerics {
    _bracket: token::Bracket,
    generics: Generics,
}
impl BracketedGenerics {
    pub fn into_inner(self) -> Generics {
        self.generics
    }
}
impl Deref for BracketedGenerics {
    type Target = Generics;

    fn deref(&self) -> &Self::Target {
        &self.generics
    }
}
impl DerefMut for BracketedGenerics {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.generics
    }
}
impl Parse for BracketedGenerics {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _bracket = bracketed!(content in input);
        let generics = content.parse()?;
        Ok(Self { _bracket, generics })
    }
}

#[allow(dead_code)]
pub fn get_docs<'a>(attrs: impl IntoIterator<Item = &'a Attribute>) -> String {
    attrs
        .into_iter()
        .filter(|a| a.path().is_ident("doc"))
        .map(|a: &'a Attribute| {
            if let Meta::NameValue(MetaNameValue {
                value:
                    Expr::Lit(ExprLit {
                        lit: Lit::Str(str), ..
                    }),
                ..
            }) = &a.meta
            {
                str.value()
            } else {
                abort!(a, "Expected doc attribute to be a name value pair")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
