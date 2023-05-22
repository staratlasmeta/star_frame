use proc_macro2::Span;
use proc_macro_error::*;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::*;

struct DerivesList(Punctuated<Ident, Token![,]>);
impl Parse for DerivesList {
    fn parse(input: ParseStream) -> Result<Self> {
        #[allow(clippy::redundant_closure_for_method_calls)]
        Ok(Self(input.parse_terminated(|input| input.parse())?))
    }
}

pub fn enum_refs_impl(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let derives = syn::parse::<DerivesList>(args)
        .unwrap_or_else(|e| abort_call_site!("Invalid derives list: {}", e))
        .0
        .into_iter()
        .collect::<Vec<_>>();

    let input2 = proc_macro2::TokenStream::from(input.clone());
    let item_enum = parse_macro_input!(input as ItemEnum);
    let mut generics = item_enum.generics;
    let vis = item_enum.vis;

    let mut has_non_unit = false;
    let mut variant_idents = Vec::with_capacity(item_enum.variants.len());
    let mut variant_types = Vec::with_capacity(item_enum.variants.len());
    for variant in item_enum.variants {
        match variant.fields {
            Fields::Named(_) => abort!(variant.ident, "Named variants are not supported"),
            Fields::Unnamed(fields) => {
                if fields.unnamed.len() > 1 {
                    abort!(
                        variant.ident,
                        "Unnamed variants with more than one field are not supported"
                    );
                }
                if fields.unnamed.is_empty() {
                    abort!(
                        variant.ident,
                        "Unnamed variants with no fields are not supported"
                    );
                }
                has_non_unit = true;
                variant_types.push(Some(fields.unnamed.into_iter().next().unwrap().ty));
            }
            Fields::Unit => {
                variant_types.push(None);
            }
        }
        variant_idents.push(variant.ident);
    }

    if has_non_unit {
        generics.params.push(syn::parse_str("'__a").unwrap());
    }
    let (impl_generics, _ty_generics, where_clause) = generics.split_for_impl();

    let ref_variants = variant_idents
        .iter()
        .zip(&variant_types)
        .map(|(ident, ty)| {
            let docs = LitStr::new(
                &format!("Ref to [`{}::{}`]", item_enum.ident, ident),
                Span::call_site(),
            );
            match ty {
                None => quote! {
                    #[doc = #docs]
                    #ident
                },
                Some(ty) => quote! {
                    #[doc = #docs]
                    #ident(Ref<'__a, #ty>)
                },
            }
        });

    let ref_mut_variants = variant_idents
        .iter()
        .zip(&variant_types)
        .map(|(ident, ty)| {
            let docs = LitStr::new(
                &format!("Ref to [`{}::{}`]", item_enum.ident, ident),
                Span::call_site(),
            );
            match ty {
                None => quote! {
                    #[doc = #docs]
                    #ident
                },
                Some(ty) => quote! {
                    #[doc = #docs]
                    #ident(RefMut<'__a, #ty>)
                },
            }
        });

    let ref_ident = format_ident!("{}Ref", item_enum.ident);
    let ref_mut_ident = format_ident!("{}RefMut", item_enum.ident);
    let ref_doc = LitStr::new(
        &format!("A reference to [`{}`]", item_enum.ident),
        Span::call_site(),
    );
    let ref_mut_doc = LitStr::new(
        &format!("A mutable reference to [`{}`]", item_enum.ident),
        Span::call_site(),
    );
    (quote! {
        #input2

        #[doc = #ref_doc]
        #[derive(#(#derives),*)]
        #vis enum #ref_ident #impl_generics #where_clause {
            #(#ref_variants,)*
        }

        #[doc = #ref_mut_doc]
        #[derive(#(#derives),*)]
        #vis enum #ref_mut_ident #impl_generics #where_clause {
            #(#ref_mut_variants,)*
        }
    })
    .into()
}
