use crate::{
    account_set::{
        generics::AccountSetGenerics, struct_impl::decode::DecodeFieldTy, AccountSetStructArgs,
        SingleAccountSetFieldArgs, StrippedDeriveInput,
    },
    util::{
        combine_gen, ignore_cfg_module, make_struct, new_generic, new_lifetime,
        recurse_type_operator, GetGenerics, Paths,
    },
};
use core::ops::Not;
use easy_proc::{find_attr, ArgumentList};
use itertools::Itertools;
use proc_macro2::TokenStream;
use proc_macro_error2::abort;
use quote::{format_ident, quote, ToTokens};
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    token, DataStruct, Field, Generics, Ident, Index, Lifetime, Token, Type,
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

#[derive(ArgumentList, Debug, Clone, Default)]
struct AccountSetFieldAttrs {
    skip: Option<TokenStream>,
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

// TODO: Refactor this into multiple methods. Its horrible
// TODO: Also refactor all the lifecycle methods into a more generic system. So much duplication. I hate it
pub(super) fn derive_account_set_impl_struct(
    paths: Paths,
    data_struct: DataStruct,
    account_set_struct_args: AccountSetStructArgs,
    input: StrippedDeriveInput,
    account_set_generics: AccountSetGenerics,
) -> TokenStream {
    let AccountSetGenerics { main_generics, .. } = &account_set_generics;

    Paths!(account_info, prelude, result, clone, debug, maybe_uninit);

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
        let (sg_impl, ..) = sg_impl.split_for_impl();

        let field_ty = &field.ty;

        let meta = args.meta.map_or_else(
            || {
                let signer = args.signer.then(|| quote!(signer: true,));
                let writable = args.writable.then(|| quote!(writable: true,));
                quote! {
                    #prelude::SingleSetMeta {
                        #signer
                        #writable
                        ..<#field_ty as #prelude::SingleAccountSet>::meta()
                    }
                }
            },
            |expr| {
                if args.signer || args.writable {
                    abort!(
                        expr,
                        "`signer` or `writable` cannot be used with custom `meta`"
                    );
                }
                quote!(#expr)
            }
        );

        let single_set_gen = combine_gen!(single_generics; where #field_ty: for<'__a> #prelude::SingleAccountSet);
        let (_, _, single_set_wc) = single_set_gen.split_for_impl();

        let client_set_gen = combine_gen!(single_generics; where #field_ty: for<'__a> #prelude::ClientAccountSet + #prelude::SingleAccountSet);
        let (_, _, client_set_wc) = client_set_gen.split_for_impl();

        let cpi_set_gen = combine_gen!(single_generics; where #field_ty: for<'__a> #prelude::CpiAccountSet + #prelude::SingleAccountSet);
        let (_, _, cpi_set_wc) = cpi_set_gen.split_for_impl();

        let cpi_set_impl = account_set_struct_args.skip_cpi_account_set.not().then(|| {
            let lt = new_lifetime(&cpi_set_gen, None);
            quote! {
                #[automatically_derived]
                unsafe impl #sg_impl #prelude::CpiAccountSet for #ident #ty_generics #cpi_set_wc {
                    type CpiAccounts = #prelude::AccountInfo;
                    type ContainsOption = <#field_ty as #prelude::CpiAccountSet>::ContainsOption;
                    type AccountLen = #prelude::typenum::U1;

                    #[inline]
                    fn to_cpi_accounts(&self) -> Self::CpiAccounts {
                        *self.account_info()
                    }
                    #[inline(always)]
                    fn write_account_infos<#lt>(
                        program: Option<&#lt #prelude::AccountInfo>,
                        accounts: &#lt #prelude::AccountInfo,
                        index: &mut usize,
                        infos: &mut [#maybe_uninit<&#lt #prelude::AccountInfo>],
                    ) -> #prelude::Result<()> {
                        <#prelude::AccountInfo as #prelude::CpiAccountSet>::write_account_infos(program,  accounts, index, infos)
                    }
                    #[inline(always)]
                    fn write_account_metas<'a>(
                        _program_id: &#lt #prelude::Pubkey,
                        accounts: &#lt #prelude::AccountInfo,
                        index: &mut usize,
                        metas: &mut [#maybe_uninit<#prelude::PinocchioAccountMeta<#lt>>],
                    ) {
                        metas[*index] = #maybe_uninit::new(#prelude::PinocchioAccountMeta {
                            pubkey: accounts.key(),
                            is_signer: <Self as #prelude::SingleAccountSet>::meta().signer,
                            is_writable: <Self as #prelude::SingleAccountSet>::meta().writable,
                        });
                        *index += 1;
                    }
                }
            }
        });

        let client_set_impl = account_set_struct_args.skip_client_account_set.not().then(|| {
            quote! {
                #[automatically_derived]
                impl #sg_impl #prelude::ClientAccountSet for #ident #ty_generics #client_set_wc {
                    type ClientAccounts = #prelude::Pubkey;
                    const MIN_LEN: usize = 1;
                    #[inline]
                    fn extend_account_metas(
                        _program_id: &#prelude::Pubkey,
                        accounts: &Self::ClientAccounts,
                        metas: &mut Vec<#prelude::AccountMeta>,
                    ) {
                        metas.push(#prelude::AccountMeta {
                            pubkey: *accounts,
                            is_signer: <Self as #prelude::SingleAccountSet>::meta().signer,
                            is_writable: <Self as #prelude::SingleAccountSet>::meta().writable,
                        });
                    }
                }
            }
        });

        let single = quote! {
            #[automatically_derived]
            impl #sg_impl #prelude::SingleAccountSet for #ident #ty_generics #single_set_wc {
                #[allow(clippy::needless_update)]
                fn meta() -> #prelude::SingleSetMeta {
                    #meta
                }

                #[inline(always)]
                fn account_info(&self) -> &#account_info {
                    <#field_ty as #prelude::SingleAccountSet>::account_info(&self.#field_name)
                }
            }

            #cpi_set_impl
            #client_set_impl
        };

        let signed_account = args.skip_signed_account.not().then(|| {
            let single_gen = combine_gen!(single_generics; where #field_ty: for<'__a> #prelude::SignedAccount);
            let (_, _, wc) = single_gen.split_for_impl();
            quote! {
                #[automatically_derived]
                impl #sg_impl #prelude::SignedAccount for #ident #ty_generics #wc {
                    #[inline]
                    fn signer_seeds(&self) -> Option<Vec<&[u8]>> {
                        <#field_ty as #prelude::SignedAccount>::signer_seeds(&self.#field_name)
                    }
                }
            }
        });

        let writable_account = args.skip_writable_account.not().then(|| {
            let single_gen = combine_gen!(single_generics; where #field_ty: for<'__a> #prelude::WritableAccount);
            let (_, _, wc) = single_gen.split_for_impl();
            quote! {
                #[automatically_derived]
                impl #sg_impl #prelude::WritableAccount for #ident #ty_generics #wc {}
            }
        });

        let has_program_account = args.skip_has_inner_type.not().then(|| {
            let single_gen = combine_gen!(single_generics; where #field_ty: for<'__a> #prelude::HasInnerType);
            let (_, _, wc) = single_gen.split_for_impl();
            quote! {
                #[automatically_derived]
                impl #sg_impl #prelude::HasInnerType for #ident #ty_generics #wc {
                    type Inner = <#field_ty as #prelude::HasInnerType>::Inner;
                }
            }
        });

        let has_owner_program = args.skip_has_owner_program.not().then(|| {
            let single_gen = combine_gen!(single_generics; where #field_ty: for<'__a> #prelude::HasOwnerProgram);
            let (_, _, wc) = single_gen.split_for_impl();
            quote! {
                #[automatically_derived]
                impl #sg_impl #prelude::HasOwnerProgram for #ident #ty_generics #wc {
                    type OwnerProgram = <#field_ty as #prelude::HasOwnerProgram>::OwnerProgram;
                }
            }
        });

        let has_seeds = args.skip_has_seeds.not().then(|| {
            let single_gen = combine_gen!(single_generics; where #field_ty: for<'__a> #prelude::HasSeeds);
            let (_, _, wc) = single_gen.split_for_impl();
            quote! {
                #[automatically_derived]
                impl #sg_impl #prelude::HasSeeds for #ident #ty_generics #wc {
                    type Seeds = <#field_ty as #prelude::HasSeeds>::Seeds;
                }
            }
        });

        let can_init_seeds = args.skip_can_init_seeds.not().then(|| {
            let new_generic = new_generic(&single_generics, None);
            let init_seeds_gen = combine_gen!(single_generics;
                <#new_generic> where
                    #field_ty: for<'__a> #prelude::CanInitSeeds<#new_generic>,
                    Self: #prelude::AccountSetValidate<#new_generic>
            );
            let (impl_gen, _, wc) = init_seeds_gen.split_for_impl();
            quote! {
                #[automatically_derived]
                impl #impl_gen #prelude::CanInitSeeds<#new_generic> for #ident #ty_generics #wc {
                    #[inline]
                    fn init_seeds(&mut self, arg: &#new_generic, ctx: &#prelude::Context) -> #result<()> {
                        <#field_ty as #prelude::CanInitSeeds<#new_generic>>::init_seeds(&mut self.#field_name, arg, ctx)
                    }
                }
            }
        });

        let can_init_account = args.skip_can_init_account.not().then(|| {
            let init_gen = new_generic(&single_generics, None);
            let if_needed = new_generic(&single_generics, Some("IF_NEEDED"));

            let init_account_gen = combine_gen!(single_generics;
                <#init_gen> where #field_ty: #prelude::CanInitAccount<#init_gen>
            );
            let (impl_gen, _, wc) = init_account_gen.split_for_impl();
            quote! {
                #[automatically_derived]
                impl #impl_gen #prelude::CanInitAccount<#init_gen> for #ident #ty_generics #wc {
                    #[inline]
                    fn init_account<const #if_needed: bool>(
                        &mut self,
                        arg: #init_gen,
                        account_seeds: Option<&[&[u8]]>,
                        ctx: &#prelude::Context,
                    ) -> #result<()> {
                        <#field_ty as #prelude::CanInitAccount<#init_gen>>::init_account::<#if_needed>(&mut self.#field_name, arg, account_seeds, ctx)
                    }
                }
            }
        });

        quote! {
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

    let cpi_account_set_impl = (!account_set_struct_args.skip_cpi_account_set
        && single_account_set_impls.is_none())
    .then(|| {
        let cpi_accounts_ident = format_ident!("{trimmed_ident_str}CpiAccounts");
        let (_, self_ty_gen, _) = main_generics.split_for_impl();
        let mut cpi_gen = main_generics.clone();
        let cpi_lt = new_lifetime(&cpi_gen, None);
        let where_clause = cpi_gen.make_where_clause();
        let cpi_set = quote!(#prelude::CpiAccountSet);

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
                    for <#cpi_lt> #ty: #cpi_set
                });
                parse_quote!(#vis #ident #colon_token <#ty as #cpi_set>::CpiAccounts)
            })
            .collect();

        let cpi_accounts_struct = make_struct(&cpi_accounts_ident, &new_fields, main_generics);
        let new_struct_ty_gen = main_generics.split_for_impl().1;

        let struct_members = cpi_accounts_struct.fields.members();

        let lt = new_lifetime(&cpi_gen, None);

        let CpiClauseResult {
            contains_option,
            account_len,
            generics: cpi_gen,
        } = create_cpi_clauses(field_type.as_slice(), &cpi_gen);

        let (impl_gen, _, where_clause) = cpi_gen.split_for_impl();


        quote! {
            #[derive(#clone, #debug)]
            #cpi_accounts_struct

            #[automatically_derived]
            unsafe impl #impl_gen #cpi_set for #ident #self_ty_gen #where_clause {
                type CpiAccounts = #cpi_accounts_ident #new_struct_ty_gen;
                type ContainsOption = #contains_option;
                type AccountLen = #prelude::typenum::Minimum<#account_len, #prelude::DynamicCpiAccountSetLen>;

                #[inline]
                fn to_cpi_accounts(&self) -> Self::CpiAccounts {
                    Self::CpiAccounts {
                        #(#struct_members: <#field_type as #cpi_set>::to_cpi_accounts(&self.#struct_members),)*
                    }
                }

                #[inline(always)]
                fn write_account_infos<#lt>(
                    program: Option<&#lt #prelude::AccountInfo>,
                    accounts: &#lt Self::CpiAccounts,
                    index: &mut usize,
                    infos: &mut [#maybe_uninit<&#lt #prelude::AccountInfo>],
                ) -> #prelude::Result<()> {
                    #(<#field_type as #cpi_set>::write_account_infos(program, &accounts.#field_name, index, infos)?;)*
                    Ok(())
                }
                #[inline(always)]
                fn write_account_metas<#lt>(
                    program_id: &#lt #prelude::Pubkey,
                    accounts: &#lt Self::CpiAccounts,
                    index: &mut usize,
                    metas: &mut [#maybe_uninit<#prelude::PinocchioAccountMeta<#lt>>],
                ) {
                    #(<#field_type as #cpi_set>::write_account_metas(program_id, &accounts.#field_name, index, metas);)*
                }
            }
        }
    });

    let client_account_set_impl = (!account_set_struct_args.skip_client_account_set && single_account_set_impls.is_none()).then(|| {
        let client_accounts_ident = format_ident!("{trimmed_ident_str}ClientAccounts");
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
            #[derive(#clone, #debug)]
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

    let idl_impls = ignore_cfg_module(
        ident,
        "_account_set_to_idl",
        quote! {
            #(#idls)*
        },
    );

    quote! {
        #(#decodes)*
        #(#validates)*
        #(#cleanups)*

        #single_account_set_impls
        #cpi_account_set_impl
        #client_account_set_impl

        #idl_impls
    }
}

#[derive(Debug)]
struct CpiClauseResult {
    generics: Generics,
    contains_option: TokenStream,
    account_len: TokenStream,
}

fn create_cpi_clauses(tys: &[&Type], generics: &impl GetGenerics) -> CpiClauseResult {
    Paths!(prelude);
    let generics = generics.get_generics();
    let cpi_set = quote!(#prelude::CpiAccountSet);
    let (option_gens, len_gens): (Vec<_>, Vec<_>) = (0..tys.len())
        .map(|i| (format_ident!("__O{}", i + 1), format_ident!("__L{}", i + 1)))
        .unzip();
    let hrtb = new_lifetime(generics, None);
    let where_clauses = tys.iter().enumerate().map(|(i, ty)| {
        let option_gen = &option_gens[i];
        let len_gen = &len_gens[i];
        quote! {
            for <#hrtb> #prelude::CpiConstWrapper<#ty, #i>: #cpi_set<ContainsOption = #option_gen, AccountLen = #len_gen>
        }
    });

    let (mut option_clauses, contains_option) = create_nested_clauses(
        &hrtb,
        &option_gens,
        &quote!(::core::ops::BitOr),
        &quote!(#prelude::typenum::Or),
        &quote!(#prelude::typenum::False),
    );
    option_clauses.push(quote!(for<#hrtb> #contains_option: #prelude::typenum::Bit));

    let (mut account_len_clauses, account_len) = create_nested_clauses(
        &hrtb,
        &len_gens,
        &quote!(::core::ops::Add),
        &quote!(#prelude::typenum::Sum),
        &quote!(#prelude::typenum::U0),
    );
    account_len_clauses.push(quote!(for<#hrtb> #account_len: #prelude::typenum::Min<#prelude::DynamicCpiAccountSetLen, Output: #prelude::typenum::Unsigned>));

    let where_clauses = where_clauses
        .chain(option_clauses)
        .chain(account_len_clauses)
        .collect_vec();

    let new_gen =
        combine_gen!(*generics; <#(#option_gens,)* #(#len_gens,)*> where #(#where_clauses),*);

    CpiClauseResult {
        generics: new_gen,
        contains_option,
        account_len,
    }
}

fn create_nested_clauses(
    hrtb: &Lifetime,
    idents: &[Ident],
    wrapper: &TokenStream,
    op: &TokenStream,
    default: &TokenStream,
) -> (Vec<TokenStream>, TokenStream) {
    let clauses = (1..idents.len())
        .map(|i| {
            let main_ident = &idents[i - 1];
            let inners = recurse_type_operator(op, &idents[i..], &quote!());
            quote! {
                for<#hrtb> #main_ident: #wrapper<#inners>
            }
        })
        .collect();
    let default = recurse_type_operator(op, idents, default);
    (clauses, default)
}
