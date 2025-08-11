use easy_proc::{find_attr, ArgumentList};
use heck::ToSnakeCase;
use itertools::Itertools;
use proc_macro2::TokenStream;
use proc_macro_error2::{abort, abort_call_site};
use quote::quote;
use syn::{parse_quote, Expr, Fields, FieldsUnnamed, ItemEnum, Type};

use crate::hash::SIGHASH_GLOBAL_NAMESPACE;
use crate::util::{enum_discriminants, get_repr, ignore_cfg_module, Paths};

#[derive(Debug, ArgumentList, Clone, Default)]
pub struct InstructionSetStructArgs {
    #[argument(presence)]
    pub skip_idl: bool,
    #[argument(presence)]
    pub use_repr: bool,
}

#[derive(Debug, ArgumentList, Clone, Default)]
pub struct InstructionSetFieldArgs {
    pub idl_arg: Option<Expr>,
    pub idl_arg_ty: Option<Type>,
}

pub fn instruction_set_impl(item: ItemEnum) -> TokenStream {
    Paths!(
        account_info,
        bytemuck,
        instruction,
        pubkey,
        result,
        prelude,
        instruction_set_args_ident,
    );
    let (impl_generics, ty_generics, where_clause) = &item.generics.split_for_impl();

    let ident = &item.ident;

    let args = find_attr(&item.attrs, &instruction_set_args_ident)
        .map(InstructionSetStructArgs::parse_arguments)
        .unwrap_or_default();

    let discriminant_type: Type = if args.use_repr {
        let repr = get_repr(&item.attrs);
        repr.repr.as_integer().map_or_else(
            || abort_call_site!("Invalid repr attribute for ix_set. Must use integer repr with `use_repr` enabled"),
            |ty| parse_quote! { #ty },
        )
    } else {
        parse_quote!([u8; 8])
    };

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

    let ix_disc_values = if args.use_repr {
        enum_discriminants(item.variants.iter()).collect_vec()
    } else {
        item.variants
            .iter()
            .map(|v| {
                let method_name = v.ident.to_string().to_snake_case();
                parse_quote!(#prelude::sighash!(#SIGHASH_GLOBAL_NAMESPACE, #method_name))
            })
            .collect()
    };

    let idl_impl = (!args.skip_idl).then( || {
        let (idl_args, idl_arg_tys) = item
            .variants
            .iter()
            .map(|v| {
                let args = find_attr(&v.attrs, &instruction_set_args_ident)
                    .map(InstructionSetFieldArgs::parse_arguments)
                    .unwrap_or_default();
                let idl_arg = args.idl_arg.unwrap_or_else(|| parse_quote!(()));
                let idl_arg_ty = args.idl_arg_ty.unwrap_or_else(|| parse_quote!(_));
                (idl_arg, idl_arg_ty)
            })
            .unzip::<_, _, Vec<_>, Vec<_>>();

        ignore_cfg_module(ident, "_instruction_set_to_idl", quote! {
            #[cfg(all(feature = "idl", not(target_os = "solana")))]
            #[automatically_derived]
            impl #impl_generics #prelude::InstructionSetToIdl for #ident #ty_generics #where_clause {
                #[allow(clippy::let_unit_value)]
                fn instruction_set_to_idl(
                    idl_definition: &mut #prelude::IdlDefinition,
                ) -> #result<()> {
                    #({
                        let definition =
                            <#variant_tys as #prelude::InstructionToIdl<#idl_arg_tys>>::instruction_to_idl(idl_definition, #idl_args)?;
                        let discriminant =
                            <#variant_tys as #prelude::InstructionDiscriminant<Self>>::discriminant_bytes();
                        idl_definition.add_instruction(definition, discriminant)?;
                    })*
                    Ok(())
                }
            }
        })
    });
    let ix_message = item
        .variants
        .iter()
        .map(|v| format!("Instruction: {}", v.ident))
        .collect_vec();

    let handle_ix_body = if variant_tys.is_empty() {
        quote! {
            #prelude::bail!("No instructions in this instruction set")
        }
    } else {
        quote! {
            let maybe_discriminant_bytes =
                #prelude::Advance::try_advance(&mut ix_bytes, ::core::mem::size_of::<#discriminant_type>());
            let discriminant_bytes = #prelude::anyhow::Context::context(maybe_discriminant_bytes, "Failed to read instruction discriminant bytes")?;
            let discriminant = *#bytemuck::try_from_bytes(discriminant_bytes)?;
            #[deny(unreachable_patterns)]
            match discriminant {
                #(
                    <#variant_tys as #prelude::InstructionDiscriminant<#ident #ty_generics>>::DISCRIMINANT => {
                        #prelude::msg!(#ix_message);
                        let mut data = #prelude::anyhow::Context::context(<#variant_tys as #instruction>::data_from_bytes(&mut ix_bytes), "Failed to read instruction data")?;
                        #prelude::anyhow::Context::context(<#variant_tys as #instruction>::run_ix_from_raw(accounts, &mut data, ctx), "Failed to run instruction")
                    }
                )*
                x => #prelude::bail!("Invalid ix discriminant: {:?}", x),
            }
        }
    };

    // todo: better error messages for getting the discriminant and invalid discriminants
    quote! {
        #[automatically_derived]
        impl #impl_generics #prelude::InstructionSet for #ident #ty_generics #where_clause {
            type Discriminant = #discriminant_type;

            fn handle_ix(
                program_id: &#pubkey,
                accounts: &[#account_info],
                mut ix_bytes: &[u8],
                ctx: &mut #prelude::Context,
            ) -> #result<()> {
                #handle_ix_body
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
