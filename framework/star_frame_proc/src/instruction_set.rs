use crate::util::{verify_repr, Paths};
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use syn::{parse_quote, Field, Fields, FieldsUnnamed, ItemEnum, Lifetime, Type};

pub fn instruction_set_impl(item: ItemEnum, _args: TokenStream) -> TokenStream {
    let vis = &item.vis;
    let ident = &item.ident;
    let attrs = &item.attrs;
    let a_lifetime: Lifetime = parse_quote! { '__a };

    let Paths { instruction, instruction_set, .. } = Paths::default();

    if !item.generics.params.is_empty() {
        abort!(item.generics, "Generics are unsupported");
    }
    let reprs = verify_repr(attrs, [], true, false);
    if reprs.len() > 1 {
        abort!(reprs, "Invalid enum reprs")
    }
    let forced_repr;
    let repr: Type = if reprs.is_empty() {
        forced_repr = quote! { #[repr(u8)] };
        parse_quote! { u8 }
    } else {
        forced_repr = quote! {};
        let repr = &reprs[0];
        if repr == "C" {
            abort!(repr, "#[repr(C)] is unsupported");
        }
        parse_quote! { #repr }
    };

    let mut variant_discriminants = Vec::new();
    let mut variants = Vec::new();
    let mut last_discriminant = None
    for variant in &item.variants {
        let variant_ident = &variant.ident;
        let variant_attrs = &variant.attrs;
        let discriminant = variant
            .discriminant
            .as_ref()
            .map(|(_, expr)| expr.clone())
            .unwrap_or_else(|| match last_discriminant {
                Some(last_discriminant) => parse_quote! { (#last_discriminant) + 1 },
                None => parse_quote! { 0 },
            });
        last_discriminant = Some(discriminant.clone());
        match &variant.fields {
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                if unnamed.len() != 1 {
                    abort!(unnamed, "Only a single unnamed field is supported.");
                }
                let Field {
                    attrs: ty_attrs,
                    ty,
                    ..
                } = &unnamed[0];
                variant_discriminants.push(discriminant);
                variants.push(quote! {
                    #(#variant_attrs)*
                    #variant_ident(#(#ty_attrs)* <#ty as #instruction>::SelfData<#a_lifetime>)
                })
            }
            Fields::Unit | Fields::Named(_) => {
                abort!(variant.fields, "Only a single unnamed field is supported.")
            }
        }
    }
    quote! {
        #(#attrs)*
        #forced_repr
        #vis enum #ident<#a_lifetime> {
            #(#variants = #variant_discriminants)*
        }
        impl<#a_lifetime> #instruction_set for #ident<#a_lifetime> {}
    }
}
