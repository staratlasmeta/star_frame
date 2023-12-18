use crate::account_set::generics::AccountSetGenerics;
use crate::account_set::{AccountSetStructArgs, StrippedDeriveInput};
use crate::util::{BracketedGenerics, Paths};
use easy_proc::{find_attrs, ArgumentList};
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use syn::{DataStruct, Expr, LitStr, Type};

#[derive(ArgumentList)]
struct ValidateStructArgs {
    id: Option<LitStr>,
    arg: Option<Type>,
    generics: Option<BracketedGenerics>,
    extra_validation: Option<Expr>,
}

#[derive(ArgumentList)]
struct ValidateFieldArgs {
    id: Option<LitStr>,
    arg: Expr,
}

pub(super) fn validates(
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
        sys_call_invoke,
        validate_ident,
        account_set_validate,
        ..
    } = paths;

    let mut validate_ids = HashMap::new();
    for validate_struct_args in
        find_attrs(&input.attrs, validate_ident).map(ValidateStructArgs::parse_arguments)
    {
        match validate_ids.entry(validate_struct_args.id.as_ref().map(LitStr::value)) {
            Entry::Vacant(entry) => {
                entry.insert(validate_struct_args);
            }
            Entry::Occupied(entry) => {
                abort!(
                    entry.get().id,
                    "Duplicate validate id `{:?}`",
                    entry.get().id.as_ref().map(LitStr::value)
                );
            }
        }
    }
    if !account_set_struct_args.skip_default_validate {
        validate_ids
            .entry(None)
            .or_insert_with(|| ValidateStructArgs {
                id: None,
                arg: None,
                generics: None,
                extra_validation: None,
            });
    }

    let field_validates = data_struct
        .fields
        .iter()
        .map(|f| {
            find_attrs(&f.attrs, validate_ident)
                .map(ValidateFieldArgs::parse_arguments)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    for field_validate in &field_validates {
        let mut field_ids = HashSet::new();
        for validate_field_arg in field_validate {
            if !validate_ids.contains_key(&validate_field_arg.id.as_ref().map(LitStr::value)) {
                abort!(
                    validate_field_arg.id,
                    "Validate id `{:?}` not found",
                    validate_field_arg.id.as_ref().map(LitStr::value)
                );
            }
            if !field_ids.insert(validate_field_arg.id.as_ref().map(LitStr::value)) {
                abort!(
                    validate_field_arg.id,
                    "Validate decode id `{:?}`",
                    validate_field_arg.id.as_ref().map(LitStr::value)
                );
            }
        }
    }

    validate_ids.into_iter().map(|(id, validate_struct_args)|{
        let validate_type: Type = validate_struct_args.arg.unwrap_or_else(|| syn::parse_quote!(()));
        let validate_args: Vec<Expr> = field_validates.iter().map(|f| {
            f.iter().find(|f| f.id.as_ref().map(LitStr::value) == id).map(|f| f.arg.clone()).unwrap_or_else(|| syn::parse_quote!(()))
        }).collect();

        let (_, ty_generics, _) = main_generics.split_for_impl();
        let mut generics = other_generics.clone();
        if let Some(extra_generics) = validate_struct_args.generics.map(|g| g.into_inner()) {
            generics.params.extend(extra_generics.params);
            if let Some(extra_where_clause) = extra_generics.where_clause {
                generics
                    .make_where_clause()
                    .predicates
                    .extend(extra_where_clause.predicates);
            }
        }
        let (impl_generics, _, where_clause) = generics.split_for_impl();
        let extra_validation = validate_struct_args.extra_validation.map(|extra_validation| quote! {{ #extra_validation }?;});

        quote!{
            #[automatically_derived]
            impl #impl_generics #account_set_validate<#info_lifetime, #validate_type> for #ident #ty_generics #where_clause {
                fn validate_accounts(
                    &mut self,
                    arg: #validate_type,
                    sys_calls: &mut impl #sys_call_invoke,
                ) -> #result<()> {
                    #(<#field_type as #account_set_validate<#info_lifetime, _>>::validate_accounts(&mut self.#field_name, #validate_args, sys_calls)?;)*
                    #extra_validation
                    Ok(())
                }
            }
        }
    }).collect()
}
