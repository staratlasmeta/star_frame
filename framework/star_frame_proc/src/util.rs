use crate::get_crate_name;
use proc_macro2::TokenStream;
use proc_macro_error::{abort, abort_call_site};
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
    // std
    pub box_ty: TokenStream,
    pub clone: TokenStream,
    pub copy: TokenStream,
    pub debug: TokenStream,
    pub default: TokenStream,
    pub deref: TokenStream,
    pub deref_mut: TokenStream,
    pub eq: TokenStream,
    pub non_null: TokenStream,
    pub panic: TokenStream,
    pub partial_eq: TokenStream,
    pub phantom_data: TokenStream,
    pub ptr: TokenStream,
    pub size_of: TokenStream,

    // derivative
    pub derivative: TokenStream,

    // account set
    pub account_set: TokenStream,
    pub account_set_decode: TokenStream,
    pub account_set_validate: TokenStream,
    pub account_set_cleanup: TokenStream,

    // syscalls
    pub sys_call_invoke: TokenStream,
    pub solana_runtime: TokenStream,

    pub result: TokenStream,

    // idl
    #[cfg(feature = "idl")]
    pub account_to_idl: TokenStream,
    #[cfg(feature = "idl")]
    pub account_set_to_idl: TokenStream,
    #[cfg(feature = "idl")]
    pub instruction_to_idl: TokenStream,
    #[cfg(feature = "idl")]
    pub instruction_set_to_idl: TokenStream,
    #[cfg(feature = "idl")]
    pub type_to_idl: TokenStream,

    // star frame idl
    pub semver: TokenStream,
    pub idl_definition: TokenStream,
    pub idl_definition_ref: TokenStream,
    pub idl_type_def: TokenStream,
    pub idl_field: TokenStream,
    pub idl_account: TokenStream,
    pub idl_account_set_def: TokenStream,
    pub idl_account_set: TokenStream,
    pub idl_account_set_struct_field: TokenStream,
    pub idl_instruction_def: TokenStream,
    pub idl_instruction: TokenStream,
    pub idl_seeds: TokenStream,
    pub account_id: TokenStream,
    pub account_set_id: TokenStream,

    // instruction
    pub framework_instruction: TokenStream,
    pub instruction_set: TokenStream,
    pub instruction: TokenStream,

    // program
    pub system_program: TokenStream,
    pub star_frame_program: TokenStream,
    pub declared_program_type: Type,

    // idents
    pub account_ident: Ident,
    pub account_set_ident: Ident,
    pub decode_ident: Ident,
    pub validate_ident: Ident,
    pub cleanup_ident: Ident,
    pub idl_ident: Ident,
    pub idl_ty_program_ident: Ident,

    pub align1: TokenStream,
    pub packed_value_checked: TokenStream,
    pub advance: TokenStream,

    // serialize
    pub build_pointer: TokenStream,
    pub build_pointer_mut: TokenStream,
    pub enum_ref_mut_wrapper: TokenStream,
    pub enum_ref_wrapper: TokenStream,
    pub framework_from_bytes: TokenStream,
    pub framework_from_bytes_mut: TokenStream,
    pub framework_init: TokenStream,
    pub framework_serialize: TokenStream,
    pub pointer_breakup: TokenStream,
    pub resize_fn: TokenStream,
    pub unsized_enum: TokenStream,
    pub unsized_type: TokenStream,

    // bytemuck
    pub checked: TokenStream,
    pub checked_bit_pattern: TokenStream,
    pub pod: TokenStream,

    // solana
    pub account_info: TokenStream,
    pub program_error: TokenStream,
    pub program_result: TokenStream,
    pub sol_memset: TokenStream,
    pub pubkey: TokenStream,
    pub msg: TokenStream,

    pub crate_name: TokenStream,
}
impl Default for Paths {
    fn default() -> Self {
        let crate_name = get_crate_name();
        Self {
            // std
            box_ty: quote! { ::std::boxed::Box },
            clone: quote! { ::std::clone::Clone },
            copy: quote! { ::std::marker::Copy },
            debug: quote! { ::std::fmt::Debug },
            default: quote! { ::std::default::Default },
            deref: quote! { ::std::ops::Deref },
            deref_mut: quote! { ::std::ops::DerefMut },
            eq: quote! { ::std::cmp::Eq },
            non_null: quote! { ::std::ptr::NonNull },
            panic: quote! { ::std::panic },
            partial_eq: quote! { ::std::cmp::PartialEq },
            phantom_data: quote! { ::std::marker::PhantomData },
            ptr: quote! { ::std::ptr },
            size_of: quote! { ::std::mem::size_of },

            // derivative
            derivative: quote! { #crate_name::derivative::Derivative },

            // account set
            account_set: quote! { #crate_name::account_set::AccountSet },
            account_set_decode: quote! { #crate_name::account_set::AccountSetDecode },
            account_set_validate: quote! { #crate_name::account_set::AccountSetValidate },
            account_set_cleanup: quote! { #crate_name::account_set::AccountSetCleanup },

            // syscalls
            sys_call_invoke: quote! { #crate_name::sys_calls::SysCallInvoke },
            solana_runtime: quote! { #crate_name::sys_calls::solana_runtime::SolanaRuntime },

            result: quote! { #crate_name::Result },

            // idl
            #[cfg(feature = "idl")]
            account_to_idl: quote! { #crate_name::idl::AccountToIdl },
            #[cfg(feature = "idl")]
            account_set_to_idl: quote! { #crate_name::idl::AccountSetToIdl },
            #[cfg(feature = "idl")]
            instruction_to_idl: quote! { #crate_name::idl::InstructionToIdl },
            #[cfg(feature = "idl")]
            instruction_set_to_idl: quote! { #crate_name::idl::InstructionSetToIdl },
            #[cfg(feature = "idl")]
            type_to_idl: quote! { #crate_name::idl::ty::TypeToIdl },

            // star frame idl
            semver: quote! { #crate_name::star_frame_idl::SemVer },
            idl_definition: quote! { #crate_name::star_frame_idl::IdlDefinition },
            idl_definition_ref: quote! { #crate_name::star_frame_idl::IdlDefinitionReference },
            idl_type_def: quote! { #crate_name::star_frame_idl::ty::IdlTypeDef },
            idl_field: quote! { #crate_name::star_frame_idl::ty::IdlField },
            idl_account: quote! { #crate_name::star_frame_idl::account::IdlAccount },
            idl_account_set_def: quote! { #crate_name::star_frame_idl::account_set::IdlAccountSetDef },
            idl_account_set: quote! { #crate_name::star_frame_idl::account_set::IdlAccountSet },
            idl_account_set_struct_field: quote! { #crate_name::star_frame_idl::account_set::IdlAccountSetStructField },
            idl_instruction_def: quote! { #crate_name::star_frame_idl::instruction::IdlInstructionDef },
            idl_instruction: quote! { #crate_name::star_frame_idl::instruction::IdlInstruction },
            idl_seeds: quote! { #crate_name::star_frame_idl::seeds::IdlSeeds },
            account_id: quote! { #crate_name::star_frame_idl::account::AccountId },
            account_set_id: quote! { #crate_name::star_frame_idl::account_set::AccountSetId },

            // instruction
            framework_instruction: quote! { #crate_name::instruction::FrameworkInstruction },
            instruction_set: quote! { #crate_name::instruction::InstructionSet },
            instruction: quote! { #crate_name::instruction::Instruction },

            // program
            system_program: quote! { #crate_name::program::system_program::SystemProgram },
            star_frame_program: quote! { #crate_name::program::StarFrameProgram },
            declared_program_type: parse_quote! { crate::StarFrameDeclaredProgram },

            // idents
            account_ident: format_ident!("account"),
            account_set_ident: format_ident!("account_set"),
            decode_ident: format_ident!("decode"),
            validate_ident: format_ident!("validate"),
            cleanup_ident: format_ident!("cleanup"),
            idl_ident: format_ident!("idl"),
            idl_ty_program_ident: format_ident!("program"),

            align1: quote! { #crate_name::align1::Align1 },
            packed_value_checked: quote! { #crate_name::packed_value::PackedValueChecked },
            advance: quote! { #crate_name::advance::Advance},

            // serialize
            build_pointer: quote! { #crate_name::serialize::pointer_breakup::BuildPointer },
            build_pointer_mut: quote! { #crate_name::serialize::pointer_breakup::BuildPointerMut },
            enum_ref_mut_wrapper: quote! { #crate_name::serialize::unsized_enum::EnumRefMutWrapper },
            enum_ref_wrapper: quote! { #crate_name::serialize::unsized_enum::EnumRefWrapper },
            framework_from_bytes: quote! { #crate_name::serialize::FrameworkFromBytes },
            framework_from_bytes_mut: quote! { #crate_name::serialize::FrameworkFromBytesMut },
            framework_init: quote! { #crate_name::serialize::FrameworkInit },
            framework_serialize: quote! { #crate_name::serialize::FrameworkSerialize },
            pointer_breakup: quote! { #crate_name::serialize::pointer_breakup::PointerBreakup },
            resize_fn: quote! { #crate_name::serialize::ResizeFn },
            unsized_enum: quote! { #crate_name::serialize::unsized_enum::UnsizedEnum },
            unsized_type: quote! { #crate_name::serialize::unsized_type::UnsizedType },

            // bytemuck
            checked: quote! { #crate_name::bytemuck::checked },
            checked_bit_pattern: quote! { #crate_name::bytemuck::checked::CheckedBitPattern },
            pod: quote! { #crate_name::bytemuck::Pod },

            // solana
            account_info: quote! { #crate_name::solana_program::account_info::AccountInfo },
            program_error: quote! { #crate_name::solana_program::program_error::ProgramError },
            program_result: quote! { #crate_name::solana_program::entrypoint::ProgramResult },
            sol_memset: quote! { #crate_name::solana_program::program_memory::sol_memset },
            pubkey: quote! { #crate_name::solana_program::pubkey::Pubkey },
            msg: quote! { #crate_name::solana_program::msg },

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

pub fn verify_repr(
    attrs: &[Attribute],
    repr_required: impl IntoIterator<Item = Ident>,
    allow_others: bool,
    require_present: bool,
) -> Punctuated<Ident, Token![,]> {
    let repr = attrs.iter().find(|attr| attr.path().is_ident("repr"));
    if let Some(repr) = repr {
        let repr_ty = repr
            .parse_args_with(|p: ParseStream| p.parse_terminated(Ident::parse, Token![,]))
            .unwrap_or_else(|e| abort!(repr, "Could not parse repr type: {}", e));
        let mut repr_required = repr_required
            .into_iter()
            .map(|r| (r, false))
            .collect::<Vec<_>>();
        for repr_ty in repr_ty.iter() {
            if let Some((_, found)) = repr_required.iter_mut().find(|(r, _)| r == repr_ty) {
                *found = true;
            } else if !allow_others {
                abort!(repr_ty, "Unexpected repr type: {}", quote! { #repr_ty });
            }
        }
        for (r, found) in repr_required {
            if !found {
                abort_call_site!("Missing #[repr({:?})] attribute", r);
            }
        }
        repr_ty
    } else if require_present {
        abort_call_site!(
            "Missing #[repr({:?})] attribute",
            repr_required.into_iter().collect::<Vec<_>>()
        );
    } else {
        Punctuated::new()
    }
}
