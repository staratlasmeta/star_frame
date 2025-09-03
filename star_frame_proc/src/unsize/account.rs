use crate::{
    idl::{derive_type_to_idl_inner, TypeToIdlArgs},
    program_account::{program_account_impl_inner, ProgramAccountArgs},
    unsize::UnsizedTypeArgs,
};
use proc_macro2::TokenStream;
use syn::DeriveInput;

pub fn account_impl(input: &DeriveInput, args: &UnsizedTypeArgs) -> TokenStream {
    if args.program_account {
        program_account_impl_inner(
            input.clone(),
            ProgramAccountArgs {
                skip_idl: args.skip_idl,
                program: args.program.clone(),
                discriminant: args.discriminant.clone(),
                seeds: args.seeds.clone(),
            },
        )
    } else if !args.skip_idl {
        derive_type_to_idl_inner(
            input,
            TypeToIdlArgs {
                program: args.program.clone(),
            },
        )
    } else {
        Default::default()
    }
}
