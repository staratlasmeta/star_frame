use crate::{
    account_set::{generics::AccountSetGenerics, struct_impl::StepInput},
    util::{new_generic, BetterGenerics, Paths},
};
use easy_proc::{find_attrs, ArgumentList};
use proc_macro2::TokenStream;
use proc_macro_error2::abort;
use quote::quote;
use std::collections::{hash_map::Entry, HashMap, HashSet};
use syn::{Expr, LitStr, Type};

#[derive(ArgumentList, Debug, Default)]
struct CleanupStructArgs {
    id: Option<LitStr>,
    arg: Option<Type>,
    generics: Option<BetterGenerics>,
    extra_cleanup: Option<Expr>,
    #[argument(presence)]
    inline_always: bool,
}

#[derive(ArgumentList)]
struct CleanupFieldArgs {
    id: Option<LitStr>,
    arg: Option<Expr>,
    #[argument(presence)]
    normalize_rent: bool,
}

pub(super) fn cleanups(
    StepInput {
        paths,
        input,
        account_set_struct_args,
        account_set_generics,
        single_set_field,
        field_name,
        fields,
        field_type,
        ..
    }: StepInput,
) -> Vec<TokenStream> {
    let ident = &input.ident;
    let AccountSetGenerics { main_generics, .. } = account_set_generics;
    let Paths {
        result,
        cleanup_ident,
        prelude,
        account_set_cleanup,
        normalize_rent,
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
        cleanup_ids.entry(None).or_insert_with(Default::default);
    }

    let field_cleanups = fields
        .iter()
        .map(|f| {
            find_attrs(&f.attrs, cleanup_ident)
                .map(|a| (a, CleanupFieldArgs::parse_arguments(a)))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    for field_cleanup in &field_cleanups {
        let mut field_ids = HashSet::new();
        for (_, cleanup_field_arg) in field_cleanup {
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

    cleanup_ids.into_iter().map(|(id, cleanup_struct_args)| {
        let (_, ty_generics, _) = main_generics.split_for_impl();
        let mut generics = main_generics.clone();
        let mut cleanup_type: Type = syn::parse_quote!(());
        let mut default_cleanup_arg: Expr = syn::parse_quote!(());
        if let Some(extra_generics) = cleanup_struct_args.generics.map(|g| g.into_inner()) {
            generics.params.extend(extra_generics.params);
            if let Some(extra_where_clause) = extra_generics.where_clause {
                generics
                    .make_where_clause()
                    .predicates
                    .extend(extra_where_clause.predicates);
            }
        } else if let Some(single_set_field) = single_set_field {
            let generic_arg = new_generic(main_generics, None);
            default_cleanup_arg = syn::parse_quote!(arg);
            cleanup_type = syn::parse_quote!(#generic_arg);
            generics.params.push(syn::parse_quote!(#generic_arg));
            let single_ty = &single_set_field.ty;
            generics.make_where_clause().predicates.push(syn::parse_quote!(#single_ty: #account_set_cleanup<#generic_arg> + #prelude::SingleAccountSet));
        }
        let cleanup_type = cleanup_struct_args.arg.unwrap_or(cleanup_type);

        let cleanup_args: Vec<Expr> = field_cleanups
            .iter()
            .map(|f| {
                let found = f.iter()
                    .find(|(_, f)| f.id.as_ref().map(LitStr::value) == id);

                match found {
                    Some((_, CleanupFieldArgs{ arg: Some(arg), normalize_rent: false, .. })) => arg.clone(),
                    Some((_, CleanupFieldArgs{ arg: None, normalize_rent: true, .. })) => syn::parse_quote!{ #normalize_rent::<()>(()) },
                    Some((a , CleanupFieldArgs{ arg: Some(_), normalize_rent: true, .. })) => abort!(a, "Cannot provide both `arg` and `normalize_rent`"),
                    Some((a, CleanupFieldArgs{ arg: None, normalize_rent: false, .. })) => abort!(a, "Must provide either `arg` or `normalize_rent`"),
                    None => default_cleanup_arg.clone()
                }
            }).collect();

        let (impl_generics, _, where_clause) = generics.split_for_impl();
        let extra_cleanup = cleanup_struct_args.extra_cleanup.map(|extra_validation| quote! {{ #extra_validation }?;});

        let inline_attr = if cleanup_struct_args.inline_always {
            quote!(#[inline(always)])
        } else {
            quote!(#[inline])
        };

        let handle_paths = field_name.iter().map(|field_name| if single_set_field.is_some() {
            quote! { res?; }
        } else {
            quote! { #prelude::ErrorInfo::account_path(res, ::std::stringify!(#field_name))?; }
        }).collect::<Vec<_>>();

        quote! {
            #[automatically_derived]
            impl #impl_generics #account_set_cleanup<#cleanup_type> for #ident #ty_generics #where_clause {
                #inline_attr
                fn cleanup_accounts(
                    &mut self,
                    arg: #cleanup_type,
                    ctx: &mut #prelude::Context,
                ) -> #result<()> {
                    #(
                        let __arg = #cleanup_args;
                        let res = #prelude::_account_set_cleanup_reverse::<#field_type, _>(
                            __arg,
                            &mut self.#field_name,
                            ctx,
                        );
                        #handle_paths
                    )*
                    #extra_cleanup
                    Ok(())
                }
            }
        }
    }).collect()
}
