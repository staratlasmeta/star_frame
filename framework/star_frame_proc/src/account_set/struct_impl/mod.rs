use crate::account_set::generics::AccountSetGenerics;
use crate::account_set::struct_impl::cleanup::cleanups;
use crate::account_set::struct_impl::validate::validates;
use crate::account_set::{AccountSetStructArgs, StrippedDeriveInput};
use crate::util::Paths;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{bracketed, token, DataStruct, Ident, Index, Token};

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
        crate_name,
        account_info,
        result,
        account_set,
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

    let field_name = data_struct
        .fields
        .iter()
        .enumerate()
        .map(|(index, field)| {
            field
                .ident
                .as_ref()
                .map(ToTokens::to_token_stream)
                .unwrap_or_else(|| Index::from(index).into_token_stream())
        })
        .collect::<Vec<_>>();
    let field_type = data_struct
        .fields
        .iter()
        .map(|field| &field.ty)
        .collect::<Vec<_>>();

    let decodes = decode::decodes(
        &paths,
        &input,
        &account_set_struct_args,
        &account_set_generics,
        &data_struct,
        &field_name,
        &field_type,
    );
    let validates = validates(
        &paths,
        &input,
        &account_set_struct_args,
        &account_set_generics,
        &data_struct,
        &field_name,
        &field_type,
    );
    let cleanups = cleanups(
        &paths,
        &input,
        &account_set_struct_args,
        &account_set_generics,
        &data_struct,
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
            &data_struct,
            &field_name,
            &field_type,
        );
    }
    #[cfg(not(feature = "idl"))]
    {
        idls = Vec::new();
    }

    quote! {
        #[automatically_derived]
        impl #other_impl_generics #account_set<#info_lifetime> for #ident #ty_generics #other_where_clause {
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
