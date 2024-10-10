use crate::account_set::generics::AccountSetGenerics;
use crate::account_set::struct_impl::cleanup::cleanups;
use crate::account_set::struct_impl::decode::DecodeFieldTy;
use crate::account_set::struct_impl::validate::validates;
use crate::account_set::{AccountSetStructArgs, StrippedDeriveInput};
use crate::util::Paths;
use easy_proc::{find_attr, ArgumentList};
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{bracketed, token, DataStruct, Field, Ident, Index, Token};

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
    system_program: bool,
    #[argument(presence)]
    funder: bool,
    #[argument(presence)]
    recipient: bool,
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
        function_lifetime,
        function_generic_type,
        ..
    } = &account_set_generics;

    let Paths {
        account_info,
        account_set,
        crate_name,
        macro_prelude,
        result,
        ..
    } = &paths;

    let ident = &input.ident;

    let mut generics = other_generics.clone();
    if let Some(extra_generics) = &account_set_struct_args.generics {
        generics.params.extend(extra_generics.params.clone());
        if let Some(extra_where_clause) = &extra_generics.where_clause {
            generics
                .make_where_clause()
                .predicates
                .extend(extra_where_clause.predicates.clone());
        }
    }
    let (other_impl_generics, _, other_where_clause) = generics.split_for_impl();

    let (_, ty_generics, _) = main_generics.split_for_impl();

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

    let field_name = data_struct
        .fields
        .iter()
        .enumerate()
        .map(resolve_field_name)
        .collect::<Vec<_>>();

    let find_field_name =
        |name: &str, is_active: fn(AccountSetFieldAttrs) -> bool| -> Option<TokenStream> {
            let mut fields = data_struct
                .fields
                .iter()
                .enumerate()
                .filter(|field| {
                    find_attr(&field.1.attrs, &paths.account_set_ident)
                        .map(AccountSetFieldAttrs::parse_arguments)
                        .map(is_active)
                        .unwrap_or_default()
                })
                .collect::<Vec<_>>();
            if fields.len() > 1 {
                abort!(
                    fields[1].1,
                    format!("Only one field can be marked as {name}")
                );
            }
            fields.pop().map(|(index, _)| field_name[index].clone())
        };

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

    let decodes = decode::decodes(
        &paths,
        &input,
        &account_set_struct_args,
        &account_set_generics,
        &data_struct,
        &field_name,
        &decode_types,
    );

    let fields = data_struct
        .fields
        .iter()
        .filter(filter_skip)
        .collect::<Vec<_>>();
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

    let validates = validates(
        &paths,
        &input,
        &account_set_struct_args,
        &account_set_generics,
        &fields,
        &field_name,
        &field_type,
    );
    let cleanups = cleanups(
        &paths,
        &input,
        &account_set_struct_args,
        &account_set_generics,
        &fields,
        &field_name,
        &field_type,
    );

    let idls: Vec<TokenStream>;
    #[cfg(feature = "idl")]
    {
        idls = idl::idls(
            &paths,
            &input,
            &account_set_struct_args,
            &account_set_generics,
            &fields,
            &field_name,
            &field_type,
        );
    }
    #[cfg(not(feature = "idl"))]
    {
        idls = Vec::new();
    }

    let set_account_caches = {
        let set_system =
            find_field_name("system_program", |args| args.system_program).map(|field_name| {
                quote! {
                    if syscalls.get_system_program().is_none() {
                        syscalls.set_system_program(self.#field_name.clone());
                    }
                }
            });
        let set_funder = find_field_name("funder", |args| args.funder).map(|field_name| {
            quote! {
                if syscalls.get_funder().is_none() {
                    syscalls.set_funder(&self.#field_name);
                }
            }
        });
        let set_recipient = find_field_name("recipient", |args| args.recipient).map(|field_name| {
            quote! {
                if syscalls.get_recipient().is_none() {
                    syscalls.set_recipient(&self.#field_name);
                }
            }
        });
        quote! {
            #set_system
            #set_funder
            #set_recipient
        }
    };

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

            fn try_to_accounts<#function_lifetime, #function_generic_type>(
                &#function_lifetime self,
                mut add_account: impl FnMut(&#function_lifetime #account_info<#info_lifetime>) -> #result<(), #function_generic_type>,
            ) -> #result<(), #function_generic_type>
            where
                #info_lifetime: #function_lifetime,
            {
                #(<#field_type as #account_set<#info_lifetime>>::try_to_accounts(&self.#field_name, &mut add_account)?;)*
                Ok(())
            }

            fn to_accounts<#function_lifetime>(
                &#function_lifetime self,
                mut add_account: impl FnMut(&#function_lifetime #account_info<#info_lifetime>),
            )
            where
                #info_lifetime: #function_lifetime,
            {
                #(<#field_type as #account_set<#info_lifetime>>::to_accounts(&self.#field_name, &mut add_account);)*
            }

            fn to_account_metas(&self, mut add_account_meta: impl FnMut(#crate_name::solana_program::instruction::AccountMeta)) {
                #(<#field_type as #account_set<#info_lifetime>>::to_account_metas(&self.#field_name, &mut add_account_meta);)*
            }
        }

        #(#decodes)*
        #(#validates)*
        #(#cleanups)*
        #(#idls)*
    }
}
