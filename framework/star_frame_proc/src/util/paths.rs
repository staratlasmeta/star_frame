use crate::util::get_crate_name;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{parse_quote, Type};

#[derive(Debug, Clone)]
pub struct Paths {
    pub crate_name: TokenStream,
    pub macro_prelude: TokenStream,

    // static_assertions
    pub static_assertions: TokenStream,

    pub prelude: TokenStream,
    // std
    pub box_ty: TokenStream,
    pub clone: TokenStream,
    pub copy: TokenStream,
    pub debug: TokenStream,
    pub default: TokenStream,
    pub deref: TokenStream,
    pub deref_mut: TokenStream,
    pub eq: TokenStream,
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
    pub syscalls: TokenStream,
    pub syscall_invoke: TokenStream,

    pub solana_runtime: TokenStream,

    pub result: TokenStream,

    // instruction
    pub star_frame_instruction: TokenStream,

    pub instruction: TokenStream,
    // program
    pub system_program: TokenStream,

    pub declared_program_type: Type,
    // idents
    pub account_ident: Ident,
    pub account_set_ident: Ident,
    pub decode_ident: Ident,
    pub validate_ident: Ident,
    pub cleanup_ident: Ident,
    pub idl_ident: Ident,
    pub star_frame_program_ident: Ident,
    pub program_id_ident: Ident,
    pub single_account_set_ident: Ident,
    pub instruction_set_args_ident: Ident,
    pub type_to_idl_args_ident: Ident,
    pub program_account_args_ident: Ident,
    pub instruction_to_idl_args_ident: Ident,
    pub get_seeds_ident: Ident,

    pub align1: TokenStream,
    pub packed_value_checked: TokenStream,
    pub advance: TokenStream,

    pub advance_array: TokenStream,

    // bytemuck
    pub checked: TokenStream,
    pub bytemuck: TokenStream,
    pub checked_bit_pattern: TokenStream,
    pub pod: TokenStream,
    // solana
    pub account_info: TokenStream,
    pub program_error: TokenStream,
    pub program_result: TokenStream,
    pub sol_memset: TokenStream,

    pub pubkey: TokenStream,

    pub msg: TokenStream,
    // anyhow
    pub anyhow_macro: TokenStream,
}

impl Default for Paths {
    fn default() -> Self {
        let crate_name = get_crate_name();
        Self {
            crate_name: crate_name.clone(),

            macro_prelude: quote! { #crate_name::__private::macro_prelude },
            prelude: quote! { #crate_name::prelude },

            // static_assertions
            static_assertions: quote! { #crate_name::static_assertions },

            // std
            box_ty: quote! { ::std::boxed::Box },
            clone: quote! { ::std::clone::Clone },
            copy: quote! { ::std::marker::Copy },
            debug: quote! { ::std::fmt::Debug },
            default: quote! { ::std::default::Default },
            deref: quote! { ::std::ops::Deref },
            deref_mut: quote! { ::std::ops::DerefMut },
            eq: quote! { ::std::cmp::Eq },
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
            syscalls: quote! { #crate_name::syscalls::Syscalls },
            syscall_invoke: quote! { #crate_name::syscalls::SyscallInvoke },
            solana_runtime: quote! { #crate_name::syscalls::solana_runtime::SolanaRuntime },

            result: quote! { #crate_name::Result },

            // instruction
            star_frame_instruction: quote! { #crate_name::instruction::StarFrameInstruction },
            instruction: quote! { #crate_name::instruction::Instruction },

            // program
            system_program: quote! { #crate_name::program::system_program::SystemProgram },
            declared_program_type: parse_quote! { crate::StarFrameDeclaredProgram },

            // idents
            account_ident: format_ident!("account"),
            account_set_ident: format_ident!("account_set"),
            decode_ident: format_ident!("decode"),
            validate_ident: format_ident!("validate"),
            cleanup_ident: format_ident!("cleanup"),
            idl_ident: format_ident!("idl"),
            type_to_idl_args_ident: format_ident!("type_to_idl"),
            program_account_args_ident: format_ident!("program_account"),
            instruction_to_idl_args_ident: format_ident!("instruction_to_idl"),
            star_frame_program_ident: format_ident!("program"),
            program_id_ident: format_ident!("program_id"),
            single_account_set_ident: format_ident!("single_account_set"),
            instruction_set_args_ident: format_ident!("ix_set"),
            get_seeds_ident: format_ident!("get_seeds"),

            align1: quote! { #crate_name::align1::Align1 },
            packed_value_checked: quote! { #crate_name::data_types::PackedValueChecked },
            advance_array: quote! { #crate_name::advance::AdvanceArray },
            advance: quote! { #crate_name::advance::Advance},

            // bytemuck
            bytemuck: quote! { #crate_name::bytemuck },
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

            // anyhow
            anyhow_macro: quote! { #crate_name::anyhow::anyhow },
        }
    }
}
