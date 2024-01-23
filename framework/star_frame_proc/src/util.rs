use crate::get_crate_name;
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::{format_ident, quote};
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{
    bracketed, parse_quote, token, Attribute, ConstParam, Expr, ExprLit, GenericParam, Generics,
    Ident, Lifetime, LifetimeParam, Lit, Meta, MetaNameValue, Token, Type, TypeParam,
};

pub struct Paths {
    pub crate_name: TokenStream,

    pub account_info: TokenStream,
    pub result: TokenStream,
    pub account_set: TokenStream,
    pub account_set_decode: TokenStream,
    pub account_set_validate: TokenStream,
    pub account_set_cleanup: TokenStream,
    pub sys_call_invoke: TokenStream,
    pub system_program: TokenStream,
    #[cfg(feature = "idl")]
    pub account_set_to_idl: TokenStream,
    // TODO - put behind feature flag
    pub instruction_to_idl: TokenStream,
    pub instruction_set_to_idl: TokenStream,
    #[cfg(feature = "idl")]
    pub type_to_idl: TokenStream,

    pub semver: TokenStream,
    pub idl_definition: TokenStream,
    pub idl_type_def: TokenStream,
    pub idl_field: TokenStream,
    pub idl_account_set_def: TokenStream,
    pub idl_account_set: TokenStream,
    pub idl_account_set_struct_field: TokenStream,
    pub idl_instruction_def: TokenStream,
    pub idl_instruction: TokenStream,
    pub account_set_id: TokenStream,
    pub framework_instruction: TokenStream,

    pub account_set_ident: Ident,
    pub decode_ident: Ident,
    pub validate_ident: Ident,
    pub cleanup_ident: Ident,
    pub idl_ident: Ident,
    pub idl_ty_program_ident: Ident,
    pub declared_program_type: Type,
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
            system_program: quote! { #crate_name::program::system_program::SystemProgram },
            #[cfg(feature = "idl")]
            account_set_to_idl: quote! { #crate_name::idl::AccountSetToIdl },
            instruction_to_idl: quote! { #crate_name::idl::InstructionToIdl },
            instruction_set_to_idl: quote! { #crate_name::idl::InstructionSetToIdl },
            #[cfg(feature = "idl")]
            type_to_idl: quote! { #crate_name::idl::ty::TypeToIdl },

            semver: quote! { #crate_name::star_frame_idl::SemVer },
            idl_definition: quote! { #crate_name::star_frame_idl::IdlDefinition },
            idl_type_def: quote! { #crate_name::star_frame_idl::ty::IdlTypeDef },
            idl_field: quote! { #crate_name::star_frame_idl::ty::IdlField },
            idl_account_set_def: quote! { #crate_name::star_frame_idl::account_set::IdlAccountSetDef },
            idl_account_set: quote! { #crate_name::star_frame_idl::account_set::IdlAccountSet },
            idl_account_set_struct_field: quote! { #crate_name::star_frame_idl::account_set::IdlAccountSetStructField },
            idl_instruction_def: quote! { #crate_name::star_frame_idl::instruction::IdlInstructionDef },
            idl_instruction: quote! { #crate_name::star_frame_idl::instruction::IdlInstruction },

            account_set_id: quote! { #crate_name::star_frame_idl::account_set::AccountSetId },
            framework_instruction: quote! { #crate_name::instruction::FrameworkInstruction },

            account_set_ident: format_ident!("account_set"),
            decode_ident: format_ident!("decode"),
            validate_ident: format_ident!("validate"),
            cleanup_ident: format_ident!("cleanup"),
            idl_ident: format_ident!("idl"),
            idl_ty_program_ident: format_ident!("program"),
            declared_program_type: parse_quote! { crate::StarFrameDeclaredProgram },

            crate_name,
        }
    }
}

#[derive(Debug)]
pub struct BetterGenerics {
    _bracket: token::Bracket,
    generics: Generics,
}
impl BetterGenerics {
    pub fn into_inner(self) -> Generics {
        self.generics
    }
}
impl Deref for BetterGenerics {
    type Target = Generics;

    fn deref(&self) -> &Self::Target {
        &self.generics
    }
}
impl DerefMut for BetterGenerics {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.generics
    }
}
impl Parse for BetterGenerics {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _bracket = bracketed!(content in input);
        let mut generics = if !content.peek(Token![<]) {
            Generics::default()
        } else {
            let lt_token: Token![<] = content.parse()?;

            let mut params = Punctuated::new();
            loop {
                if content.peek(Token![>]) {
                    break;
                }

                let attrs = content.call(Attribute::parse_outer)?;
                let lookahead = content.lookahead1();
                if lookahead.peek(Lifetime) {
                    params.push_value(GenericParam::Lifetime(LifetimeParam {
                        attrs,
                        ..content.parse()?
                    }));
                } else if lookahead.peek(Ident) {
                    params.push_value(GenericParam::Type(TypeParam {
                        attrs,
                        ..content.parse()?
                    }));
                } else if lookahead.peek(Token![const]) {
                    params.push_value(GenericParam::Const(ConstParam {
                        attrs,
                        ..content.parse()?
                    }));
                } else {
                    return Err(lookahead.error());
                }

                if content.peek(Token![>]) {
                    break;
                }
                let punct = content.parse()?;
                params.push_punct(punct);
            }

            let gt_token: Token![>] = content.parse()?;

            Generics {
                lt_token: Some(lt_token),
                params,
                gt_token: Some(gt_token),
                where_clause: None,
            }
        };
        generics.where_clause = content.parse()?;
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
