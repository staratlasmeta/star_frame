use crate::account_set::generics::AccountSetGenerics;
use crate::account_set::struct_impl::decode::DecodeFieldTy;
use crate::account_set::{AccountSetStructArgs, SingleAccountSetFieldArgs, StrippedDeriveInput};
use crate::util::{make_struct, new_generic, new_lifetime, Paths};
use easy_proc::{find_attr, ArgumentList};
use itertools::Itertools;
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::{format_ident, quote, ToTokens};
use std::ops::Not;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{
    bracketed, parse_quote, token, Attribute, DataStruct, Field, Ident, Index, Token,
    WherePredicate,
};

mod cleanup;
mod decode;
mod idl;
mod validate;

#[derive(Debug, Clone)]
struct Requires {
    #[allow(dead_code)]
    bracket: token::Bracket,
    required_fields: Punctuated<Ident, Token![,]>,
}
impl Parse for Requires {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            bracket: bracketed!(content in input),
            required_fields: content.parse_terminated(Ident::parse, Token![,])?,
        })
    }
}

#[derive(ArgumentList, Debug, Clone)]
struct AccountSetFieldAttrs {
    skip: Option<TokenStream>,
    #[argument(presence)]
    program: bool,
    #[argument(presence)]
    funder: bool,
    #[argument(presence)]
    recipient: bool,
}

impl AccountSetFieldAttrs {
    fn skip(&self) -> bool {
        if self.skip.is_some() {
            if self.program || self.funder || self.recipient {
                abort!(
                    self.skip,
                    "Cannot use `skip` with `program`, `funder`, or `recipient`"
                );
            }
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct StepInput<'a> {
    paths: &'a Paths,
    input: &'a StrippedDeriveInput,
    account_set_struct_args: &'a AccountSetStructArgs,
    account_set_generics: &'a AccountSetGenerics,
    single_set_field: Option<&'a Field>,
    field_name: &'a [TokenStream],
    fields: &'a [&'a Field],
    field_type: &'a [&'a syn::Type],
}

pub(super) fn derive_account_set_impl_struct(
    paths: Paths,
    data_struct: DataStruct,
    account_set_struct_args: AccountSetStructArgs,
    input: StrippedDeriveInput,
    account_set_generics: AccountSetGenerics,
) -> TokenStream {
    let AccountSetGenerics {
        main_generics,
        other_generics,
        info_lifetime,
        function_generic_type,
        ..
    } = &account_set_generics;

    let Paths {
        account_info,
        account_set,
        prelude,
        result,
        ..
    } = &paths;

    let ident = &input.ident;

    let filter_skip = |f: &&Field| -> bool {
        find_attr(&f.attrs, &paths.account_set_ident)
            .map(AccountSetFieldAttrs::parse_arguments)
            .map(|args| !args.skip())
            .unwrap_or(true)
    };

    let resolve_field_name = |(index, field): (_, &Field)| {
        field
            .ident
            .as_ref()
            .map(ToTokens::to_token_stream)
            .unwrap_or_else(|| Index::from(index).into_token_stream())
    };

    let all_field_name = data_struct
        .fields
        .iter()
        .enumerate()
        .map(resolve_field_name)
        .collect::<Vec<_>>();

    let (fields, skipped_fields): (Vec<_>, Vec<_>) =
        data_struct.fields.iter().partition(filter_skip);
    let field_name = data_struct
        .fields
        .iter()
        .enumerate()
        .filter(|(_, f)| filter_skip(f))
        .map(resolve_field_name)
        .collect::<Vec<_>>();
    let field_type = data_struct
        .fields
        .iter()
        .filter(filter_skip)
        .map(|field| &field.ty)
        .collect::<Vec<_>>();

    let mut single_account_sets = fields
        .iter()
        .copied()
        .enumerate()
        .filter_map(|field| {
            find_attr(&field.1.attrs, &paths.single_account_set_ident).map(|a| {
                let parsed = if a.meta.require_path_only().is_ok() {
                    Default::default()
                } else {
                    SingleAccountSetFieldArgs::parse_arguments(a)
                };
                (field.1, field_name[field.0].clone(), parsed)
            })
        })
        .collect::<Vec<_>>();

    skipped_fields.iter().for_each(|field| {
        if let Some(attr) = find_attr(&field.attrs, &paths.single_account_set_ident) {
            abort!(
                attr,
                "`{}` cannot be applied to skipped fields",
                &paths.single_account_set_ident
            );
        }
    });

    if single_account_sets.len() > 1 {
        abort!(
            single_account_sets[1].0,
            "Only one field can be marked as `{}`",
            &paths.single_account_set_ident
        );
    }

    let single_account_set = single_account_sets.pop();
    let mut single_set_field = None;

    let (_, ty_generics, _) = main_generics.split_for_impl();

    let single_account_set_impls = single_account_set.map(|(field, field_name, args)| {
        if fields.len() > 1 {
            abort!(
                field,
                "`{}` can only be applied to a struct with a single unskipped field",
                &paths.single_account_set_ident
            );
        }
        let single_generics = main_generics.clone();
        single_set_field.replace(field.clone());
        let sg_impl = single_generics.clone();
        let (sg_impl, _, _) = sg_impl.split_for_impl();

        let mut info_sg = single_generics.clone();
        if !info_sg.lifetimes().any(|l| l.lifetime.ident == info_lifetime.ident) {
            info_sg.params.push(parse_quote! {
                #info_lifetime
            });
        }

        let info_sg_impl = info_sg.clone();
        let (info_sg_impl, _, _) = info_sg_impl.split_for_impl();

        let mut info_gen_sg = info_sg.clone();
        let new_generic = new_generic(&info_gen_sg);

        info_gen_sg.params.push(parse_quote! {
            #new_generic
        });

        let info_gen_sg_impl = info_gen_sg.clone();
        let (info_gen_sg_impl, _, _) = info_gen_sg_impl.split_for_impl();

        let self_single_bound: WherePredicate = parse_quote! {
            Self: #prelude::SingleAccountSet<#info_lifetime>
        };

        let mut single_set_generics = info_sg.clone();
        let single_where = single_set_generics.make_where_clause();
        let field_ty = &field.ty;

        single_where.predicates.push(parse_quote! {
            #field_ty: #prelude::SingleAccountSet<#info_lifetime>
        });
        single_where.predicates.push(parse_quote!{
            Self: #prelude::AccountSet<#info_lifetime>
        });

        let signer = args.signer.then(||quote!(signer: true,));
        let writable = args.writable.then(||quote!(writable: true,));

        let single = quote! {
            #[automatically_derived]
            impl #info_sg_impl #prelude::SingleAccountSet<#info_lifetime> for #ident #ty_generics #single_where {
                #[allow(clippy::needless_update)]
                const META: #prelude::SingleSetMeta = #prelude::SingleSetMeta {
                    #signer
                    #writable
                    ..<#field_ty as #prelude::SingleAccountSet<#info_lifetime>>::META
                };

                #[inline]
                fn account_info(&self) -> &#account_info<#info_lifetime> {
                    <#field_ty as #prelude::SingleAccountSet<#info_lifetime>>::account_info(&self.#field_name)
                }
            }
        };

        let signed_account = args.skip_signed_account.not().then(||{
            let mut signed_generics = info_sg.clone();
            let signed_where = signed_generics.make_where_clause();
            signed_where.predicates.push(parse_quote! {
                #field_ty: #prelude::SignedAccount<#info_lifetime>
            });
            signed_where.predicates.push(self_single_bound.clone());
            quote! {
                #[automatically_derived]
                impl #info_sg_impl #prelude::SignedAccount<#info_lifetime> for #ident #ty_generics #signed_where {
                    #[inline]
                    fn signer_seeds(&self) -> Option<Vec<&[u8]>> {
                        <#field_ty as #prelude::SignedAccount<#info_lifetime>>::signer_seeds(&self.#field_name)
                    }
                }
            }
        });

        let writable_account = args.skip_writable_account.not().then(||{
            let mut writable_generics = info_sg.clone();
            let writable_where = writable_generics.make_where_clause();
            writable_where.predicates.push(parse_quote! {
                #field_ty: #prelude::WritableAccount<#info_lifetime>
            });
            writable_where.predicates.push(self_single_bound.clone());
            quote! {
                #[automatically_derived]
                impl #info_sg_impl #prelude::WritableAccount<#info_lifetime> for #ident #ty_generics #writable_where {}
            }
        });

        let has_program_account = args.skip_has_program_account.not().then(||{
            let mut program_generics = single_generics.clone();
            let program_where = program_generics.make_where_clause();
            program_where.predicates.push(parse_quote! {
                #field_ty: #prelude::HasProgramAccount
            });
            quote! {
                #[automatically_derived]
                impl #sg_impl #prelude::HasProgramAccount for #ident #ty_generics #program_where {
                    type ProgramAccount = <#field_ty as #prelude::HasProgramAccount>::ProgramAccount;
                }
            }
        });

        let has_owner_program = args.skip_has_owner_program.not().then(||{
            let mut owner_generics = single_generics.clone();
            let owner_where = owner_generics.make_where_clause();
            owner_where.predicates.push(parse_quote! {
                #field_ty: #prelude::HasOwnerProgram
            });
            quote! {
                #[automatically_derived]
                impl #sg_impl #prelude::HasOwnerProgram for #ident #ty_generics #owner_where {
                    type OwnerProgram = <#field_ty as #prelude::HasOwnerProgram>::OwnerProgram;
                }
            }
        });

        let has_seeds = args.skip_has_seeds.not().then(||{
            let mut seeds_generics = single_generics.clone();
            let seeds_where = seeds_generics.make_where_clause();
            seeds_where.predicates.push(parse_quote! {
                #field_ty: #prelude::HasSeeds
            });
            quote! {
                #[automatically_derived]
                impl #sg_impl #prelude::HasSeeds for #ident #ty_generics #seeds_where {
                    type Seeds = <#field_ty as #prelude::HasSeeds>::Seeds;
                }
            }
        });

        let can_init_seeds = args.skip_can_init_seeds.not().then(||{
            let mut init_seeds_generics = info_gen_sg.clone();
            let init_seeds_where = init_seeds_generics.make_where_clause();
            init_seeds_where.predicates.push(parse_quote! {
                #field_ty: #prelude::CanInitSeeds<#info_lifetime, #function_generic_type>
            });
            init_seeds_where.predicates.push(parse_quote! {
                Self: #prelude::AccountSetValidate<#info_lifetime, #function_generic_type>
            });
            init_seeds_where.predicates.push(self_single_bound.clone());
            quote! {
                #[automatically_derived]
                impl #info_gen_sg_impl #prelude::CanInitSeeds<#info_lifetime, #new_generic> for #ident #ty_generics #init_seeds_where {
                    #[inline]
                    fn init_seeds(&mut self, arg: &#new_generic, syscalls: &impl #prelude::SyscallInvoke<#info_lifetime>) -> #result<()> {
                        <#field_ty as #prelude::CanInitSeeds<#info_lifetime, #new_generic>>::init_seeds(&mut self.#field_name, arg, syscalls)
                    }
                }
            }
        });

        let can_init_account = args.skip_can_init_account.not().then(||{
            let mut init_generics = info_gen_sg.clone();
            let init_where = init_generics.make_where_clause();
            init_where.predicates.push(parse_quote! {
                #field_ty: #prelude::CanInitAccount<#info_lifetime, #new_generic>
            });
            quote! {
                #[automatically_derived]
                impl #info_gen_sg_impl #prelude::CanInitAccount<#info_lifetime, #new_generic> for #ident #ty_generics #init_where {
                    #[inline]
                    fn init_account(
                        &mut self,
                        arg: #new_generic,
                        syscalls: &impl #prelude::SyscallInvoke<#info_lifetime>,
                        account_seeds: Option<Vec<&[u8]>>,
                    ) -> #result<()> {
                        <#field_ty as #prelude::CanInitAccount<#info_lifetime, #new_generic>>::init_account(&mut self.#field_name, arg, syscalls, account_seeds)
                    }
                }
            }
        });

        quote!{
            #single

            #signed_account
            #writable_account
            #has_program_account
            #has_owner_program
            #has_seeds
            #can_init_seeds
            #can_init_account
        }
    });

    let ident_str = ident.to_string();
    let trimmed_ident_str = ident_str.strip_suffix("Accounts").unwrap_or(&ident_str);

    let cpi_account_set_impl = (!account_set_struct_args.skip_cpi_account_set && single_account_set_impls.is_none()).then(|| {
        let cpi_accounts_ident= format_ident!("{trimmed_ident_str}CpiAccounts");
        let (_, self_ty_gen, _) = main_generics.split_for_impl();
        let mut cpi_gen = other_generics.clone();
        let where_clause = cpi_gen.make_where_clause();
        let cpi_set = quote!(#prelude::CpiAccountSet<#info_lifetime>);
        let cpi_accounts = quote!(Self::CpiAccounts<#info_lifetime>);

        let new_fields: Vec<Field> = fields
            .iter()
            .map(|field| {
                let Field {
                    vis,
                    ident,
                    colon_token,
                    ty,
                    ..
                } = field;
                where_clause.predicates.push(parse_quote! {
                    #ty: #cpi_set
                });
                parse_quote!(#vis #ident #colon_token <#ty as #cpi_set>::CpiAccounts<#info_lifetime>)
            })
            .collect();

        let new_struct_gen = if new_fields.is_empty() {
            main_generics
        } else {
            other_generics
        };
        let cpi_accounts_struct = make_struct(&cpi_accounts_ident, &new_fields, new_struct_gen);
        let new_struct_ty_gen = new_struct_gen.split_for_impl().1;

        let accounts_lifetime = new_lifetime(&cpi_gen);

        let (impl_gen, _, where_clause) = cpi_gen.split_for_impl();

        quote! {
            #[automatically_derived]
            #[derive(Clone, Debug)]
            #cpi_accounts_struct

            #[automatically_derived]
            impl #impl_gen #cpi_set for #ident #self_ty_gen #where_clause {
                type CpiAccounts<#accounts_lifetime> = #cpi_accounts_ident #new_struct_ty_gen;
                const MIN_LEN: usize =  0#(+ <#field_type as #cpi_set>::MIN_LEN)*;

                #[inline]
                fn extend_account_infos(
                    accounts: #cpi_accounts,
                    infos: &mut Vec<#account_info<#info_lifetime>>,
                ) {
                    #(<#field_type as #cpi_set>::extend_account_infos(accounts.#field_name, infos);)*
                }

                #[inline]
                fn extend_account_metas(
                    program_id: &#prelude::Pubkey,
                    accounts: &#cpi_accounts,
                    metas: &mut Vec<#prelude::AccountMeta>,
                ) {
                    #(<#field_type as #cpi_set>::extend_account_metas(program_id, &accounts.#field_name, metas);)*
                }
            }
        }
    });

    let client_account_set_impl = (!account_set_struct_args.skip_client_account_set && single_account_set_impls.is_none()).then(|| {
        let client_accounts_ident= format_ident!("{trimmed_ident_str}ClientAccounts");
        let client_set = quote!(#prelude::ClientAccountSet);
        let client_accounts = quote!(Self::ClientAccounts);

        let mut client_gen = main_generics.clone();
        let where_clause = client_gen.make_where_clause();

        let new_fields: Vec<Field> = fields
            .iter()
            .map(|field| {
                let Field {
                    vis,
                    ident,
                    colon_token,
                    ty,
                    ..
                } = field;
                where_clause.predicates.push(parse_quote! {
                    #ty: #client_set
                });
                parse_quote!(#vis #ident #colon_token <#ty as #client_set>::ClientAccounts)
            })
            .collect();

        let client_accounts_struct = make_struct(&client_accounts_ident, &new_fields, &client_gen);


        let (impl_gen, ty_gen, where_clause) = client_gen.split_for_impl();

        quote! {
            #[automatically_derived]
            #[derive(Clone, Debug)]
            #client_accounts_struct

            #[automatically_derived]
            impl #impl_gen #client_set for #ident #ty_gen #where_clause {
                type ClientAccounts = #client_accounts_ident #ty_gen;
                const MIN_LEN: usize =  0#(+ <#field_type as #client_set>::MIN_LEN)*;

                #[inline]
                fn extend_account_metas(
                    program_id: &#prelude::Pubkey,
                    accounts: &#client_accounts,
                    metas: &mut Vec<#prelude::AccountMeta>,
                ) {
                    #(<#field_type as #client_set>::extend_account_metas(program_id, &accounts.#field_name, metas);)*
                }
            }
        }
    });

    let mut generics = other_generics.clone();
    if let Some(extra_generics) = &account_set_struct_args.generics {
        generics.params.extend(extra_generics.params.clone());
        if let Some(extra_where_clause) = &extra_generics.where_clause {
            generics
                .make_where_clause()
                .predicates
                .extend(extra_where_clause.predicates.clone());
        }
    } else if let Some(single_set_field) = &single_set_field {
        let single_ty = &single_set_field.ty;
        generics.make_where_clause().predicates.push(parse_quote! {
            #single_ty: #account_set<#info_lifetime>
        });
    }
    let (other_impl_generics, _, other_where_clause) = generics.split_for_impl();

    let decode_types = data_struct
        .fields
        .iter()
        .map(|field| {
            find_attr(&field.attrs, &paths.account_set_ident)
                .map(AccountSetFieldAttrs::parse_arguments)
                .and_then(|args| args.skip)
                .map_or_else(|| DecodeFieldTy::Type(&field.ty), DecodeFieldTy::Default)
        })
        .collect::<Vec<_>>();

    let step_input = StepInput {
        paths: &paths,
        input: &input,
        account_set_struct_args: &account_set_struct_args,
        account_set_generics: &account_set_generics,
        single_set_field: single_set_field.as_ref(),
        field_name: &field_name,
        fields: &fields,
        field_type: &field_type,
    };

    let decodes = decode::decodes(step_input, &data_struct, &all_field_name, &decode_types);
    let validates = validate::validates(step_input);
    let cleanups = cleanup::cleanups(step_input);
    let idls = idl::idls(step_input);

    let set_account_caches = {
        let find_field_names =
            |is_active: fn(AccountSetFieldAttrs) -> bool| -> Vec<(TokenStream, Attribute)> {
                data_struct
                    .fields
                    .iter()
                    .zip(all_field_name.iter())
                    .filter_map(|(field, name)| {
                        let attr = find_attr(&field.attrs, &paths.account_set_ident)?;
                        let args = AccountSetFieldAttrs::parse_arguments(attr);
                        is_active(args).then_some((name.clone(), attr.clone()))
                    })
                    .collect_vec()
            };

        let single_name = |name: &str, names: &[(TokenStream, Attribute)]| {
            if names.len() > 1 {
                abort!(
                    names[1].1,
                    format!("Only one field can be marked as {}", name)
                );
            }
            names.first().map(|(name, _)| name.clone())
        };

        let set_programs = find_field_names(|args| args.program)
            .iter()
            .map(|(name, _attr)| {
                quote! {
                    syscalls.insert_program(&self.#name);
                }
            })
            .collect_vec();

        let set_funder =
            single_name("funder", &find_field_names(|args| args.funder)).map(|field_name| {
                quote! {
                    if syscalls.get_funder().is_none() {
                        syscalls.set_funder(&self.#field_name);
                    }
                }
            });

        let set_recipient =
            single_name("recipient", &find_field_names(|args| args.recipient)).map(|field_name| {
                quote! {
                    if syscalls.get_recipient().is_none() {
                        syscalls.set_recipient(&self.#field_name);
                    }
                }
            });
        quote! {
            #(#set_programs)*
            #set_funder
            #set_recipient
        }
    };

    let account_set_impl = account_set_struct_args.skip_account_set.not().then(|| {
        quote! {
            #[automatically_derived]
            impl #other_impl_generics #account_set<#info_lifetime> for #ident #ty_generics #other_where_clause {
                #[inline]
                fn set_account_cache(
                    &mut self,
                    syscalls: &mut impl #prelude::SyscallAccountCache<#info_lifetime>,
                ) {
                    #set_account_caches
                    #(<#field_type as #account_set<#info_lifetime>>::set_account_cache(&mut self.#field_name, syscalls);)*
                }
            }
        }
    });

    quote! {
        #account_set_impl
        #cpi_account_set_impl
        #client_account_set_impl

        #(#decodes)*
        #(#validates)*
        #(#cleanups)*
        #(#idls)*

        #single_account_set_impls
    }
}
