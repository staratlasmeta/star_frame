use crate::{
    account_set::{
        generics::AccountSetGenerics,
        struct_impl::{Requires, StepInput},
    },
    util::{new_generic, BetterGenerics, Paths},
};
use daggy::Dag;
use easy_proc::{find_attrs, ArgumentList};
use itertools::Itertools;
use proc_macro2::{Span, TokenStream};
use proc_macro_error2::abort;
use quote::quote;
use std::collections::{hash_map::Entry, HashMap, HashSet};
use syn::{Expr, Ident, LitStr, Type};

#[derive(ArgumentList, Default)]
struct ValidateStructArgs {
    id: Option<LitStr>,
    arg: Option<Type>,
    generics: Option<BetterGenerics>,
    before_validation: Option<Expr>,
    extra_validation: Option<Expr>,
    #[argument(presence)]
    inline_always: bool,
}

#[derive(ArgumentList, Clone)]
struct ValidateFieldArgs {
    /// The ident of the whole attribute, not required and can only be one
    #[argument(attr_ident)]
    attr_ident: Ident,
    id: Option<LitStr>,
    #[argument(presence)]
    funder: bool,
    #[argument(presence)]
    recipient: bool,
    #[argument(presence)]
    skip: bool,
    requires: Option<Requires>,
    arg: Option<Expr>,
    temp: Option<Expr>,
    arg_ty: Option<Type>,
    address: Option<Expr>,
}

impl Default for ValidateFieldArgs {
    fn default() -> Self {
        Self {
            attr_ident: Ident::new("validate", Span::call_site()),
            id: Default::default(),
            funder: Default::default(),
            recipient: Default::default(),
            skip: Default::default(),
            requires: Default::default(),
            arg: Default::default(),
            temp: Default::default(),
            arg_ty: Default::default(),
            address: Default::default(),
        }
    }
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
    let AccountSetGenerics { main_generics, .. } = account_set_generics;
    let Paths {
        result,
        validate_ident,
        prelude,
        account_set_validate,
        clone,
        box_ty,
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
        validate_ids.entry(None).or_insert_with(Default::default);
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
                    validate_field_arg.attr_ident,
                    "Validate id `{:?}` not found",
                    validate_field_arg.id.as_ref().map(LitStr::value)
                );
            }
            if !field_ids.insert(validate_field_arg.id.as_ref().map(LitStr::value)) {
                abort!(
                    validate_field_arg.attr_ident,
                    "Duplicate validate decode id `{:?}`",
                    validate_field_arg.id.as_ref().map(LitStr::value)
                );
            }
        }
    }

    validate_ids.into_iter().map(|(id, validate_struct_args)| {
        let relevant_field_validates = field_validates.iter().map(|f| f.iter().find(|f| f.id.as_ref().map(LitStr::value) == id).cloned().unwrap_or_default()).collect::<Vec<_>>();
        let (_, ty_generics, _) = main_generics.split_for_impl();
        let mut generics = main_generics.clone();
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
            generics.make_where_clause().predicates.push(syn::parse_quote!(#single_ty: #account_set_validate<#generic_arg> + #prelude::SingleAccountSet));
        }

        validate_type = validate_struct_args.arg.unwrap_or(validate_type);

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

        // set caches
        let mut has_funder = false;
        let mut has_recipient = false;

        // build the validate calls
        let validates = field_type.iter()
            .zip_eq(field_name.iter())
            .zip_eq(relevant_field_validates.iter())
            .map(|((field_type, field_name), args)| {
                if args.temp.is_some() && args.arg.is_none() {
                    abort!(args.arg, "Cannot specify `temp` when `arg` is not specified");
                }
                let validate = if args.skip {
                    quote! {}
                } else {
                    let default_expr: Type = syn::parse_quote!(_);
                    let validate_arg = args.arg.as_ref().unwrap_or(&default_validate_arg);
                    let validate_ty = args.arg_ty.as_ref().unwrap_or(&default_expr);
                    let temp = args.temp.as_ref();
                    let address_check = args.address.as_ref().map(|address| quote! {
                        #prelude::anyhow::Context::context(
                            <#field_type as #prelude::CheckKey>::check_key(&self.#field_name, #address),
                            ::std::stringify!(#ident::#field_name(#id)),
                        )?;
                    });
                    let temp = temp.as_ref().map(|temp| quote! {
                        let temp = #temp;
                    });
                    quote! {
                        {
                            #address_check
                            #temp
                            let __arg = #validate_arg;
                            #prelude::anyhow::Context::context(
                                #prelude::_account_set_validate_reverse::<#field_type, #validate_ty>(
                                    __arg,
                                    &mut self.#field_name,
                                    ctx
                                ),
                                ::std::stringify!(#ident::#field_name(#id)),
                            )?;
                        }
                    }
                };
                let funder = args.funder.then(|| {
                    if has_funder {
                        abort!(args.attr_ident, "Only one field can be marked as funder");
                    }
                    has_funder = true;
                    quote! {
                        if ctx.get_funder().is_none() {
                            ctx.set_funder(Box::new(#clone::clone(&self.#field_name)));
                        }
                    }
                });
                let recipient = args.recipient.then(|| {
                    if has_recipient {
                        abort!(args.attr_ident, "Only one field can be marked as recipient");
                    }
                    has_recipient = true;
                    quote! {
                        if ctx.get_recipient().is_none() {
                            ctx.set_recipient(#box_ty::new(#clone::clone(&self.#field_name)));
                        }
                    }
                });
                quote! {
                    #validate
                    #funder
                    #recipient
                }
            }).collect_vec();

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
            #prelude::anyhow::Context::context(res, ::std::stringify!(#ident::{Before Validation Failed} (#id)))?;
        });
        let extra_validation = validate_struct_args.extra_validation.map(|extra_validation| quote! {
            let res: #result<()> = { #extra_validation };
            #prelude::anyhow::Context::context(res, ::std::stringify!(#ident::{Extra Validation Failed} (#id)))?;
        });

        let inline_attr = if validate_struct_args.inline_always {
            quote!(#[inline(always)])
        } else {
            quote!(#[inline])
        };

        quote! {
            #[automatically_derived]
            impl #impl_generics #account_set_validate<#validate_type> for #ident #ty_generics #where_clause {
                #inline_attr
                fn validate_accounts(
                    &mut self,
                    arg: #validate_type,
                    ctx: &mut #prelude::Context,
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
