use crate::{
    util,
    util::{ensure_data_struct, ignore_cfg_module, reject_generics, Paths},
};
use easy_proc::{find_attrs, ArgumentList};
use proc_macro2::TokenStream;
use proc_macro_error2::{abort, abort_call_site};
use quote::{quote, ToTokens};
use syn::{parse_quote, DeriveInput, Expr, ExprLit, Lit, Type};

#[derive(ArgumentList, Default)]
pub struct StarFrameProgramDerive {
    account_discriminant: Option<Type>,
    instruction_set: Option<Type>,
    id: Option<Expr>,
    errors: Option<Type>,
    #[argument(presence)]
    no_entrypoint: bool,
    #[argument(presence)]
    no_setup: bool,
    #[argument(presence)]
    skip_idl: bool,
}

pub(crate) fn program_impl(input: DeriveInput) -> TokenStream {
    Paths!(crate_name, address, prelude, star_frame_program_ident);

    ensure_data_struct(&input, None);
    reject_generics(&input, None);

    let mut derive_input = StarFrameProgramDerive::default();

    for program_derive in find_attrs(&input.attrs, &star_frame_program_ident) {
        let StarFrameProgramDerive {
            account_discriminant,
            instruction_set,
            id: program_id,
            errors,
            no_entrypoint,
            no_setup,
            skip_idl,
        } = StarFrameProgramDerive::parse_arguments(program_derive);

        if let Some(account_discriminant) = account_discriminant {
            let current = derive_input
                .account_discriminant
                .replace(account_discriminant.clone());
            if current.is_some() {
                abort!(
                    account_discriminant,
                    "Duplicate `account_discriminant` argument"
                );
            }
        }

        if let Some(instruction_set) = instruction_set {
            let current = derive_input
                .instruction_set
                .replace(instruction_set.clone());
            if current.is_some() {
                abort!(instruction_set, "Duplicate `instruction_set` argument");
            }
        }

        if let Some(program_id) = program_id {
            let current = derive_input.id.replace(program_id.clone());
            if current.is_some() {
                abort!(program_id, "Duplicate `id` argument");
            }
        }

        if no_entrypoint {
            if derive_input.no_entrypoint {
                abort!(no_entrypoint, "Duplicate `no_entrypoint` argument");
            }
            derive_input.no_entrypoint = true;
        }

        if no_setup {
            if derive_input.no_setup {
                abort!(no_setup, "Duplicate `no_setup` argument");
            }
            derive_input.no_setup = true;
        }

        if skip_idl {
            if derive_input.skip_idl {
                abort!(skip_idl, "Duplicate `skip_idl` argument");
            }
            derive_input.skip_idl = true;
        }

        if let Some(errors) = errors {
            let current = derive_input.errors.replace(errors.clone());
            if current.is_some() {
                abort!(errors, "Duplicate `errors` argument");
            }
        }
    }

    let Some(program_id) = derive_input.id else {
        abort_call_site!("expected an `id` {} argument", star_frame_program_ident);
    };

    let Some(instruction_set_type) = derive_input.instruction_set else {
        abort_call_site!(
            "expected an `instruction_set` {} argument",
            star_frame_program_ident
        );
    };
    let program_id = match program_id {
        Expr::Lit(ExprLit {
            lit: Lit::Str(lit), ..
        }) => quote! {
            #crate_name::address!(#lit)
        },
        e => e.to_token_stream(),
    };

    let ident = &input.ident;
    let StarFrameProgramDerive {
        mut account_discriminant,
        no_entrypoint,
        no_setup,
        skip_idl,
        errors,
        ..
    } = derive_input;

    let errors = errors.unwrap_or_else(|| {
        parse_quote! {()}
    });

    if account_discriminant.is_none() {
        account_discriminant.replace(parse_quote! { [u8; 8] });
    }

    let entrypoint = if no_entrypoint {
        quote! {}
    } else {
        quote! { #crate_name::star_frame_entrypoint!(#ident); }
    };

    let program_setup = if no_setup {
        quote! {}
    } else {
        quote! { #crate_name::program_setup!(#ident); }
    };

    let idl_impl = (!skip_idl).then(|| {
        let docs = util::get_docs(&input.attrs);
        ignore_cfg_module(
            ident,
            "_program_to_idl",
            quote! {
                use #crate_name::alloc::vec;

                #[cfg(all(feature = "idl", not(target_os = "solana")))]
                #[automatically_derived]
                impl #prelude::ProgramToIdl for #ident {
                    type Errors = #errors;
                    fn crate_metadata() -> #prelude::CrateMetadata {
                        #prelude::CrateMetadata {
                            docs: #docs,
                            ..#prelude::crate_metadata!()
                        }
                    }
                }
            },
        )
    });

    quote! {
        #[automatically_derived]
        impl #prelude::StarFrameProgram for #ident {
            type InstructionSet = #instruction_set_type;
            type AccountDiscriminant = #account_discriminant;
            const ID: #address = #program_id;
        }
        #program_setup
        #entrypoint

        #idl_impl
    }
}
