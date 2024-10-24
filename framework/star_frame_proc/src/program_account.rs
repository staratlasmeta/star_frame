use crate::hash::SIGHASH_ACCOUNT_NAMESPACE;
use crate::idl::TypeToIdlArgs;
use crate::util::Paths;
use easy_proc::{find_attr, ArgumentList};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Type;
use syn::*;

#[derive(Debug, ArgumentList, Clone, Default)]
pub struct ProgramAccountArgs {
    #[argument(presence)]
    pub skip_idl: bool,
    pub program: Option<Type>,
    pub seeds: Option<Type>,
}

pub fn program_account_impl(input: DeriveInput) -> TokenStream {
    let Paths {
        program_account_args_ident,
        ..
    } = &Paths::default();

    let args = find_attr(&input.attrs, program_account_args_ident)
        .map(ProgramAccountArgs::parse_arguments)
        .unwrap_or_default();

    program_account_impl_inner(input, args)
}

pub fn program_account_impl_inner(input: DeriveInput, args: ProgramAccountArgs) -> TokenStream {
    let Paths {
        macro_prelude: prelude,
        declared_program_type,
        ..
    } = &Paths::default();

    let owner_program = args.program.unwrap_or(declared_program_type.clone());
    let ident = &input.ident;

    let (impl_gen, ty_gen, where_clause) = input.generics.split_for_impl();

    let owner_program_impl = quote! {
        #[automatically_derived]
        impl #impl_gen #prelude::HasOwnerProgram for #ident #ty_gen #where_clause {
            type OwnerProgram = #owner_program;
        }
    };

    let account_ident_str = ident.to_string();
    let program_account_impl = quote! {
        #[automatically_derived]
        impl #impl_gen #prelude::ProgramAccount for #ident #ty_gen #where_clause {
            const DISCRIMINANT: <Self::OwnerProgram as #prelude::StarFrameProgram>::AccountDiscriminant = #prelude::sighash!(#SIGHASH_ACCOUNT_NAMESPACE, #account_ident_str);
        }
    };

    let has_seeds_impl = args.seeds.map(|seeds| {
        quote! {
            #[automatically_derived]
            impl #impl_gen #prelude::HasSeeds for #ident #ty_gen #where_clause {
                type Seeds = #seeds;
            }
        }
    });

    let idl_impl = (!args.skip_idl && cfg!(feature = "idl")).then(|| {
        let type_args = TypeToIdlArgs {
            program: Some(owner_program.clone()),
        };
        let type_to_idl_impl = crate::idl::derive_type_to_idl_inner(&input, type_args);
        quote!{
            #type_to_idl_impl

            #[automatically_derived]
            impl #impl_gen #prelude::AccountToIdl for #ident #ty_gen #where_clause {
                fn account_to_idl(idl_definition: &mut #prelude::IdlDefinition) -> #prelude::Result<#prelude::IdlAccountId> {
                    let source = #prelude::item_source::<Self>();
                    let idl_account = #prelude::IdlAccount {
                        discriminant: <Self as #prelude::ProgramAccount>::discriminant_bytes(),
                        type_def: <Self as #prelude::TypeToIdl>::type_to_idl(idl_definition)?,
                        // todo: Handle seeds! Need new trait to convert GetSeeds to IdlSeeds
                        seeds: None,
                    };
                    let namespace = idl_definition.add_account(idl_account, Self::AssociatedProgram::PROGRAM_ID)?;
                    Ok(#prelude::IdlAccountId {
                        namespace,
                        source,
                    })
                }
            }
        }
    });

    quote! {
        #owner_program_impl
        #program_account_impl
        #has_seeds_impl
        #idl_impl
    }
}
