use crate::account_set::generics::AccountSetGenerics;
use crate::account_set::{AccountSetStructArgs, StrippedDeriveInput};
use crate::util;
use crate::util::{BetterGenerics, Paths};
use easy_proc::{find_attrs, ArgumentList};
use proc_macro2::{Span, TokenStream};
use proc_macro_error::abort;
use quote::{format_ident, quote};
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use syn::{DataStruct, Expr, LitStr, Type, WhereClause};

#[derive(ArgumentList)]
struct IdlStructArgs {
    id: Option<LitStr>,
    arg: Option<Type>,
    generics: Option<BetterGenerics>,
}

#[derive(ArgumentList)]
struct IdlFieldArgs {
    id: Option<LitStr>,
    arg: Expr,
}

pub(super) fn idls(
    paths: &Paths,
    input: &StrippedDeriveInput,
    account_set_struct_args: &AccountSetStructArgs,
    account_set_generics: &AccountSetGenerics,
    data_struct: &DataStruct,
    field_name: &[TokenStream],
    field_type: &[&Type],
) -> Vec<TokenStream> {
    let ident = &input.ident;
    let AccountSetGenerics {
        main_generics,
        other_generics,
        info_lifetime,
        ..
    } = account_set_generics;
    let Paths {
        result,
        idl_ident,
        account_set_to_idl,
        idl_definition,
        idl_account_set_def,
        idl_account_set,
        idl_account_set_struct_field,
        account_set_id,
        ..
    } = paths;

    let mut idl_ids = HashMap::new();
    for idl_struct_args in find_attrs(&input.attrs, idl_ident).map(IdlStructArgs::parse_arguments) {
        match idl_ids.entry(idl_struct_args.id.as_ref().map(LitStr::value)) {
            Entry::Vacant(entry) => {
                entry.insert(idl_struct_args);
            }
            Entry::Occupied(entry) => {
                abort!(
                    entry.get().id,
                    "Duplicate idl id `{:?}`",
                    entry.get().id.as_ref().map(LitStr::value)
                );
            }
        }
    }
    if !account_set_struct_args.skip_default_idl {
        idl_ids.entry(None).or_insert_with(|| IdlStructArgs {
            id: None,
            arg: None,
            generics: None,
        });
    }

    let field_idls = data_struct
        .fields
        .iter()
        .map(|f| {
            find_attrs(&f.attrs, idl_ident)
                .map(IdlFieldArgs::parse_arguments)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    for field_idl in &field_idls {
        let mut field_ids = HashSet::new();
        for idl_field_arg in field_idl {
            if !idl_ids.contains_key(&idl_field_arg.id.as_ref().map(LitStr::value)) {
                abort!(
                    idl_field_arg.id,
                    "idl id `{:?}` not found",
                    idl_field_arg.id.as_ref().map(LitStr::value)
                );
            }
            if !field_ids.insert(idl_field_arg.id.as_ref().map(LitStr::value)) {
                abort!(
                    idl_field_arg.id,
                    "idl id `{:?}` duplicated",
                    idl_field_arg.id.as_ref().map(LitStr::value)
                );
            }
        }
    }

    let struct_docs = LitStr::new(&util::get_docs(&input.attrs), Span::call_site());
    let ident_str = LitStr::new(&ident.to_string(), Span::call_site());
    let field_docs: Vec<LitStr> = data_struct
        .fields
        .iter()
        .map(|field| LitStr::new(&util::get_docs(&field.attrs), Span::call_site()))
        .collect();
    let field_str = field_name
        .iter()
        .map(|field_name| LitStr::new(&field_name.to_string(), Span::call_site()))
        .collect::<Vec<_>>();

    idl_ids.into_iter().map(|(id, idl_struct_args)|{
        let idl_type: Type = idl_struct_args.arg.unwrap_or_else(|| syn::parse_quote!(()));
        let extra_where_clause: Option<WhereClause> = idl_struct_args.generics.and_then(|generics| generics.where_clause.clone());
        let idl_args: Vec<Expr> = field_idls.iter().map(|f| {
            f.iter().find(|f| f.id.as_ref().map(LitStr::value) == id).map(|f| f.arg.clone()).unwrap_or_else(|| syn::parse_quote!(()))
        }).collect();

        let mut generics = other_generics.clone();
        if let Some(extra_where_clause) = extra_where_clause {
            generics
                .make_where_clause()
                .predicates
                .extend(extra_where_clause.predicates);
        }
        let (_, ty_generics, _) = main_generics.split_for_impl();
        let (impl_generics, _, where_clause) = generics.split_for_impl();
        let field_name = field_name.iter().map(|field_name| format_ident!("__{}", field_name.to_string())).collect::<Vec<_>>();
        quote! {
                #[automatically_derived]
                impl #impl_generics #account_set_to_idl<#info_lifetime, #idl_type> for #ident #ty_generics #where_clause {
                    fn account_set_to_idl(
                        idl_definition: &mut #idl_definition,
                        arg: #idl_type,
                    ) -> #result<#idl_account_set_def> {
                        #(let #field_name = <#field_type as #account_set_to_idl<#info_lifetime, _>>::account_set_to_idl(idl_definition, #idl_args)?;)*
                        idl_definition.account_sets.insert(
                            #ident_str.to_string(),
                            #idl_account_set {
                                name: #ident_str.to_string(),
                                description: #struct_docs.to_string(),
                                type_generics: vec![],
                                account_generics: vec![],
                                def: #idl_account_set_def::Struct(vec![#(
                                    #idl_account_set_struct_field {
                                        name: #field_str.to_string(),
                                        description: #field_docs.to_string(),
                                        path: #field_str.to_string(),
                                        account_set: #field_name,
                                        extension_fields: Default::default(),
                                    },
                                )*]),
                                extension_fields: Default::default(),
                            },
                        );
                        Ok(#idl_account_set_def::AccountSet(#account_set_id {
                            namespace: None,
                            account_set_id: #ident_str.to_string(),
                            provided_type_generics: vec![],
                            provided_account_generics: vec![],
                            extension_fields: Default::default(),
                        }))
                    }
                }
            }
    }).collect()
}
