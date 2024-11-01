use crate::account_set::generics::AccountSetGenerics;
use crate::account_set::struct_impl::decode::DecodeFieldTy;
use crate::account_set::{AccountSetStructArgs, SingleAccountSetFieldArgs, StrippedDeriveInput};
use crate::util::{new_generic, Paths};
use easy_proc::{find_attr, ArgumentList};
use itertools::Itertools;
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::{quote, ToTokens};
use std::ops::Not;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{
    bracketed, parse_quote, token, Attribute, DataStruct, Field, Ident, Index, Token,
    WherePredicate,
};

mod cleanup;
mod decode;
#[cfg(feature = "idl")]
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
        macro_prelude,
        result,
        ..
    } = &paths;

    let ident = &input.ident;

    let filter_skip = |f: &&Field| -> bool {
        find_attr(&f.attrs, &paths.account_set_ident)
            .map(AccountSetFieldAttrs::parse_arguments)
            .map(|args| args.skip.is_none())
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
            Self: #macro_prelude::SingleAccountSet<#info_lifetime>
        };

        let mut single_set_generics = info_sg.clone();
        let single_where = single_set_generics.make_where_clause();
        let field_ty = &field.ty;

        single_where.predicates.push(parse_quote! {
            #field_ty: #macro_prelude::SingleAccountSet<#info_lifetime>
        });
        single_where.predicates.push(parse_quote!{
            Self: #macro_prelude::AccountSet<#info_lifetime>
        });

        let single = quote! {
            #[automatically_derived]
            impl #info_sg_impl #macro_prelude::SingleAccountSet<#info_lifetime> for #ident #ty_generics #single_where {
                fn account_info(&self) -> &#account_info<#info_lifetime> {
                    <#field_ty as #macro_prelude::SingleAccountSet<#info_lifetime>>::account_info(&self.#field_name)
                }
            }
        };

        let signed_account = args.skip_signed_account.not().then(||{
            let mut signed_generics = info_sg.clone();
            let signed_where = signed_generics.make_where_clause();
            signed_where.predicates.push(parse_quote! {
                #field_ty: #macro_prelude::SignedAccount<#info_lifetime>
            });
            signed_where.predicates.push(self_single_bound.clone());
            quote! {
                #[automatically_derived]
                impl #info_sg_impl #macro_prelude::SignedAccount<#info_lifetime> for #ident #ty_generics #signed_where {
                    fn signer_seeds(&self) -> Option<Vec<&[u8]>> {
                        <#field_ty as #macro_prelude::SignedAccount<#info_lifetime>>::signer_seeds(&self.#field_name)
                    }
                }
            }
        });

        let writable_account = args.skip_writable_account.not().then(||{
            let mut writable_generics = info_sg.clone();
            let writable_where = writable_generics.make_where_clause();
            writable_where.predicates.push(parse_quote! {
                #field_ty: #macro_prelude::WritableAccount<#info_lifetime>
            });
            writable_where.predicates.push(self_single_bound.clone());
            quote! {
                #[automatically_derived]
                impl #info_sg_impl #macro_prelude::WritableAccount<#info_lifetime> for #ident #ty_generics #writable_where {}
            }
        });

        let has_program_account = args.skip_has_program_account.not().then(||{
            let mut program_generics = single_generics.clone();
            let program_where = program_generics.make_where_clause();
            program_where.predicates.push(parse_quote! {
                #field_ty: #macro_prelude::HasProgramAccount
            });
            quote! {
                #[automatically_derived]
                impl #sg_impl #macro_prelude::HasProgramAccount for #ident #ty_generics #program_where {
                    type ProgramAccount = <#field_ty as #macro_prelude::HasProgramAccount>::ProgramAccount;
                }
            }
        });

        let has_owner_program = args.skip_has_owner_program.not().then(||{
            let mut owner_generics = single_generics.clone();
            let owner_where = owner_generics.make_where_clause();
            owner_where.predicates.push(parse_quote! {
                #field_ty: #macro_prelude::HasOwnerProgram
            });
            quote! {
                #[automatically_derived]
                impl #sg_impl #macro_prelude::HasOwnerProgram for #ident #ty_generics #owner_where {
                    type OwnerProgram = <#field_ty as #macro_prelude::HasOwnerProgram>::OwnerProgram;
                }
            }
        });

        let has_seeds = args.skip_has_seeds.not().then(||{
            let mut seeds_generics = single_generics.clone();
            let seeds_where = seeds_generics.make_where_clause();
            seeds_where.predicates.push(parse_quote! {
                #field_ty: #macro_prelude::HasSeeds
            });
            quote! {
                #[automatically_derived]
                impl #sg_impl #macro_prelude::HasSeeds for #ident #ty_generics #seeds_where {
                    type Seeds = <#field_ty as #macro_prelude::HasSeeds>::Seeds;
                }
            }
        });

        let can_set_seeds = args.skip_can_set_seeds.not().then(||{
            let mut set_seeds_generics = info_gen_sg.clone();
            let set_seeds_where = set_seeds_generics.make_where_clause();
            set_seeds_where.predicates.push(parse_quote! {
                #field_ty: #macro_prelude::CanSetSeeds<#info_lifetime, #function_generic_type>
            });
            set_seeds_where.predicates.push(self_single_bound.clone());
            quote! {
                #[automatically_derived]
                impl #info_gen_sg_impl #macro_prelude::CanSetSeeds<#info_lifetime, #new_generic> for #ident #ty_generics #set_seeds_where {
                    fn set_seeds(&mut self, arg: &#new_generic, syscalls: &mut impl #macro_prelude::SyscallInvoke<#info_lifetime>) -> #result<()> {
                        <#field_ty as #macro_prelude::CanSetSeeds<#info_lifetime, #new_generic>>::set_seeds(&mut self.#field_name, arg, syscalls)
                    }
                }
            }
        });

        let can_init_account = args.skip_can_init_account.not().then(||{
            let mut init_generics = info_gen_sg.clone();
            let init_where = init_generics.make_where_clause();
            init_where.predicates.push(parse_quote! {
                #field_ty: #macro_prelude::CanInitAccount<#info_lifetime, #new_generic>
            });
            quote! {
                #[automatically_derived]
                impl #info_gen_sg_impl #macro_prelude::CanInitAccount<#info_lifetime, #new_generic> for #ident #ty_generics #init_where {
                    fn init(
                        &mut self,
                        arg: #new_generic,
                        syscalls: &mut impl #macro_prelude::SyscallInvoke<#info_lifetime>,
                        account_seeds: Option<Vec<&[u8]>>,
                    ) -> #result<()> {
                        <#field_ty as #macro_prelude::CanInitAccount<#info_lifetime, #new_generic>>::init(&mut self.#field_name, arg, syscalls, account_seeds)
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
            #can_set_seeds
            #can_init_account
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

    #[cfg(feature = "idl")]
    let idls = idl::idls(step_input);
    #[cfg(not(feature = "idl"))]
    let idls = Vec::<TokenStream>::new();

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

    let account_set_impl = account_set_struct_args.skip_default_account_set.not().then(|| {
        quote! {
            #[automatically_derived]
            impl #other_impl_generics #account_set<#info_lifetime> for #ident #ty_generics #other_where_clause {
                fn set_account_cache(
                    &mut self,
                    syscalls: &mut impl #macro_prelude::SyscallAccountCache<#info_lifetime>,
                ) {
                    #set_account_caches
                    #(<#field_type as #account_set<#info_lifetime>>::set_account_cache(&mut self.#field_name, syscalls);)*
                }
            }
        }
    });

    quote! {
        #account_set_impl

        #(#decodes)*
        #(#validates)*
        #(#cleanups)*
        #(#idls)*

        #single_account_set_impls
    }
}
