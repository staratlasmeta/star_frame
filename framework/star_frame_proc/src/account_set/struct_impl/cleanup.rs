use crate::account_set::generics::AccountSetGenerics;
use crate::account_set::{AccountSetStructArgs, StrippedDeriveInput};
use crate::util::{BetterGenerics, Paths};
use easy_proc::{find_attrs, ArgumentList};
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use syn::{Expr, Field, LitStr, Type};

#[derive(ArgumentList)]
struct CleanupStructArgs {
    id: Option<LitStr>,
    arg: Option<Type>,
    generics: Option<BetterGenerics>,
    extra_cleanup: Option<Expr>,
}

#[derive(ArgumentList)]
struct CleanupFieldArgs {
    id: Option<LitStr>,
    arg: Expr,
}

pub(super) fn cleanups(
    paths: &Paths,
    input: &StrippedDeriveInput,
    account_set_struct_args: &AccountSetStructArgs,
    account_set_generics: &AccountSetGenerics,
    fields: &[&Field],
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
        syscall_invoke,
        cleanup_ident,
        account_set_cleanup,
        ..
    } = paths;

    let mut cleanup_ids = HashMap::new();
    for cleanup_struct_args in
        find_attrs(&input.attrs, cleanup_ident).map(CleanupStructArgs::parse_arguments)
    {
        match cleanup_ids.entry(cleanup_struct_args.id.as_ref().map(LitStr::value)) {
            Entry::Vacant(entry) => {
                entry.insert(cleanup_struct_args);
            }
            Entry::Occupied(entry) => {
                abort!(
                    entry.get().id,
                    "Duplicate cleanup id `{:?}`",
                    entry.get().id.as_ref().map(LitStr::value)
                );
            }
        }
    }
    if !account_set_struct_args.skip_default_cleanup {
        cleanup_ids
            .entry(None)
            .or_insert_with(|| CleanupStructArgs {
                id: None,
                arg: None,
                generics: None,
                extra_cleanup: None,
            });
    }

    let field_cleanups = fields
        .iter()
        .map(|f| {
            find_attrs(&f.attrs, cleanup_ident)
                .map(CleanupFieldArgs::parse_arguments)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    for field_cleanup in &field_cleanups {
        let mut field_ids = HashSet::new();
        for cleanup_field_arg in field_cleanup {
            if !cleanup_ids.contains_key(&cleanup_field_arg.id.as_ref().map(LitStr::value)) {
                abort!(
                    cleanup_field_arg.id,
                    "Cleanup id `{:?}` not found",
                    cleanup_field_arg.id.as_ref().map(LitStr::value)
                );
            }
            if !field_ids.insert(cleanup_field_arg.id.as_ref().map(LitStr::value)) {
                abort!(
                    cleanup_field_arg.id,
                    "Cleanup decode id `{:?}`",
                    cleanup_field_arg.id.as_ref().map(LitStr::value)
                );
            }
        }
    }

    cleanup_ids.into_iter().map(|(id, cleanup_struct_args)|{
        let cleanup_type: Type = cleanup_struct_args.arg.unwrap_or_else(|| syn::parse_quote!(()));
        let cleanup_args: Vec<Expr> = field_cleanups.iter().map(|f| {
            f.iter().find(|f| f.id.as_ref().map(LitStr::value) == id).map(|f| f.arg.clone()).unwrap_or_else(|| syn::parse_quote!(()))
        }).collect();

        let (_, ty_generics, _) = main_generics.split_for_impl();
        let mut generics = other_generics.clone();
        if let Some(extra_generics) = cleanup_struct_args.generics.map(|g| g.into_inner()) {
            generics.params.extend(extra_generics.params);
            if let Some(extra_where_clause) = extra_generics.where_clause {
                generics
                    .make_where_clause()
                    .predicates
                    .extend(extra_where_clause.predicates);
            }
        }
        let (impl_generics, _, where_clause) = generics.split_for_impl();
        let extra_cleanup = cleanup_struct_args.extra_cleanup.map(|extra_validation| quote! {{ #extra_validation }?;});

        quote!{
            #[automatically_derived]
            impl #impl_generics #account_set_cleanup<#info_lifetime, #cleanup_type> for #ident #ty_generics #where_clause {
                fn cleanup_accounts(
                    &mut self,
                    arg: #cleanup_type,
                    syscalls: &mut impl #syscall_invoke,
                ) -> #result<()> {
                    #(<#field_type as #account_set_cleanup<#info_lifetime, _>>::cleanup_accounts(&mut self.#field_name, #cleanup_args, syscalls)?;)*
                    #extra_cleanup
                    Ok(())
                }
            }
        }
    }).collect()
}
