use crate::account_set::generics::AccountSetGenerics;
use crate::account_set::{AccountSetStructArgs, StrippedDeriveInput};
use crate::util::{BetterGenerics, Paths};
use easy_proc::{find_attrs, ArgumentList};
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use syn::{DataStruct, Expr, Fields, LitStr, Type};

#[derive(ArgumentList)]
struct DecodeStructArgs {
    id: Option<LitStr>,
    arg: Option<Type>,
    generics: Option<BetterGenerics>,
}

#[derive(ArgumentList)]
struct DecodeFieldArgs {
    id: Option<LitStr>,
    arg: Expr,
}

#[derive(Debug)]
pub enum DecodeFieldTy<'a> {
    Type(&'a Type),
    Default(TokenStream),
}

pub(super) fn decodes(
    paths: &Paths,
    input: &StrippedDeriveInput,
    account_set_struct_args: &AccountSetStructArgs,
    account_set_generics: &AccountSetGenerics,
    data_struct: &DataStruct,
    field_name: &[TokenStream],
    decode_field_ty: &[DecodeFieldTy],
) -> Vec<TokenStream> {
    let ident = &input.ident;
    let AccountSetGenerics {
        main_generics,
        decode_generics,
        info_lifetime,
        decode_lifetime,
        ..
    } = account_set_generics;
    let Paths {
        account_info,
        result,
        account_set_decode,
        sys_call_invoke,
        decode_ident,
        ..
    } = paths;
    let init = |inits: &mut dyn Iterator<Item = TokenStream>| match &data_struct.fields {
        Fields::Named(_) => quote! {
            #ident {
                #(#field_name: #inits,)*
            }
        },
        Fields::Unnamed(_) => quote! {
            #ident (
                #(#inits,)*
            )
        },
        Fields::Unit => quote! {
            #ident
        },
    };

    let mut decode_ids = HashMap::new();
    for decode_struct_arg in
        find_attrs(&input.attrs, decode_ident).map(DecodeStructArgs::parse_arguments)
    {
        match decode_ids.entry(decode_struct_arg.id.as_ref().map(LitStr::value)) {
            Entry::Vacant(entry) => {
                entry.insert(decode_struct_arg);
            }
            Entry::Occupied(entry) => {
                abort!(
                    entry.get().id,
                    "Duplicate decode id `{:?}`",
                    entry.get().id.as_ref().map(LitStr::value)
                );
            }
        }
    }
    if !account_set_struct_args.skip_default_decode {
        decode_ids.entry(None).or_insert_with(|| DecodeStructArgs {
            id: None,
            arg: None,
            generics: None,
        });
    }

    let field_decodes = data_struct
        .fields
        .iter()
        .map(|f| {
            find_attrs(&f.attrs, decode_ident)
                .map(DecodeFieldArgs::parse_arguments)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    for field_decode in &field_decodes {
        let mut field_ids = HashSet::new();
        for decode_field_arg in field_decode {
            if !decode_ids.contains_key(&decode_field_arg.id.as_ref().map(LitStr::value)) {
                abort!(
                    decode_field_arg.id,
                    "Decode id `{:?}` not found",
                    decode_field_arg.id.as_ref().map(LitStr::value)
                );
            }
            if !field_ids.insert(decode_field_arg.id.as_ref().map(LitStr::value)) {
                abort!(
                    decode_field_arg.id,
                    "Duplicate decode id `{:?}`",
                    decode_field_arg.id.as_ref().map(LitStr::value)
                );
            }
        }
    }

    decode_ids.into_iter().map(|(id, decode_struct_args)|{
        let decode_type: Type = decode_struct_args.arg.unwrap_or_else(|| syn::parse_quote!(()));
        let decode_args: Vec<Expr> = field_decodes.iter().map(|f| {
            f.iter().find(|f| f.id.as_ref().map(LitStr::value) == id).map(|f| f.arg.clone()).unwrap_or_else(|| syn::parse_quote!(()))
        }).collect();

        let (_, ty_generics, _) = main_generics.split_for_impl();
        let mut generics = decode_generics.clone();
        if let Some(extra_generics) = decode_struct_args.generics.map(|g| g.into_inner()) {
            generics.params.extend(extra_generics.params);
            if let Some(extra_where_clause) = extra_generics.where_clause {
                generics
                    .make_where_clause()
                    .predicates
                    .extend(extra_where_clause.predicates);
            }
        }
        let (impl_generics, _, where_clause) = generics.split_for_impl();

        let decode_inner = init(&mut decode_field_ty.iter().zip(&decode_args).map(|(field_ty, decode_args)| {
            match &field_ty {
                DecodeFieldTy::Type(field_type) => quote! {
                        <#field_type as #account_set_decode<#decode_lifetime, #info_lifetime, _>>::decode_accounts(accounts, #decode_args, sys_calls)?
                    },
                DecodeFieldTy::Default(default) => quote!(#default)
            }
        }));

        quote!{
            #[automatically_derived]
            impl #impl_generics #account_set_decode<#decode_lifetime, #info_lifetime, #decode_type> for #ident #ty_generics #where_clause {
                fn decode_accounts(
                    accounts: &mut &#decode_lifetime [#account_info<#info_lifetime>],
                    arg: #decode_type,
                    sys_calls: &mut impl #sys_call_invoke,
                ) -> #result<Self> {
                    Ok(#decode_inner)
                }
            }
        }
    }).collect()
}
