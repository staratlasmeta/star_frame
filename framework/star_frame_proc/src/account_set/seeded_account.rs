use crate::util;
use crate::util::Paths;
use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::abort;
use quote::{quote, ToTokens};
use syn::{Attribute, Data, DeriveInput, LitStr, Visibility};

#[allow(dead_code)]
struct StrippedDeriveInput {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
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

    let field_names = data_struct
        .fields
        .iter()
        .map(|field| field.ident.as_ref().unwrap())
        .collect::<Vec<_>>();

    let out = quote! {
        impl GetSeeds for #ident {
            fn seeds(&self) -> Vec<&[u8]> {
                vec![
                    #(
                        self.#field_names.seed(),
                    )*
                ]
            }
        }
    };

    out
}
