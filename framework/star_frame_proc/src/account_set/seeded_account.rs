use crate::util::Paths;
use easy_proc::find_attr;
use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::abort;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Expr, Generics, Visibility};

#[allow(dead_code)]
struct StrippedDeriveInput {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    generics: Generics,
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
    let Paths { get_seeds, .. } = paths;

    let ident = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    let seed_struct_args = find_attr(&input.attrs, &Ident::new("seed_const", Span::call_site()));

    let opt_seed_expr = seed_struct_args.map(|attr| {
        attr.parse_args::<Expr>()
            .expect("Failed to parse seed expression")
    });

    let field_names = data_struct
        .fields
        .iter()
        .map(|field| field.ident.as_ref().unwrap())
        .collect::<Vec<_>>();

    let seeds_content = match opt_seed_expr {
        Some(seed_expr) => quote! {
            #seed_expr,
            #(
                self.#field_names.seed()
            ),*
        },
        None => quote! {
            #(
                self.#field_names.seed()
            ),*
        },
    };

    let out = quote! {
        impl #impl_generics #get_seeds for #ident #type_generics #where_clause {
            fn seeds(&self) -> Vec<&[u8]> {
                vec![#seeds_content]
            }
        }
    };

    out
}
