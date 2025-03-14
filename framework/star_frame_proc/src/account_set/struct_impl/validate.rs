use crate::account_set::generics::AccountSetGenerics;
use crate::account_set::struct_impl::{Requires, StepInput};
use crate::util::{new_generic, BetterGenerics, Paths};
use daggy::Dag;
use easy_proc::{find_attrs, ArgumentList};
use itertools::Itertools;
use proc_macro2::TokenStream;
use proc_macro_error2::abort;
use quote::quote;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use syn::{Expr, LitStr, Type};

#[derive(ArgumentList)]
struct ValidateStructArgs {
    id: Option<LitStr>,
    arg: Option<Type>,
    generics: Option<BetterGenerics>,
    before_validation: Option<Expr>,
    extra_validation: Option<Expr>,
}

#[derive(ArgumentList, Clone, Default)]
struct ValidateFieldArgs {
    id: Option<LitStr>,
    #[argument(presence)]
    skip: bool,
    requires: Option<Requires>,
    arg: Option<Expr>,
    arg_ty: Option<Type>,
    address: Option<Expr>,
}

pub(super) fn validates(
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
        syscall_invoke,
        validate_ident,
        prelude,
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
                before_validation: None,
            });
    }

    let field_validates = fields
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

    validate_ids.into_iter().map(|(id, validate_struct_args)| {
        let relevant_field_validates = field_validates.iter().map(|f| f.iter().find(|f| f.id.as_ref().map(LitStr::value) == id).cloned().unwrap_or_default()).collect::<Vec<_>>();
        let (_, ty_generics, _) = main_generics.split_for_impl();
        let mut generics = other_generics.clone();
        let mut validate_type: Type = syn::parse_quote!(());
        let mut default_validate_arg: Expr = syn::parse_quote!(());
        if let Some(extra_generics) = validate_struct_args.generics.map(|g| g.into_inner()) {
            generics.params.extend(extra_generics.params);
            if let Some(extra_where_clause) = extra_generics.where_clause {
                generics
                    .make_where_clause()
                    .predicates
                    .extend(extra_where_clause.predicates);
            }
        } else if let Some(single_set_field) = single_set_field {
            let generic_arg = new_generic(main_generics, None);
            default_validate_arg = syn::parse_quote!(arg);
            validate_type = syn::parse_quote!(#generic_arg);
            generics.params.push(syn::parse_quote!(#generic_arg));
            let single_ty = &single_set_field.ty;
            generics.make_where_clause().predicates.push(syn::parse_quote!(#single_ty: #account_set_validate<#info_lifetime, #generic_arg> + #prelude::SingleAccountSet<#info_lifetime>));
        }

        validate_type = validate_struct_args.arg.unwrap_or(validate_type);

        let validate_args: Vec<(Expr, Type)> = relevant_field_validates
            .iter()
            .map(|f| {
                (f.arg.clone().unwrap_or_else(|| default_validate_arg.clone()), f.arg_ty.clone().unwrap_or_else(|| syn::parse_quote!(_)))
            }).collect();

        let validate_addresses = relevant_field_validates.iter().map(|f| f.address.clone()).collect_vec();

        // Cycle detection
        let mut field_id_map = HashMap::new();
        let mut validates_dag = Dag::<_, _, u32>::new();
        for field_name in field_name.iter().map(|f| f.to_string()) {
            field_id_map.insert(field_name, validates_dag.add_node(()));
        }
        for (field_arg, field_name) in relevant_field_validates.iter().zip(field_name).filter_map(|(a, name)| a.requires.as_ref().map(|r| (r, name.to_string()))) {
            for required in field_arg.required_fields.iter() {
                let from = field_id_map.get(&required.to_string()).unwrap_or_else(|| abort!(required, "Field `{:?}` not found", required));
                let to = field_id_map.get(&field_name).unwrap();
                if validates_dag.add_edge(*from, *to, ()).is_err() {
                    abort!(required, "Cycle detected in `requires`")
                }
            }
        }

        // build requires

        // build the validate calls
        let validates = field_type.iter()
            .zip(field_name.iter())
            .zip(validate_args.iter())
            .zip(validate_addresses.iter())
            .zip(relevant_field_validates.iter().map(|a| {
                if a.skip && a.arg.is_some() {
                    abort!(a.arg, "Cannot specify arg when skip is true");
                }
                a.skip
            }))
            .map(|((((field_type, field_name), (validate_arg, validate_ty)), validate_address), skip)| if skip {
                quote! {}
            } else {
                let address_check = validate_address.as_ref().map(|address| quote! {
                    <#field_type as #prelude::SingleAccountSet<#info_lifetime>>::check_key(&self.#field_name, #address)?;
                });
                quote! {
                    {
                        #address_check
                        let __arg = #validate_arg;
                        #prelude::_account_set_validate_reverse::<#field_type, #validate_ty>(
                            __arg,
                            &mut self.#field_name,
                            syscalls
                        )?;
                    }
                }
            })
            .collect::<Vec<_>>();
        // Stores named validates in order
        let mut out: Vec<(TokenStream, String)> = Vec::new();
        // Map requires to vec of strings
        let relevant_requires = relevant_field_validates
            .iter()
            .map(|f| f.requires
                .as_ref()
                .map(|r| &r.required_fields)
                .map(|r| r.clone()
                    .into_iter()
                    .map(|r| r.to_string())
                    .collect::<Vec<_>>()
                )
                .unwrap_or_default()
            );
        // Go backwards over validate calls paired with field name and what it requires
        let iter = validates.into_iter()
            .zip(relevant_requires)
            .zip(field_name.iter().map(|f| f.to_string()))
            .rev();
        for ((validate, required), field_name) in iter {
            let insert_index = out.iter()
                .enumerate()
                // find from the end
                .rev()
                .find(|(_, (_, name))| required.contains(name))
                .map(|(index, _)| index + 1)
                .unwrap_or(0);
            out.insert(insert_index, (validate, field_name));
        }
        let validates = out.into_iter().map(|(validate, _)| validate);

        let (impl_generics, _, where_clause) = generics.split_for_impl();
        let before_validation = validate_struct_args.before_validation.map(|before_validation| quote! {
            let res: #result<()> = { #before_validation };
            res?;
        });
        let extra_validation = validate_struct_args.extra_validation.map(|extra_validation| quote! {
            let res: #result<()> = { #extra_validation };
            res?;
        });

        quote! {
            #[automatically_derived]
            impl #impl_generics #account_set_validate<#info_lifetime, #validate_type> for #ident #ty_generics #where_clause {
                fn validate_accounts(
                    &mut self,
                    arg: #validate_type,
                    syscalls: &mut impl #syscall_invoke<#info_lifetime>,
                ) -> #result<()> {
                    #before_validation
                    #(#validates)*
                    #extra_validation
                    Ok(())
                }
            }
        }
    }).collect()
}
