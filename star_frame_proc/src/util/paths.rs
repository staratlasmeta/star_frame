use crate::util::get_crate_name;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{parse_quote, Type};

#[derive(Debug, Clone)]
pub struct Paths {
    pub crate_name: TokenStream,
    pub prelude: TokenStream,

    // std
    pub sized: TokenStream,
    #[allow(dead_code)]
    pub box_ty: TokenStream,
    pub clone: TokenStream,
    pub copy: TokenStream,
    pub debug: TokenStream,
    #[allow(dead_code)]
    pub default: TokenStream,
    #[allow(dead_code)]
    pub deref: TokenStream,
    #[allow(dead_code)]
    pub deref_mut: TokenStream,
    pub eq: TokenStream,
    pub partial_eq: TokenStream,
    #[allow(dead_code)]
    pub partial_ord: TokenStream,
    #[allow(dead_code)]
    pub ord: TokenStream,
    pub phantom_data: TokenStream,
    #[allow(dead_code)]
    pub ptr: TokenStream,

    pub size_of: TokenStream,

    // account set
    pub account_set_decode: TokenStream,
    pub account_set_validate: TokenStream,
    pub account_set_cleanup: TokenStream,
    pub result: TokenStream,
    pub instruction: TokenStream,
    pub declared_program_type: Type,
    // idents
    pub account_set_ident: Ident,
    pub decode_ident: Ident,
    pub validate_ident: Ident,
    pub cleanup_ident: Ident,
    pub idl_ident: Ident,
    pub star_frame_program_ident: Ident,
    pub single_account_set_ident: Ident,
    pub instruction_set_args_ident: Ident,
    pub type_to_idl_args_ident: Ident,
    pub program_account_args_ident: Ident,
    pub instruction_to_idl_args_ident: Ident,
    pub ix_args_ident: Ident,
    pub instruction_args_ident: Ident,
    pub get_seeds_ident: Ident,

    // bytemuck
    pub bytemuck: TokenStream,

    // solana
    pub account_info: TokenStream,
    pub pubkey: TokenStream,
}

macro_rules! paths_macro {
    ($($name:ident $(: $rename:ident)? $(,)?)*) => {
        let Paths {
            $($name $(: $rename)? ,)*
            ..
        } = Default::default();
    };
}

pub(crate) use paths_macro as Paths;

impl Default for Paths {
    fn default() -> Self {
        let crate_name = get_crate_name();
        let prelude = quote! { #crate_name::__private::macro_prelude };
        Self {
            crate_name: crate_name.clone(),

            // std
            sized: quote! { ::core::marker::Sized },
            box_ty: quote! { ::std::boxed::Box },
            clone: quote! { ::core::clone::Clone },
            copy: quote! { ::core::marker::Copy },
            debug: quote! { ::core::fmt::Debug },
            default: quote! { ::core::default::Default },
            deref: quote! { ::core::ops::Deref },
            deref_mut: quote! { ::core::ops::DerefMut },
            eq: quote! { ::core::cmp::Eq },
            partial_eq: quote! { ::core::cmp::PartialEq },
            ord: quote! { ::core::cmp::Ord },
            partial_ord: quote! { ::core::cmp::PartialOrd },
            phantom_data: quote! { ::core::marker::PhantomData },
            ptr: quote! { ::core::ptr },
            size_of: quote! { ::core::mem::size_of },
            // account set
            account_set_decode: quote! { #crate_name::account_set::AccountSetDecode },
            account_set_validate: quote! { #crate_name::account_set::AccountSetValidate },
            account_set_cleanup: quote! { #crate_name::account_set::AccountSetCleanup },
            result: quote! { #prelude::Result },

            // instruction
            instruction: quote! { #crate_name::instruction::Instruction },
            // program
            declared_program_type: parse_quote! { crate::StarFrameDeclaredProgram },
            // idents
            account_set_ident: format_ident!("account_set"),
            decode_ident: format_ident!("decode"),
            validate_ident: format_ident!("validate"),
            cleanup_ident: format_ident!("cleanup"),
            idl_ident: format_ident!("idl"),
            type_to_idl_args_ident: format_ident!("type_to_idl"),
            program_account_args_ident: format_ident!("program_account"),
            instruction_to_idl_args_ident: format_ident!("instruction_to_idl"),
            star_frame_program_ident: format_ident!("program"),
            single_account_set_ident: format_ident!("single_account_set"),
            instruction_set_args_ident: format_ident!("ix_set"),
            ix_args_ident: format_ident!("ix_args"),
            instruction_args_ident: format_ident!("instruction_args"),
            get_seeds_ident: format_ident!("get_seeds"),

            // bytemuck
            bytemuck: quote! { #crate_name::bytemuck },
            // solana
            account_info: quote! { #prelude::AccountInfo },
            pubkey: quote! { #prelude::Pubkey },
            prelude,
        }
    }
}

pub fn pretty_path(path: &TokenStream) -> String {
    let path = path.to_string();
    path.replace(" :: ", "::")
}
