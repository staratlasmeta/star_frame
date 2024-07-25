use heck::ToSnakeCase;
use itertools::Itertools;
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::*;
use syn::{parse_quote, Field, Fields, FieldsUnnamed, ItemEnum, Lifetime, Type};

use crate::hash::{hash_tts, sighash, SIGHASH_GLOBAL_NAMESPACE};
use crate::util::{verify_repr, Paths};

pub fn instruction_set_impl(item: ItemEnum, args: TokenStream) -> TokenStream {
    let vis = &item.vis;
    let ident = &item.ident;
    let attrs = &item.attrs;
    let a_lifetime: Lifetime = parse_quote! { '__a };

    let valid_reprs: Punctuated<Type, Comma> =
        parse_quote! { u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize };

    let (discriminant_type, use_repr) = if args.is_empty() {
        (parse_quote! { [u8; 8] }, false)
    } else {
        let Ok(repr_ident) = syn::parse2::<Type>(args.clone()) else {
            abort!(args, "Invalid repr type")
        };

        if !valid_reprs.iter().contains(&repr_ident) {
            abort!(repr_ident, "Invalid repr type")
        }
        (repr_ident, true)
    };

    let Paths {
        account_info,
        advance_array,
        anyhow_macro,
        instruction,
        instruction_set,
        pubkey,
        result,
        sys_calls,
        macro_prelude: prelude,
        ..
    } = Paths::default();

    if !item.generics.params.is_empty() {
        abort!(item.generics, "Generics are unsupported");
    }
    let reprs = verify_repr(attrs, [], true, false);
    // todo: potentially allow reprs in the future. Shouldn't really matter with the InstructionDiscriminant trait anymore
    if !reprs.is_empty() {
        abort!(reprs, "Enum reprs are unsupported. Use the `#[star_frame_instruction_set(<repr>)]` syntax instead for repr discriminants.");
    }
    let forced_repr = if use_repr {
        quote! { #[repr(#discriminant_type)] }
    } else if item.variants.len() > u8::MAX as usize {
        quote! { #[repr(u16)] }
    } else {
        quote! { #[repr(u8)] }
    };

    // let repr: Type = if use_repr {
    //     forced_repr = quote! { #[repr(u8)] };
    //     parse_quote! { u8 }
    // } else {
    //     forced_repr = quote! {};
    //     let repr = &reprs[0];
    //     if repr == "C" {
    //         abort!(repr, "#[repr(C)] is unsupported");
    //     }
    //     parse_quote! { #repr }
    // };

    let mut variant_discriminants = Vec::new();
    let mut variant_tys = Vec::new();
    let mut variants = Vec::new();
    let mut last_discriminant = None;
    for variant in &item.variants {
        let variant_ident = &variant.ident;
        let variant_attrs = &variant.attrs;
        let discriminant = variant
            .discriminant
            .as_ref()
            .map(|(_, expr)| expr.clone())
            .unwrap_or_else(|| match last_discriminant {
                Some(last_discriminant) => parse_quote! { (#last_discriminant) + 1 },
                None => parse_quote! { 0 },
            });
        last_discriminant = Some(discriminant.clone());

        match &variant.fields {
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                if unnamed.len() != 1 {
                    abort!(unnamed, "Only a single unnamed field is supported.");
                }
                let Field {
                    attrs: ty_attrs,
                    ty,
                    ..
                } = &unnamed[0];
                variant_discriminants.push(discriminant);
                variants.push(quote! {
                    #(#variant_attrs)*
                    #variant_ident(#(#ty_attrs)* <#ty as #instruction>::SelfData<#a_lifetime>)
                });
                variant_tys.push(ty);
            }
            Fields::Unit | Fields::Named(_) => {
                abort!(variant.fields, "Only a single unnamed field is supported.")
            }
        }
    }

    let get_discriminant = if use_repr {
        quote! {
            let discriminant = #discriminant_type::from_le_bytes(*#advance_array::try_advance_array(&mut ix_bytes)?);
        }
    } else {
        quote! {
            let discriminant = *#advance_array::try_advance_array(&mut ix_bytes)?;
        }
    };

    let ix_disc_values = if use_repr {
        variant_discriminants.clone()
    } else {
        item.variants
            .iter()
            .map(|v| {
                let method_name = v.ident.to_string().to_snake_case();
                parse2(hash_tts(&sighash(SIGHASH_GLOBAL_NAMESPACE, &method_name)))
                    .expect("Hash should be valid expression")
            })
            .collect()
    };

    quote! {
        #(#attrs)*
        #forced_repr
        #vis enum #ident<#a_lifetime> {
            #(#variants = #variant_discriminants,)*
        }

        #[automatically_derived]
        impl<#a_lifetime> #instruction_set for #ident<#a_lifetime> {
            type Discriminant = #discriminant_type;

            fn handle_ix(
                program_id: &#pubkey,
                accounts: &[#account_info],
                mut ix_bytes: &[u8],
                sys_calls: &mut impl #sys_calls,
            ) -> #result<()> {
                #get_discriminant
                #[deny(unreachable_patterns)]
                match discriminant {
                    #(
                        <#variant_tys as #prelude::InstructionDiscriminant<#ident<#a_lifetime>>>::DISCRIMINANT => {
                            let data = <#variant_tys as #instruction>::data_from_bytes(&mut ix_bytes)?;
                            <#variant_tys as #instruction>::run_ix_from_raw(program_id, accounts, &data, sys_calls)
                        }
                    )*
                    x => Err(#anyhow_macro!("Invalid ix discriminant: {:?}", x)),
                }
            }
        }

        #(
            #[automatically_derived]
            impl #prelude::InstructionDiscriminant<#ident <'_>> for #variant_tys {
                const DISCRIMINANT: #discriminant_type = #ix_disc_values;
            }
        )*
    }
}
