use crate::account_set::generics::AccountSetGenerics;
use crate::account_set::struct_impl::cleanup::cleanups;
use crate::account_set::struct_impl::validate::validates;
use crate::account_set::{AccountSetStructArgs, StrippedDeriveInput};
use crate::util::Paths;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{DataStruct, Index};

mod cleanup;
mod decode;
mod idl;
mod validate;

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
        idl_account_set_def,
        idl_account_set,
        idl_account_set_struct_field,
        account_set_id,
        ..
    } = &paths;

    #[cfg(feature = "idl")]
    let account_set_to_idl = &paths.account_set_to_idl;
    #[cfg(feature = "idl")]
    let idl_definition = &paths.idl_definition;

    let ident = &input.ident;

    let (other_impl_generics, _, other_where_clause) = other_generics.split_for_impl();
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
        use crate::util;
        use proc_macro2::Span;
        use std::iter::once;
        use syn::{Expr, LitStr, Type, WhereClause};

        let struct_docs = LitStr::new(&util::get_docs(&input.attrs), Span::call_site());
        let ident_str = LitStr::new(&ident.to_string(), Span::call_site());
        let field_docs: Vec<LitStr> = data_struct
            .fields
            .iter()
            .map(|field| LitStr::new(&util::get_docs(&field.attrs), Span::call_site()))
            .collect();
        let field_str = field_name
            .iter()
            .map(|field_name| LitStr::new(&field_name.to_string(), Span::call_site()));
        idls = once({
            let idl_type: Type = syn::parse_quote!(());
            let extra_where_clause: Option<WhereClause> = None;
            let idl_args: Vec<Expr> = vec![syn::parse_quote!(()); field_type.len()];

            let mut generics = other_generics.clone();
            if let Some(extra_where_clause) = extra_where_clause {
                generics
                    .make_where_clause()
                    .predicates
                    .extend(extra_where_clause.predicates);
            }
            let (_, ty_generics, _) = main_generics.split_for_impl();
            let (impl_generics, _, where_clause) = generics.split_for_impl();
            let field_name = field_name.iter().map(|field_name| format_ident!("__{}", field_name.to_string())).collect::<Vec<_>>();
            quote! {
                #[automatically_derived]
                impl #impl_generics #account_set_to_idl<#info_lifetime, #idl_type> for #ident #ty_generics #where_clause {
                    fn account_set_to_idl(
                        idl_definition: &mut #idl_definition,
                        arg: #idl_type,
                    ) -> #result<#idl_account_set_def> {
                        #(let #field_name = <#field_type as #account_set_to_idl<#info_lifetime, _>>::account_set_to_idl(idl_definition, #idl_args)?;)*
                        idl_definition.account_sets.insert(
                            #ident_str.to_string(),
                            #idl_account_set {
                                name: #ident_str.to_string(),
                                description: #struct_docs.to_string(),
                                type_generics: vec![],
                                account_generics: vec![],
                                def: #idl_account_set_def::Struct(vec![#(
                                    #idl_account_set_struct_field {
                                        name: #field_str.to_string(),
                                        description: #field_docs.to_string(),
                                        path: #field_str.to_string(),
                                        account_set: #field_name,
                                        extension_fields: Default::default(),
                                    },
                                )*]),
                                extension_fields: Default::default(),
                            },
                        );
                        Ok(#idl_account_set_def::AccountSet(#account_set_id {
                            namespace: None,
                            account_set_id: #ident_str.to_string(),
                            provided_type_generics: vec![],
                            provided_account_generics: vec![],
                            extension_fields: Default::default(),
                        }))
                    }
                }
            }
        })
        .collect();
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
