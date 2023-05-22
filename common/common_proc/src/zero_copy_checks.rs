use crate::get_crate_name;
use proc_macro2::TokenStream;
use proc_macro_error::*;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::*;

#[derive(Copy, Clone, Debug)]
pub enum ZeroCopyType {
    Account,
    Struct,
}

struct ZeroCopyArgs {
    skip_zero_copy: bool,
    transparent: bool,
}
impl Parse for ZeroCopyArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content: Punctuated<_, Token![,]> = input.parse_terminated(Ident::parse)?;
        let mut out = ZeroCopyArgs {
            skip_zero_copy: false,
            transparent: false,
        };
        for ident in content {
            match ident.to_string().as_str() {
                "skip_zero_copy" => out.skip_zero_copy = true,
                "transparent" => out.transparent = true,
                _ => abort!(ident, "expected `skip_zero_copy` or `transparent`"),
            }
        }
        Ok(out)
    }
}

pub fn zero_copy_checks(
    args: proc_macro::TokenStream,
    input: TokenStream,
    ty: ZeroCopyType,
) -> proc_macro::TokenStream {
    let crate_name = get_crate_name();
    let input_tokens = input.into();
    let mut input_strct = parse_macro_input!(input_tokens as ItemStruct);
    let strct = input_strct.clone();

    let ident = &strct.ident;

    let args = parse_macro_input!(args as ZeroCopyArgs);
    let mut found_attr_index = if args.skip_zero_copy {
        Some(None)
    } else {
        None
    };
    for (index, attribute) in strct.attrs.iter().enumerate() {
        match ty {
            ZeroCopyType::Account => {
                if attribute.path.is_ident("account")
                    && attribute
                        .clone()
                        .parse_args::<Ident>()
                        .map_or(false, |ident| ident.to_string().as_str() == "zero_copy")
                {
                    found_attr_index = Some(Some(index));
                    break;
                }
            }
            ZeroCopyType::Struct => {
                if attribute.path.is_ident("zero_copy") {
                    found_attr_index = Some(Some(index));
                    break;
                }
            }
        }
    }
    if let Some(index) = found_attr_index {
        if let Some(index) = index {
            input_strct.attrs.insert(
                index + 1,
                syn::parse_quote! {
                    #[derive(Debug)]
                },
            );
        }
    } else {
        match ty {
            ZeroCopyType::Struct => {
                abort!(
                    strct,
                    "Struct `{}` is not annotated with `#[zero_copy]`",
                    ident
                );
            }
            ZeroCopyType::Account => {
                abort!(
                    strct,
                    "Struct `{}` is not annotated with `#[account(zero_copy)]`",
                    ident
                );
            }
        }
    }

    let mut types = Vec::with_capacity(strct.fields.len());
    let mut names = Vec::with_capacity(strct.fields.len());
    match strct.fields {
        Fields::Named(fields) => {
            for field in fields.named {
                let name = field.ident.unwrap();
                names.push(quote! { #name });
                types.push(field.ty);
            }
        }
        Fields::Unnamed(fields) => {
            for (index, field) in fields.unnamed.into_iter().enumerate() {
                let index = Index::from(index);
                names.push(quote! { #index });
                types.push(field.ty);
            }
        }
        Fields::Unit => abort!(strct.struct_token, "Unit structs are not supported"),
    }

    let repr = if args.transparent {
        quote! { transparent }
    } else {
        quote! { C, packed }
    };

    let (data_size, safe_trait) = match ty {
        ZeroCopyType::Account => (
            quote! { 8 + },
            quote! {
                #[automatically_derived]
                unsafe impl #crate_name::SafeZeroCopy for #ident {}
                #[automatically_derived]
                unsafe impl #crate_name::SafeZeroCopyAccount for #ident {}
            },
        ),
        ZeroCopyType::Struct => {
            input_strct.attrs.push(syn::parse_quote! {
                #[derive(#crate_name::bytemuck::Pod, #crate_name::bytemuck::Zeroable)]
            });
            (
                quote! {},
                quote! {
                    #[automatically_derived]
                    unsafe impl #crate_name::SafeZeroCopy for #ident {}
                },
            )
        }
    };

    let fields_size = types.iter().map(|ty| {
        quote! { + ::std::mem::size_of::<#ty>() }
    });

    (quote! {
        #[repr(#repr)]
        #input_strct

        #crate_name::static_assertions::const_assert_eq!(
            ::std::mem::size_of::<#ident>(),
            0 #(#fields_size)*
        );

        #(
            #crate_name::static_assertions::assert_impl_all!(#types: #crate_name::bytemuck::Pod);
        )*

        #[automatically_derived]
        impl #crate_name::DataSize for #ident {
            const MIN_DATA_SIZE: usize = #data_size ::std::mem::size_of::<#ident>();
        }
        #safe_trait
    })
    .into()
}
