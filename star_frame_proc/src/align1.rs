use crate::util::{get_crate_name, get_repr, IntegerRepr};
use proc_macro2::TokenStream;
use proc_macro_error2::abort;
use quote::quote;
use syn::{
    parse_quote, punctuated::Punctuated, Data, DataStruct, DataUnion, DeriveInput, Fields, Token,
    Variant,
};

pub fn derive_align1_impl(derive_input: DeriveInput) -> TokenStream {
    let crate_name = get_crate_name();
    match derive_input.data.clone() {
        Data::Struct(DataStruct { fields, .. }) => {
            derive_align1_for_struct(fields, derive_input, &crate_name)
        }
        Data::Union(DataUnion { fields, .. }) => {
            derive_align1_for_struct(Fields::Named(fields), derive_input, &crate_name)
        }
        Data::Enum(e) => derive_align1_for_enum(e.variants, derive_input, &crate_name),
    }
}

fn derive_align1_for_struct(
    fields: Fields,
    derive_input: DeriveInput,
    crate_name: &TokenStream,
) -> TokenStream {
    let repr = get_repr(&derive_input.attrs);
    let ident = derive_input.ident;
    let mut gen = derive_input.generics;
    let wc = gen.make_where_clause();
    if !repr.is_packed() {
        for field in fields {
            let ty = field.ty;
            wc.predicates
                .push(parse_quote!(#ty: #crate_name::align1::Align1));
        }
    }
    let (impl_gen, type_gen, where_clause) = gen.split_for_impl();

    quote! {
        unsafe impl #impl_gen #crate_name::align1::Align1 for #ident #type_gen #where_clause {}
    }
}

fn derive_align1_for_enum(
    variants: Punctuated<Variant, Token![,]>,
    derive_input: DeriveInput,
    crate_name: &TokenStream,
) -> TokenStream {
    let repr = get_repr(&derive_input.attrs);
    if repr.repr.as_integer() != Some(IntegerRepr::U8) {
        abort!(derive_input, "Align1 requires repr(u8) for enums");
    }

    let ident = derive_input.ident;
    let (impl_gen, type_gen, where_clause) = derive_input.generics.split_for_impl();

    for variant in &variants {
        if variant.fields != Fields::Unit {
            if !derive_input.generics.params.is_empty() {
                abort!(
                    variant.fields,
                    "Align1 does not support generic enums with data"
                );
            }

            return quote! {
                unsafe impl #impl_gen #crate_name::align1::Align1 for #ident #where_clause {}
                #crate_name::static_assertions::assert_eq_align!(#ident, u8);
            };
        }
    }
    quote! {
        unsafe impl #impl_gen #crate_name::align1::Align1 for #ident #type_gen #where_clause {}
    }
}
