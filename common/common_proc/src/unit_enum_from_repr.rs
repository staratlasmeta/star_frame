use crate::get_crate_name;
use proc_macro2::TokenStream;
use proc_macro_error::*;
use quote::quote;
use syn::*;

pub fn unit_enum_from_repr_impl(derive_input: DeriveInput) -> TokenStream {
    let crate_name = get_crate_name();

    let repr = if let Some(repr) = derive_input
        .attrs
        .iter()
        .find(|attr| attr.path.get_ident().map_or(false, |ident| ident == "repr"))
    {
        let ty: Type = repr.parse_args().unwrap();
        match &ty {
            Type::Path(path)
                if path.path.segments.len() == 1
                    && path.path.segments[0]
                        .ident
                        .to_string()
                        .chars()
                        .next()
                        .map_or(false, |c| c == 'u') =>
            {
                ty
            }
            ty => abort!(
                ty,
                "UnitEnumFromRepr can only be derived for enums with a `u*` repr attribute"
            ),
        }
    } else {
        abort_call_site!(
            "UnitEnumFromRepr can only be derived for enums with a `u*` repr attribute"
        )
    };
    let input = match derive_input.data {
        Data::Enum(input) => input,
        Data::Struct(_) | Data::Union(_) => {
            // TODO: Allow ZST fields. Maybe just explicitly phantom data.
            abort_call_site!("UnitEnumFromRepr can only be derived for enums")
        }
    };

    let mut variants = Vec::new();
    let mut values = Vec::new();
    for variant in input.variants {
        variants.push(variant.ident);
        values.push(variant.discriminant.as_ref().map_or_else(
            || {
                values
                    .last()
                    .map_or_else(|| quote! { 0 }, |value| quote! { #value + 1 })
            },
            |(_, expr)| quote! { #expr },
        ));
    }

    let ident = derive_input.ident;
    quote! {
        impl #crate_name::UnitEnumFromRepr for #ident {
            type Repr = #repr;

            fn from_repr(repr: Self::Repr) -> ::std::result::Result<Self, Self::Repr> {
                match repr {
                    #(v if v == #values => Ok(Self::#variants),)*
                    v => Err(v),
                }
            }

            fn into_repr(self) -> Self::Repr {
                self as #repr
            }
        }

        impl std::fmt::Display for #ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }
    }
}
