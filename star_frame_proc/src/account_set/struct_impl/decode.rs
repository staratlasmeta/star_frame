use crate::{
    account_set::{generics::AccountSetGenerics, struct_impl::StepInput},
    util::{new_generic, BetterGenerics, Paths},
};
use easy_proc::{find_attrs, ArgumentList};
use itertools::Itertools;
use proc_macro2::TokenStream;
use proc_macro_error2::abort;
use quote::quote;
use std::collections::{hash_map::Entry, HashMap, HashSet};
use syn::{DataStruct, Expr, Fields, LitStr, Type};

#[derive(ArgumentList, Default)]
struct DecodeStructArgs {
    id: Option<LitStr>,
    arg: Option<Type>,
    generics: Option<BetterGenerics>,
    #[argument(presence)]
    inline_always: bool,
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
    StepInput {
        paths,
        input,
        account_set_struct_args,
        account_set_generics,
        single_set_field,
        ..
    }: StepInput,
    data_struct: &DataStruct,
    all_field_name: &[TokenStream],
    decode_field_ty: &[DecodeFieldTy],
) -> Vec<TokenStream> {
    let ident = &input.ident;
    let AccountSetGenerics {
        main_generics,
        decode_generics,
        decode_lifetime,
        ..
    } = account_set_generics;
    let Paths {
        account_info,
        result,
        account_set_decode,
        prelude,
        decode_ident,
        ..
    } = paths;
    let init = |inits: &mut dyn Iterator<Item = TokenStream>| match &data_struct.fields {
        Fields::Named(_) => quote! {
            #ident {
                #(#all_field_name: #inits,)*
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
        decode_ids.entry(None).or_insert_with(Default::default);
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

    decode_ids.into_iter().map(|(id, decode_struct_args)| {
        let (_, ty_generics, _) = main_generics.split_for_impl();
        let mut generics = decode_generics.clone();
        let mut default_decode_arg: Expr = syn::parse_quote!(());
        let mut decode_type: Type = syn::parse_quote!(());
        if let Some(extra_generics) = decode_struct_args.generics.map(|g| g.into_inner()) {
            generics.params.extend(extra_generics.params);
            if let Some(extra_where_clause) = extra_generics.where_clause {
                generics
                    .make_where_clause()
                    .predicates
                    .extend(extra_where_clause.predicates);
            }
        } else if let Some(single_set_field) = single_set_field {
            let generic_arg = new_generic(main_generics, None);
            default_decode_arg = syn::parse_quote!(arg);
            decode_type = syn::parse_quote!(#generic_arg);
            generics.params.push(syn::parse_quote!(#generic_arg));
            let single_ty = &single_set_field.ty;
            generics.make_where_clause().predicates.push(syn::parse_quote!(#single_ty: #account_set_decode<#decode_lifetime, #generic_arg> + #prelude::SingleAccountSet));
        }
        let decode_type = decode_struct_args.arg.unwrap_or(decode_type);
        let decode_args: Vec<Expr> = field_decodes
            .iter()
            .map(|f| {
                f.iter()
                    .find(|f| f.id.as_ref().map(LitStr::value) == id)
                    .map(|f| f.arg.clone())
                    .unwrap_or_else(|| default_decode_arg.clone())
            }).collect();

        let (impl_generics, _, where_clause) = generics.split_for_impl();

        let decode_inner = init(&mut decode_field_ty.iter().zip_eq(all_field_name).zip_eq(&decode_args).map(|((field_ty, field_name), decode_args)| {
            
            match &field_ty {
                DecodeFieldTy::Type(field_type) => {
                    let decode = quote! {
                        <#field_type as #account_set_decode<#decode_lifetime, _>>::decode_accounts(accounts, #decode_args, ctx)
                    };
                    if single_set_field.is_some() {
                        quote! { #decode? }
                    } else {
                        quote! {
                            #prelude::ErrorInfo::account_path(
                                #decode,
                                ::std::stringify!(#field_name),
                            )?
                        }
                    }
                },
                DecodeFieldTy::Default(default) => quote!(#default)
            }
        }));

        let inline_attr = if decode_struct_args.inline_always {
            quote!(#[inline(always)])
        } else {
            quote!(#[inline])
        };

        quote! {
            #[automatically_derived]
            impl #impl_generics #account_set_decode<#decode_lifetime, #decode_type> for #ident #ty_generics #where_clause {
                #inline_attr
                fn decode_accounts(
                    accounts: &mut &#decode_lifetime [#account_info],
                    arg: #decode_type,
                    ctx: &mut #prelude::Context,
                ) -> #result<Self> {
                    Ok(#decode_inner)
                }
            }
        }
    }).collect()
}
