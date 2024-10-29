use easy_proc::{find_attr, ArgumentList};
use heck::ToSnakeCase;
use itertools::Itertools;
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use syn::*;
use syn::{parse_quote, Fields, FieldsUnnamed, ItemEnum, Lifetime, Type};

use crate::hash::SIGHASH_GLOBAL_NAMESPACE;
use crate::util::Paths;

#[derive(Debug, ArgumentList, Clone, Default)]
pub struct InstructionSetStructArgs {
    #[argument(presence)]
    pub skip_idl: bool,
}

pub fn instruction_set_impl(item: ItemEnum) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = &item.generics.split_for_impl();
    let ident = &item.ident;
    let info_lifetime: Lifetime = parse_quote! { '__info };

    // todo: allow for repr discriminants
    let discriminant_type: Type = parse_quote!([u8; 8]);

    let Paths {
        account_info,
        advance,
        bytemuck,
        anyhow_macro,
        instruction,
        pubkey,
        result,
        syscalls,
        macro_prelude: prelude,
        instruction_set_args_ident,
        ..
    } = Paths::default();

    let args = find_attr(&item.attrs, &instruction_set_args_ident)
        .map(InstructionSetStructArgs::parse_arguments)
        .unwrap_or_default();

    let variant_tys = item
        .variants
        .iter()
        .map(|v| {
            const UNNAMED_ERROR: &str = "Each variant must have a single unnamed field";
            let unnamed_fields = match &v.fields {
                Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => unnamed,
                _ => abort!(v.fields, UNNAMED_ERROR),
            };
            if unnamed_fields.len() != 1 {
                abort!(unnamed_fields, UNNAMED_ERROR);
            }
            &unnamed_fields[0].ty
        })
        .collect_vec();

    let ix_disc_values: Vec<Expr> = item
        .variants
        .iter()
        .map(|v| {
            let method_name = v.ident.to_string().to_snake_case();
            parse_quote!(#prelude::sighash!(#SIGHASH_GLOBAL_NAMESPACE, #method_name))
        })
        .collect();

    let idl_impl = (!args.skip_idl && cfg!(feature = "idl")).then(|| {
        let mut generics = item.generics.clone();
        let where_clause = generics.make_where_clause();
        variant_tys.iter().for_each(|ty| {
            where_clause.predicates.push(parse_quote! {
                // todo: support passing args to instruction_to_idl per variant
                #ty: #prelude::InstructionToIdl<()>
            });
        });

        quote! {
            #[automatically_derived]
            impl #impl_generics #prelude::InstructionSetToIdl for #ident #ty_generics #where_clause {
                #[allow(clippy::let_unit_value)]
                fn instruction_set_to_idl(
                    idl_definition: &mut #prelude::IdlDefinition,
                ) -> #result<()> {
                    #({
                        // todo: support passing args to instruction_to_idl per variant
                        type __ArgTy = ();
                        let arg: __ArgTy = ();
                        let definition = <#variant_tys as #prelude::InstructionToIdl<_>>::instruction_to_idl(idl_definition, arg)?;
                        let discriminant =
                            <#variant_tys as #prelude::InstructionDiscriminant<Self>>::discriminant_bytes();
                        idl_definition.add_instruction(definition, discriminant)?;
                    })*
                    Ok(())
                }
            }
        }
    });

    // todo: better error messages for getting the discriminant and invalid discriminants
    quote! {
        #[automatically_derived]
        impl #impl_generics #prelude::InstructionSet for #ident #ty_generics #where_clause {
            type Discriminant = #discriminant_type;

            fn handle_ix<#info_lifetime>(
                program_id: &#pubkey,
                accounts: &[#account_info<#info_lifetime>],
                mut ix_bytes: &[u8],
                syscalls: &mut impl #syscalls<#info_lifetime>,
            ) -> #result<()> {
                let discriminant_bytes =
                    #advance::try_advance(&mut ix_bytes, ::core::mem::size_of::<#discriminant_type>())?;
                let discriminant = *#bytemuck::try_from_bytes(discriminant_bytes)?;
                #[deny(unreachable_patterns)]
                match discriminant {
                    #(
                        <#variant_tys as #prelude::InstructionDiscriminant<#ident #ty_generics>>::DISCRIMINANT => {
                            let data = <#variant_tys as #instruction>::data_from_bytes(&mut ix_bytes)?;
                            <#variant_tys as #instruction>::run_ix_from_raw(accounts, &data, syscalls)
                        }
                    )*
                    x => Err(#anyhow_macro!("Invalid ix discriminant: {:?}", x)),
                }
            }
        }

        #(
            #[automatically_derived]
            impl #impl_generics #prelude::InstructionDiscriminant<#ident #ty_generics> for #variant_tys #where_clause {
                const DISCRIMINANT: #discriminant_type = #ix_disc_values;
            }
        )*

        #idl_impl
    }
}
