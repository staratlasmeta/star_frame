mod generics;
mod paths;
mod repr;

pub use generics::*;
pub use paths::*;
pub use repr::*;
use std::borrow::Borrow;

use easy_proc::find_attr;
use heck::ToSnakeCase;
use itertools::Itertools;
use proc_macro2::{Ident, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use proc_macro_error2::abort;
use quote::{format_ident, quote, ToTokens};
use std::fmt::Debug;
use syn::{
    parse_quote,
    punctuated::Punctuated,
    token::{Brace, Paren},
    Attribute, Data, DataStruct, DataUnion, DeriveInput, Expr, ExprLit, Field, Fields, FieldsNamed,
    FieldsUnnamed, ItemStruct, Lit, Meta, MetaNameValue, Path, Token, Type, Variant, Visibility,
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
    parse_quote! { vec![#(#doc_strings.to_string()),*] }
}

pub fn is_doc_attribute(attribute: &impl Borrow<Attribute>) -> bool {
    attribute.borrow().path().is_ident("doc")
        && attribute.borrow().meta.require_name_value().is_ok()
}

pub fn get_doc_attributes(attrs: &[Attribute]) -> Vec<Attribute> {
    attrs.iter().filter(is_doc_attribute).cloned().collect_vec()
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StrippedAttribute {
    pub index: usize,
    pub attribute: Attribute,
}

pub trait EnumerableAttributes {
    fn enumerate_attributes_mut(&mut self) -> impl Iterator<Item = (usize, &mut Vec<Attribute>)>;
    fn enumerate_attributes(&self) -> impl Iterator<Item = (usize, &Vec<Attribute>)>;
}

impl EnumerableAttributes for ItemStruct {
    fn enumerate_attributes_mut(&mut self) -> impl Iterator<Item = (usize, &mut Vec<Attribute>)> {
        self.fields
            .iter_mut()
            .enumerate()
            .map(|(index, f)| (index, &mut f.attrs))
    }
    fn enumerate_attributes(&self) -> impl Iterator<Item = (usize, &Vec<Attribute>)> {
        self.fields
            .iter()
            .enumerate()
            .map(|(index, f)| (index, &f.attrs))
    }
}

impl EnumerableAttributes for syn::ItemEnum {
    fn enumerate_attributes_mut(&mut self) -> impl Iterator<Item = (usize, &mut Vec<Attribute>)> {
        self.variants
            .iter_mut()
            .enumerate()
            .map(|(index, v)| (index, &mut v.attrs))
    }
    fn enumerate_attributes(&self) -> impl Iterator<Item = (usize, &Vec<Attribute>)> {
        self.variants
            .iter()
            .enumerate()
            .map(|(index, v)| (index, &v.attrs))
    }
}

impl EnumerableAttributes for Vec<Attribute> {
    fn enumerate_attributes_mut(&mut self) -> impl Iterator<Item = (usize, &mut Vec<Attribute>)> {
        std::iter::once((0, self))
    }
    fn enumerate_attributes(&self) -> impl Iterator<Item = (usize, &Vec<Attribute>)> {
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
    item.enumerate_attributes_mut().flat_map(|(index, attrs)| {
        let mut removed = vec![];
        attrs.retain(|attr| {
            if attr.path().is_ident(attribute_name) {
                {
                    removed.push(StrippedAttribute {
                        index,
                        attribute: attr.clone(),
                    });
                    false
                }
            } else {
                true
            }
        });
        removed
    })
}

pub fn reject_attributes(attributes: &[Attribute], ident: &Ident, message: Option<&str>) {
    if let Some(find) = find_attr(attributes, ident) {
        let message = message
            .map(ToString::to_string)
            .unwrap_or_else(|| format!("Cannot use `{ident}` attribute here"));
        abort!(find, message);
    }
}

pub fn restrict_attributes(attributes: &impl EnumerableAttributes, allowed_attributes: &[&str]) {
    attributes.enumerate_attributes().for_each(|(_, attrs)| {
        for attr in attrs.iter() {
            let ident = attr.path().get_ident().unwrap_or_else(|| {
                abort!(attr, "Expected attribute to be an identifier");
            });
            if !allowed_attributes.iter().any(|allowed| ident == allowed) {
                let message = if allowed_attributes.is_empty() {
                    "No attributes are allowed here".into()
                } else {
                    format!(
                        "Only the following attribute idents are allowed: {}",
                        allowed_attributes
                            .iter()
                            .map(ToString::to_string)
                            .join(", ")
                    )
                };
                abort!(attr, message);
            }
        }
    });
}

pub fn get_field_types(fields: &impl FieldIter) -> impl Iterator<Item = &Type> {
    fields.field_iter().map(|field| &field.ty)
}

pub fn get_field_vis(fields: &impl FieldIter) -> impl Iterator<Item = &Visibility> {
    fields.field_iter().map(|field| &field.vis)
}

pub fn get_field_idents(fields: &impl FieldIter) -> impl Iterator<Item = &Ident> {
    fields
        .field_iter()
        .map(|field| field.ident.as_ref().expect("Unnamed field"))
}

/// Check that all fields implement a given trait
///
/// Adapted from the bytemuck derive crate
pub fn generate_fields_are_trait<F: FieldIter, G: GetGenerics>(
    fields: &F,
    generics: &G,
    trait_: Punctuated<Path, Token![+]>,
) -> TokenStream {
    let generics = generics.get_generics();
    let (impl_generics, _ty_generics, where_clause) = generics.split_for_impl();
    let field_types = get_field_types(fields);
    quote! {const _: fn() = || {
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

pub fn make_struct(
    ident: &Ident,
    fields: &impl FieldIter,
    generics: &impl GetGenerics,
) -> ItemStruct {
    let fields = make_struct_fields(fields);
    let generics = generics.get_generics();
    let (impl_generics, _ty_generics, where_clause) = generics.split_for_impl();
    match fields {
        Fields::Named(named) => {
            parse_quote! {
                pub struct #ident #impl_generics #where_clause #named
            }
        }
        unnamed => {
            parse_quote! {
                pub struct #ident #impl_generics #unnamed #where_clause;
            }
        }
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

impl FieldIter for Vec<&Field> {
    fn field_iter(&self) -> impl Iterator<Item = &Field> {
        self.iter().cloned()
    }
}

impl FieldIter for &[Field] {
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

pub fn make_struct_fields(fields: &impl FieldIter) -> Fields {
    let fields =
        syn::punctuated::Punctuated::<_, Token![,]>::from_iter(fields.field_iter().cloned());
    let Some(first) = fields.first() else {
        return Fields::Unit;
    };
    if first.ident.is_some() {
        Fields::Named(FieldsNamed {
            named: fields,
            brace_token: Brace::default(),
        })
    } else {
        Fields::Unnamed(FieldsUnnamed {
            unnamed: fields,
            paren_token: Paren::default(),
        })
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

pub fn ignore_cfg_module(ident: &Ident, suffix: &str, body: TokenStream) -> TokenStream {
    if body.is_empty() {
        return TokenStream::new();
    }
    let module_name = format_ident!("_{}{suffix}", ident.to_string().to_snake_case());
    quote! {
        #[allow(unexpected_cfgs)]
        #[doc(hidden)]
        mod #module_name {
            use super::*;
            #body
        }
        pub use #module_name::*;
    }
}

pub fn recurse_type_operator(
    op: &TokenStream,
    types: &[impl ToTokens],
    default: &TokenStream,
) -> TokenStream {
    let Some((first, rem)) = types.split_first() else {
        return default.clone();
    };
    if rem.is_empty() {
        return first.to_token_stream();
    }
    let last = recurse_type_operator(op, rem, default);
    quote! { #op<#first, #last> }
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

    #[test]
    fn test_recurse_type_operator() {
        let pair_tokens = recurse_type_operator(&quote!(op), &[quote!(A), quote!(B)], &quote!());
        let ty: Type = parse_quote!(#pair_tokens);
        assert_eq!(ty, parse_quote!(op<A, B>));

        let five_tokens = recurse_type_operator(
            &quote!(op),
            &[quote!(A), quote!(B), quote!(C), quote!(D), quote!(E)],
            &quote!(),
        );
        let ty: Type = parse_quote!(#five_tokens);
        assert_eq!(ty, parse_quote!(op<A, op<B, op<C, op<D, E>>>>));
    }
}
