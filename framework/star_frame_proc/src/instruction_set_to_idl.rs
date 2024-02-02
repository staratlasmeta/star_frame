use crate::util::Paths;
use crate::{util, IdentWithArgs};
use heck::{ToSnakeCase, ToTitleCase, ToUpperCamelCase};
use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::abort;
use quote::{format_ident, quote, ToTokens};
use strum::{EnumIter, IntoEnumIterator};
use syn::parse::{Parse, ParseStream};
use syn::{
    parse_quote, Attribute, Data, DataEnum, DeriveInput, Fields, GenericParam, LifetimeParam,
    LitInt, LitStr, Token, Type, Visibility,
};

#[allow(dead_code)]
struct StrippedDeriveInput {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
}

/// Valid `[repr(...)]` types for `InstructionSet`
#[derive(EnumIter)]
pub enum ValidReprTypes {
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
}

impl ValidReprTypes {
    const fn as_str(&self) -> &'static str {
        match self {
            Self::U8 => "u8",
            Self::U16 => "u16",
            Self::U32 => "u32",
            Self::U64 => "u64",
            Self::U128 => "u128",
            Self::I8 => "i8",
            Self::I16 => "i16",
            Self::I32 => "i32",
            Self::I64 => "i64",
            Self::I128 => "i128",
        }
    }

    const fn disc_size(&self) -> u8 {
        match self {
            Self::U8 | Self::I8 => 1,
            Self::U16 | Self::I16 => 2,
            Self::U32 | Self::I32 => 4,
            Self::U64 | Self::I64 => 8,
            Self::U128 | Self::I128 => 16,
        }
    }

    fn as_type(&self) -> Type {
        match self {
            Self::U8 => parse_quote!(u8),
            Self::U16 => parse_quote!(u16),
            Self::U32 => parse_quote!(u32),
            Self::U64 => parse_quote!(u64),
            Self::U128 => parse_quote!(u128),
            Self::I8 => parse_quote!(i8),
            Self::I16 => parse_quote!(i16),
            Self::I32 => parse_quote!(i32),
            Self::I64 => parse_quote!(i64),
            Self::I128 => parse_quote!(i128),
        }
    }
}

impl ToTokens for ValidReprTypes {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.as_str().to_tokens(tokens);
    }
}

fn find_repr(attrs: &Vec<Attribute>) -> Option<ValidReprTypes> {
    let mut inner_type = None;
    for attr in attrs {
        if attr.path().is_ident("repr") {
            let Ok(args) = attr.parse_args_with(|p: ParseStream| {
                p.parse_terminated(IdentWithArgs::<LitInt>::parse, Token![,])
            }) else {
                abort!(attr, "Failed to parse repr attribute")
            };
            for arg in args {
                let ident = arg.ident.to_string();
                for repr in ValidReprTypes::iter() {
                    if ident == repr.as_str() {
                        inner_type = Some(repr);
                    }
                }
            }
        }
    }
    inner_type
}

pub fn derive_instruction_set_to_idl_impl(input: DeriveInput) -> TokenStream {
    let paths = Paths::default();

    match input.data {
        Data::Enum(e) => derive_instruction_set_to_idl_impl_enum(
            paths,
            e,
            StrippedDeriveInput {
                attrs: input.attrs,
                vis: input.vis,
                ident: input.ident,
            },
        ),
        Data::Struct(s) => abort!(s.struct_token, "Structs are not supported"),
        Data::Union(u) => abort!(u.union_token, "Unions are not supported"),
    }
}

fn derive_instruction_set_to_idl_impl_enum(
    paths: Paths,
    data: DataEnum,
    input: StrippedDeriveInput,
) -> TokenStream {
    let Paths {
        idl_definition,
        idl_instruction,
        instruction_set_to_idl,
        instruction_to_idl,
        result,
        ..
    } = paths;

    let ident = &input.ident;

    let variant_discriminants =
        format_ident!("{}Discriminants", ident.to_string().to_upper_camel_case());

    let variant_names = data
        .variants
        .iter()
        .map(|name| name.ident.clone())
        .collect::<Vec<_>>();

    let variant_snake_names = variant_names
        .clone()
        .into_iter()
        .map(|name| format_ident!("{}", name.to_string().to_snake_case()))
        .collect::<Vec<_>>();

    let variant_snake_str = variant_snake_names
        .clone()
        .into_iter()
        .map(|name| LitStr::new(&name.to_string(), Span::call_site()))
        .collect::<Vec<LitStr>>();

    let variant_title_names = variant_names
        .clone()
        .into_iter()
        .map(|name| LitStr::new(&name.to_string().to_title_case(), Span::call_site()))
        .collect::<Vec<LitStr>>();

    let variant_docs: Vec<LitStr> = data
        .variants
        .iter()
        .map(|field| LitStr::new(&util::get_docs(&field.attrs), Span::call_site()))
        .collect();

    let out = quote! {
        impl<'a> #instruction_set_to_idl<'a> for #ident<'a> {
            fn instruction_set_to_idl(idl_definition: &mut #idl_definition) -> #result<()> {
                #(
                    let #variant_snake_names = <#variant_names as #instruction_to_idl<_>>::instruction_to_idl(idl_definition, ())?;
                    idl_definition.instructions.insert(
                        #variant_snake_str.to_string(),
                        #idl_instruction {
                            name: #variant_title_names.to_string(),
                            description: #variant_docs.to_string(),
                            discriminant: serde_json::to_value(#variant_discriminants::#variant_names.into_repr())
                                .expect("Cannot serialize u32? Banana"),
                            definition: #variant_snake_names,
                            extension_fields: Default::default(),
                        }
                    );
                )*
                Ok(())
            }
        }
    };
    out
}
