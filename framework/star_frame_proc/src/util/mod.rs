mod generics;
mod paths;
mod repr;

pub use generics::*;
pub use paths::*;
pub use repr::*;

use easy_proc::find_attr;
use itertools::Itertools;
use proc_macro2::{Ident, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use proc_macro_error::abort;
use quote::{format_ident, quote, quote_spanned, ToTokens};
use std::fmt::Debug;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    parse_quote, Attribute, Data, DataStruct, DataUnion, DeriveInput, Expr, ExprLit, Field, Fields,
    ItemStruct, Lit, Meta, MetaNameValue, Path, Token, Type, Variant,
};

pub fn get_crate_name() -> TokenStream {
    let generator_crate = crate_name("star_frame").expect("Could not find `star_frame`");
    match generator_crate {
        FoundCrate::Itself => quote! { star_frame },
        FoundCrate::Name(name) => {
            let ident = format_ident!("{}", name);
            quote! { ::#ident }
        }
    }
}

pub fn get_docs<'a>(attrs: impl IntoIterator<Item = &'a Attribute>) -> Expr {
    let doc_strings = attrs
        .into_iter()
        .filter(|a| a.path().is_ident("doc"))
        .map(|a: &'a Attribute| {
            if let Meta::NameValue(MetaNameValue {
                value:
                    Expr::Lit(ExprLit {
                        lit: Lit::Str(str), ..
                    }),
                ..
            }) = &a.meta
            {
                str
            } else {
                abort!(a, "Expected doc attribute to be a name value pair")
            }
        })
        .map(|s| {
            let string = s.value();
            string.trim().to_string()
        })
        .collect::<Vec<_>>();
    parse_quote! { vec![#(#doc_strings.into()),*] }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StrippedAttribute {
    pub index: usize,
    pub attribute: Attribute,
}

pub trait EnumerableAttributes {
    fn enumerate_attributes(&mut self) -> impl Iterator<Item = (usize, &mut Vec<Attribute>)>;
}

impl EnumerableAttributes for ItemStruct {
    fn enumerate_attributes(&mut self) -> impl Iterator<Item = (usize, &mut Vec<Attribute>)> {
        self.fields
            .iter_mut()
            .enumerate()
            .map(|(index, f)| (index, &mut f.attrs))
    }
}

impl EnumerableAttributes for syn::ItemEnum {
    fn enumerate_attributes(&mut self) -> impl Iterator<Item = (usize, &mut Vec<Attribute>)> {
        self.variants
            .iter_mut()
            .enumerate()
            .map(|(index, v)| (index, &mut v.attrs))
    }
}

impl EnumerableAttributes for Vec<Attribute> {
    fn enumerate_attributes(&mut self) -> impl Iterator<Item = (usize, &mut Vec<Attribute>)> {
        std::iter::once((0, self))
    }
}

/// Strips all matching attributes from each attribute group (e.g., struct fields, enum variants) and returns them in order with
/// their group index.
pub fn strip_inner_attributes<'a, I>(
    item: &'a mut impl EnumerableAttributes,
    attribute_name: &'a I,
) -> impl Iterator<Item = StrippedAttribute> + 'a
where
    I: ?Sized,
    Ident: PartialEq<I>,
{
    item.enumerate_attributes().flat_map(|(index, attrs)| {
        let mut removed = vec![];
        attrs.retain(|attr| {
            attr.path()
                .is_ident(attribute_name)
                .then(|| {
                    removed.push(StrippedAttribute {
                        index,
                        attribute: attr.clone(),
                    });
                    false
                })
                .unwrap_or(true)
        });
        removed
    })
}

pub fn reject_attributes(attributes: &[Attribute], ident: &Ident, message: Option<&str>) {
    if find_attr(attributes, ident).is_some() {
        let message = message
            .map(ToString::to_string)
            .unwrap_or_else(|| format!("Cannot use `{}` attribute here", ident));
        abort!(ident, message);
    }
}

pub fn make_derivative_attribute(
    traits: Punctuated<Path, Token![,]>,
    types: &[impl ToTokens],
) -> Attribute {
    let bounds = traits
        .iter()
        .map(|t| {
            let derivitive_bounds = types.iter().map(|ty| quote!(#ty: #t)).collect::<Vec<_>>();
            let derivative_bounds = quote!(#(#derivitive_bounds),*).to_string();
            quote!(#t(bound = #derivative_bounds))
        })
        .collect_vec();
    parse_quote!(#[derivative(#(#bounds),*)])
}

pub fn add_derivative_attributes(
    struct_item: &mut ItemStruct,
    traits: Punctuated<Path, Token![,]>,
) {
    let attributes = make_derivative_attribute(traits, &get_field_types(struct_item).collect_vec());
    struct_item.attrs.push(attributes);
}

pub fn get_field_types(fields: &impl FieldIter) -> impl Iterator<Item = &Type> {
    fields.field_iter().map(|field| &field.ty)
}

/// Check that all fields implement a given trait
///
/// Adapted from the bytemuck derive crate
pub fn generate_fields_are_trait<T: GetGenerics + FieldIter + Spanned>(
    input: &T,
    trait_: Punctuated<Path, Token![+]>,
) -> TokenStream {
    let generics = input.get_generics();
    let (impl_generics, _ty_generics, where_clause) = generics.split_for_impl();
    let span = input.span();
    let field_types = get_field_types(input);
    quote_spanned! {span => const _: fn() = || {
        #[allow(clippy::missing_const_for_fn)]
        #[doc(hidden)]
        fn check #impl_generics () #where_clause {
          fn assert_impl<T: #trait_>() {}
          #(assert_impl::<#field_types>();)*
        }
      };
    }
}

pub fn ensure_data_struct<'a>(item: &'a DeriveInput, error: Option<&str>) -> &'a DataStruct {
    match &item.data {
        Data::Struct(s) => s,
        _ => abort!(item, error.unwrap_or("Expected a struct")),
    }
}

pub trait FieldIter {
    fn field_iter(&self) -> impl Iterator<Item = &Field>;
}

impl FieldIter for Fields {
    fn field_iter(&self) -> impl Iterator<Item = &Field> {
        self.iter()
    }
}

impl FieldIter for Vec<Field> {
    fn field_iter(&self) -> impl Iterator<Item = &Field> {
        self.iter()
    }
}

macro_rules! field_iter {
    ($($item:ty),*) => {
        $(
            impl FieldIter for $item {
                fn field_iter(&self) -> impl Iterator<Item = &Field> {
                    self.fields.iter()
                }
            }
        )*
    };
}

field_iter!(DataStruct, ItemStruct);

impl FieldIter for DeriveInput {
    fn field_iter(&self) -> impl Iterator<Item = &Field> {
        match &self.data {
            Data::Struct(DataStruct { fields, .. }) => fields.iter(),
            Data::Union(DataUnion { fields, .. }) => fields.named.iter(),
            Data::Enum(_) => abort!(self, "cannot get fields on an enum"),
        }
    }
}

pub fn enum_discriminants<'a>(
    variants: impl Iterator<Item = &'a Variant> + 'a,
) -> impl Iterator<Item = Expr> + 'a {
    let mut next_discriminant: Expr = parse_quote!(0);
    variants.map(move |variant| {
        let discriminant = if let Some((_, ref discriminant)) = variant.discriminant {
            discriminant.clone()
        } else {
            next_discriminant.clone()
        };
        next_discriminant = parse_quote! { #discriminant + 1 };
        discriminant
    })
}

pub fn discriminant_vec(expr: &Expr, repr: IntegerRepr) -> TokenStream {
    let bytemuck = Paths::default().bytemuck;
    quote! { #bytemuck::bytes_of::<#repr>(&(#expr)).to_vec() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_strip_attributes_struct() {
        let mut struct_item: ItemStruct = parse_quote! {
            struct MyStruct {
                #[my_attr]
                field1: u8,
                field2: u8,
                #[my_attr(hello)]
                #[my_attr]
                field3: u8,
            }
        };
        let stripped_attributes: Vec<_> =
            strip_inner_attributes(&mut struct_item, "my_attr").collect();
        let expected_stripped_attributes = vec![
            StrippedAttribute {
                index: 0,
                attribute: parse_quote! { #[my_attr] },
            },
            StrippedAttribute {
                index: 2,
                attribute: parse_quote! { #[my_attr(hello)] },
            },
            StrippedAttribute {
                index: 2,
                attribute: parse_quote! { #[my_attr] },
            },
        ];
        assert_eq!(stripped_attributes, expected_stripped_attributes);
        assert_eq!(
            struct_item,
            parse_quote! {
                struct MyStruct {
                    field1: u8,
                    field2: u8,
                    field3: u8,
                }
            }
        );
    }

    #[test]
    fn test_strip_attributes_enum() {
        let mut enum_item: syn::ItemEnum = parse_quote! {
            enum MyEnum {
                #[my_attr]
                Variant1,
                Variant2,
                #[my_attr(hello)]
                #[my_attr(hello2)]
                Variant3,
            }
        };
        let stripped_attributes: Vec<_> =
            strip_inner_attributes(&mut enum_item, "my_attr").collect();
        let expected_stripped_attributes = vec![
            StrippedAttribute {
                index: 0,
                attribute: parse_quote! { #[my_attr] },
            },
            StrippedAttribute {
                index: 2,
                attribute: parse_quote! { #[my_attr(hello)] },
            },
            StrippedAttribute {
                index: 2,
                attribute: parse_quote! { #[my_attr(hello2)] },
            },
        ];
        assert_eq!(stripped_attributes, expected_stripped_attributes);
        assert_eq!(
            enum_item,
            parse_quote! {
                enum MyEnum {
                    Variant1,
                    Variant2,
                    Variant3,
                }
            }
        );
    }
}
