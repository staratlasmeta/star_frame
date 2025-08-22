use crate::{
    hash::SIGHASH_ACCOUNT_NAMESPACE,
    idl::TypeToIdlArgs,
    util::{ignore_cfg_module, reject_attributes, Paths},
};
use easy_proc::{find_attr, ArgumentList};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Type, *};

#[derive(Debug, ArgumentList, Clone, Default)]
pub struct ProgramAccountArgs {
    #[argument(presence)]
    pub skip_idl: bool,
    pub program: Option<Type>,
    pub seeds: Option<Type>,
    pub discriminant: Option<Expr>,
}

pub fn program_account_impl(input: DeriveInput) -> TokenStream {
    Paths!(program_account_args_ident);

    let args = find_attr(&input.attrs, &program_account_args_ident)
        .map(ProgramAccountArgs::parse_arguments)
        .unwrap_or_default();

    program_account_impl_inner(input, args)
}

pub fn program_account_impl_inner(input: DeriveInput, args: ProgramAccountArgs) -> TokenStream {
    Paths!(
        prelude,
        result,
        type_to_idl_args_ident,
        declared_program_type
    );

    reject_attributes(&input.attrs, &type_to_idl_args_ident, None);

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
    let discriminant = args.discriminant.unwrap_or_else(
        || parse_quote!(#prelude::sighash!(#SIGHASH_ACCOUNT_NAMESPACE, #account_ident_str)),
    );
    let program_account_impl = quote! {
        #[automatically_derived]
        impl #impl_gen #prelude::ProgramAccount for #ident #ty_gen #where_clause {
            const DISCRIMINANT: <Self::OwnerProgram as #prelude::StarFrameProgram>::AccountDiscriminant = #discriminant;
        }
    };

    let has_seeds_impl = args.seeds.as_ref().map(|seeds| {
        quote! {
            #[automatically_derived]
            impl #impl_gen #prelude::HasSeeds for #ident #ty_gen #where_clause {
                type Seeds = #seeds;
            }
        }
    });

    let idl_impl =(!args.skip_idl).then( || {
        let type_args = TypeToIdlArgs {
            program: Some(owner_program.clone()),
        };
        let type_to_idl_impl = crate::idl::derive_type_to_idl_inner(&input, type_args);

        let seeds = match &args.seeds {
            Some(seeds) => {
                quote! { Some(<#seeds as #prelude::SeedsToIdl>::seeds_to_idl(idl_definition)?) }
            }
            None => quote! { None },
        };

        let account_to_idl_impl = ignore_cfg_module(ident, "_account_to_idl", quote! {
            #[cfg(all(feature = "idl", not(target_os = "solana")))]
            #[automatically_derived]
            impl #impl_gen #prelude::AccountToIdl for #ident #ty_gen #where_clause {
                fn account_to_idl(idl_definition: &mut #prelude::IdlDefinition) -> #result<#prelude::IdlAccountId> {
                    let source = #prelude::item_source::<Self>();
                    let type_def = <Self as #prelude::TypeToIdl>::type_to_idl(idl_definition)?;
                    let type_id = type_def.assert_defined()?.clone();
                    let idl_account = #prelude::IdlAccount {
                        discriminant: <Self as #prelude::ProgramAccount>::discriminant_bytes(),
                        type_id,
                        seeds: #seeds,
                    };
                    let namespace = idl_definition.add_account(idl_account, <Self::AssociatedProgram as #prelude::ProgramToIdl>::crate_metadata().name)?;
                    Ok(#prelude::IdlAccountId {
                        namespace,
                        source,
                    })
                }
            }
        });

        quote! {
            #type_to_idl_impl
            #account_to_idl_impl
        }
    });

    quote! {
        #owner_program_impl
        #program_account_impl
        #has_seeds_impl
        #idl_impl
    }
}
