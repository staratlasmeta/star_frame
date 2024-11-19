use crate::idl::{derive_type_to_idl_inner, TypeToIdlArgs};
use crate::program_account::{program_account_impl_inner, ProgramAccountArgs};
use crate::unsize::UnsizedTypeArgs;
use crate::util::cfg_idl;
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use syn::ItemStruct;

pub fn account_impl(input: &ItemStruct, args: &UnsizedTypeArgs) -> TokenStream {
    let derive_input = input.clone().into();
    if args.program_account {
        program_account_impl_inner(
            derive_input,
            ProgramAccountArgs {
                skip_idl: args.skip_idl,
                program: args.program.clone(),
                discriminant: args.discriminant.clone(),
                seeds: args.seeds.clone(),
            },
        )
    } else {
        if args.seeds.is_some() {
            abort!(args.seeds, "Seeds are only allowed with #[program_account]");
        }
        cfg_idl(args.skip_idl, || {
            derive_type_to_idl_inner(
                &derive_input,
                TypeToIdlArgs {
                    program: args.program.clone(),
                },
            )
        })
    }
}
