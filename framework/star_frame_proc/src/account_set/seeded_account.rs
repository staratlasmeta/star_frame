use crate::util::Paths;
use easy_proc::find_attr;
use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::abort;
use quote::quote;
use syn::{Data, DeriveInput, Expr};

pub fn derive_get_seeds_impl(input: DeriveInput) -> TokenStream {
    let data_struct = match input.data {
        Data::Struct(s) => s,
        Data::Enum(e) => abort!(e.enum_token, "GetSeeds cannot be derived for enums"),
        Data::Union(u) => abort!(u.union_token, "GetSeeds cannot be derived for unions"),
    };

    let Paths { get_seeds, .. } = Paths::default();

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
