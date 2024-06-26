use crate::account_set::AccountSetStructArgs;
use crate::util;
use crate::util::Paths;
use easy_proc::{find_attr, ArgumentList};
use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::abort;
use quote::{quote, ToTokens};
use syn::{Attribute, Data, DeriveInput, Expr, Generics, LitStr, Visibility};

#[allow(dead_code)]
struct StrippedDeriveInput {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    generics: Generics,
}

// TODO - Rename "id" to something representative of string const. Support paths also. Is there a way to do this without requiring "id" to be passed in?
#[derive(ArgumentList, Default)]
pub struct GetSeedsStructArgs {
    id: Option<Expr>,
}

pub fn derive_get_seeds_impl(input: DeriveInput) -> TokenStream {
    let out = match input.data {
        Data::Struct(s) => derive_get_seeds_impl_struct(
            Paths::default(),
            s,
            StrippedDeriveInput {
                attrs: input.attrs,
                vis: input.vis,
                ident: input.ident,
                generics: input.generics,
            },
        ),
        Data::Enum(e) => abort!(e.enum_token, "GetSeeds cannot be derived for enums"),
        Data::Union(u) => abort!(u.union_token, "GetSeeds cannot be derived for unions"),
    };

    out
}

fn derive_get_seeds_impl_struct(
    paths: Paths,
    data_struct: syn::DataStruct,
    input: StrippedDeriveInput,
) -> TokenStream {
    let Paths { .. } = paths;

    let ident = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    let seed_struct_args = find_attr(&input.attrs, &Ident::new("seed_const", Span::call_site()))
        .map(GetSeedsStructArgs::parse_arguments)
        .unwrap_or_default();

    let seed_const = seed_struct_args.id;

    let field_names = data_struct
        .fields
        .iter()
        .map(|field| field.ident.as_ref().unwrap())
        .collect::<Vec<_>>();

    let out = if let Some(seed_const) = seed_const {
        quote! {
            impl #impl_generics GetSeeds for #ident #type_generics #where_clause {
                fn seeds(&self) -> Vec<&[u8]> {
                    vec![
                        #seed_const,
                        #(
                            self.#field_names.seed()
                        ),*
                    ]
                }
            }
        }
    } else {
        quote! {
            impl #impl_generics GetSeeds for #ident #type_generics #where_clause {
                fn seeds(&self) -> Vec<&[u8]> {
                    vec![
                        #(
                            self.#field_names.seed()
                        ),*
                    ]
                }
            }
        }
    };

    out
}
