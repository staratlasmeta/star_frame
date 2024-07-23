use easy_proc::{find_attrs, ArgumentList};
use proc_macro2::TokenStream;
use proc_macro_error::{abort, abort_call_site};
use quote::{quote, ToTokens};
use syn::{parse_quote, DeriveInput, Expr, ExprLit, Lit, Type};

use crate::util::Paths;

#[derive(ArgumentList, Default)]
pub struct StarFrameProgramDerive {
    account_discriminant: Option<Type>,
    instruction_set: Option<Type>,
    closed_account_discriminant: Option<Expr>,
    id: Option<Expr>,
    #[argument(presence)]
    no_entrypoint: bool,
}

pub(crate) fn program_impl(input: DeriveInput) -> TokenStream {
    let Paths {
        crate_name,
        pubkey,
        star_frame_program,
        instruction_set,
        star_frame_program_ident,
        ..
    } = Paths::default();

    if !matches!(input.data, syn::Data::Struct(_)) {
        abort!(
            input.ident,
            "StarFrameProgram can only be derived on structs"
        );
    }

    if !input.generics.params.is_empty() || input.generics.where_clause.is_some() {
        abort!(
            input.generics,
            "StarFrameProgram cannot be derived on generic structs"
        );
    }

    let mut derive_input = StarFrameProgramDerive::default();

    for program_derive in find_attrs(&input.attrs, &star_frame_program_ident) {
        let StarFrameProgramDerive {
            account_discriminant,
            instruction_set,
            closed_account_discriminant,
            id: program_id,
            no_entrypoint,
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

        if let Some(closed_account_discriminant) = closed_account_discriminant {
            let current = derive_input
                .closed_account_discriminant
                .replace(closed_account_discriminant.clone());
            if current.is_some() {
                abort!(
                    closed_account_discriminant,
                    "Duplicate `closed_account_discriminant` argument"
                );
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
            #crate_name::pubkey!(#lit)
        },
        e => e.to_token_stream(),
    };

    let ident = &input.ident;
    let StarFrameProgramDerive {
        mut account_discriminant,
        mut closed_account_discriminant,
        no_entrypoint,
        ..
    } = derive_input;

    if account_discriminant.is_none() && closed_account_discriminant.is_none() {
        closed_account_discriminant.replace(parse_quote! { [u8::MAX; 8] });
        account_discriminant.replace(parse_quote! { [u8; 8] });
    }

    const DISCRIMINANT_WARNING: &str =
        "`closed_account_discriminant` argument must be used with `account_discriminant` argument";
    if account_discriminant.is_some() && closed_account_discriminant.is_none() {
        abort!(account_discriminant, DISCRIMINANT_WARNING);
    }

    if closed_account_discriminant.is_some() && account_discriminant.is_none() {
        abort!(closed_account_discriminant, DISCRIMINANT_WARNING);
    }

    let entrypoint = if no_entrypoint {
        quote! {}
    } else {
        quote! { #crate_name::star_frame_entrypoint!(#ident); }
    };

    quote! {
        impl #star_frame_program for #ident {
            type InstructionSet<'a> = #instruction_set_type;
            type InstructionDiscriminant = <Self::InstructionSet<'static> as #instruction_set>::Discriminant;
            type AccountDiscriminant = #account_discriminant;
            const CLOSED_ACCOUNT_DISCRIMINANT: Self::AccountDiscriminant = #closed_account_discriminant;
            const PROGRAM_ID: #pubkey = #program_id;
        }
        pub type StarFrameDeclaredProgram = #ident;

        #[doc = r" The const program ID."]
        pub const ID: #pubkey = <#ident as #star_frame_program>::PROGRAM_ID;

        #[doc = r" Returns `true` if given pubkey is the program ID."]
        pub fn check_id(id: &#pubkey) -> bool { id == &ID }

        #[doc = r" Returns the program ID."]
        pub const fn id() -> #pubkey { ID }

        #[test]
        fn test_id() { assert!(check_id(&id())); }

        #entrypoint
    }
}
