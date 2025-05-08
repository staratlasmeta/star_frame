use crate::account_set::generics::AccountSetGenerics;
use crate::account_set::struct_impl::StepInput;
use crate::util;
use crate::util::{new_generic, BetterGenerics, Paths};
use easy_proc::{find_attrs, ArgumentList};
use itertools::Itertools;
use proc_macro2::{Span, TokenStream};
use proc_macro_error2::abort;
use quote::quote;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use syn::spanned::Spanned;
use syn::{parse_quote, Expr, LitStr, Type};

#[derive(ArgumentList)]
struct IdlStructArgs {
    id: Option<LitStr>,
    arg: Option<Type>,
    generics: Option<BetterGenerics>,
}

#[derive(ArgumentList)]
struct IdlFieldArgs {
    id: Option<LitStr>,
    arg: Option<Expr>,
    address: Option<Expr>,
}

pub(super) fn idls(
    StepInput {
        paths,
        input,
        account_set_struct_args,
        account_set_generics,
        single_set_field,
        field_name,
        fields,
        field_type,
    }: StepInput,
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
        prelude,
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

    let field_idls = fields
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

    let struct_docs = &util::get_docs(&input.attrs);
    let ident_str = LitStr::new(&ident.to_string(), Span::call_site());
    let field_docs: Vec<Expr> = fields
        .iter()
        .map(|field| util::get_docs(&field.attrs))
        .collect();
    let is_tuple_struct = fields.first().is_some_and(|f| f.ident.is_none());
    let field_path: Vec<Expr> = field_name
        .iter()
        .map(|field_name| {
            if is_tuple_struct {
                parse_quote!(None)
            } else {
                let name = LitStr::new(&field_name.to_string(), field_name.span());
                parse_quote!(Some(#name.to_string()))
            }
        })
        .collect::<Vec<_>>();

    idl_ids
        .into_iter()
        .map(|(id, idl_struct_args)| {
            let (_, ty_generics, _) = main_generics.split_for_impl();
            let mut generics = other_generics.clone();
            let mut idl_type: Type = syn::parse_quote!(());
            let mut default_idl_arg: Expr = syn::parse_quote!(());
            if let Some(extra_generics) = idl_struct_args.generics.map(|g| g.into_inner()) {
                generics.params.extend(extra_generics.params);
                if let Some(extra_where_clause) = extra_generics.where_clause {
                    generics
                        .make_where_clause()
                        .predicates
                        .extend(extra_where_clause.predicates);
                }
            } else if let Some(single_set_field) = single_set_field {
                let generic_arg = new_generic(main_generics, None);
                default_idl_arg = syn::parse_quote!(arg);
                idl_type = syn::parse_quote!(#generic_arg);
                generics.params.push(syn::parse_quote!(#generic_arg));
                let single_ty = &single_set_field.ty;
                generics.make_where_clause().predicates.push(syn::parse_quote!(#single_ty: #prelude::AccountSetToIdl<#info_lifetime, #generic_arg>));
            }
            let idl_type: Type = idl_struct_args.arg.unwrap_or(idl_type);
            let (idl_args, idl_addresses): (Vec<Option<Expr>>, Vec<Option<Expr>>) = field_idls
                .iter()
                .map(|f| {
                    f.iter()
                        .find(|f| f.id.as_ref().map(LitStr::value) == id)
                        .map(|f| (f.arg.clone(), f.address.clone()))
                        .unwrap_or_default()
                })
                .collect();
            let idl_args: Vec<Expr> = idl_args.into_iter().map(|a| a.unwrap_or(default_idl_arg.clone())).collect();
            let (impl_generics, _, where_clause) = generics.split_for_impl();



            let mut single_vec = Vec::with_capacity(1);
            let relevant_field_types = if let Some(single) = single_set_field {
                single_vec.push(&single.ty);
                &single_vec
            } else {
                field_type
            };

            let account_set_defs = relevant_field_types.iter().zip(idl_args).zip(idl_addresses).map(|((ty, idl_arg), idl_address)| {
                let expression = quote! {
                    <#ty as #prelude::AccountSetToIdl<#info_lifetime, _>>::account_set_to_idl(idl_definition, #idl_arg)
                };
                if let Some(address) = idl_address {
                    quote! (#expression?.with_single_address(#address))
                } else {
                    expression
                }
            }).collect_vec();

            let inner = if account_set_defs.len() == 1 {
                account_set_defs[0].clone()
            } else {
                quote! {
                    let source = #prelude::item_source::<Self>();
                    let account_set_def = #prelude::IdlAccountSetDef::Struct(vec![
                        #(
                            #prelude::IdlAccountSetStructField {
                                path: #field_path,
                                description: #field_docs,
                                account_set_def: #account_set_defs?,
                            }
                        ),*
                    ]);
                    let account_set = #prelude::IdlAccountSet {
                        info: #prelude::ItemInfo {
                            name: #ident_str.to_string(),
                            description: #struct_docs,
                            source: source.clone(),
                        },
                        account_set_def,
                        type_generics: vec![],
                        account_generics: vec![],
                    };
                    idl_definition.add_account_set(account_set);
                    Ok(#prelude::IdlAccountSetDef::Defined(#prelude::IdlAccountSetId {
                        source,
                        provided_type_generics: vec![],
                        provided_account_generics: vec![],
                    }))
                }
            };

            quote! {
                #[cfg(all(feature = "idl", not(target_os = "solana")))]
                #[automatically_derived]
                impl #impl_generics #prelude::AccountSetToIdl<#info_lifetime, #idl_type> for #ident #ty_generics #where_clause {
                    fn account_set_to_idl(
                        idl_definition: &mut #prelude::IdlDefinition,
                        arg: #idl_type,
                    ) -> #result<#prelude::IdlAccountSetDef> {
                        #inner
                    }
                }
            }
        })
        .collect()
}
