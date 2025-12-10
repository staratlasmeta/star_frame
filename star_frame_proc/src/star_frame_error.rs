use easy_proc::ArgumentList;
use itertools::Itertools;
use proc_macro2::TokenStream;
use proc_macro_error2::{abort, ResultExt};
use quote::quote;
use sha2::{Digest, Sha256};
use syn::{parse2, parse_quote, Fields, ItemEnum, LitInt, LitStr};

use crate::util::{
    enum_discriminants, get_docs, ignore_cfg_module, reject_generics, strip_inner_attributes, Paths,
};

#[derive(Debug, ArgumentList, Clone)]
pub struct StarFrameErrorArgs {
    pub offset: Option<LitInt>,
    #[argument(presence)]
    pub skip_idl: bool,
}

impl StarFrameErrorArgs {
    fn get_offset(&self) -> LitInt {
        if let Some(offset) = &self.offset {
            return offset.clone();
        }

        let crate_name = env!("CARGO_PKG_NAME");
        let mut hasher = Sha256::default();
        hasher.update(crate_name.as_bytes());
        let offset = u16::from_le_bytes(
            hasher.finalize().as_slice()[0..2]
                .try_into()
                .expect("The slice is more than 2 bytes"),
        );
        parse_quote!(#offset)
    }
}

const ERROR_MESSAGE_ATTR: &str =
    "Each variant must have an attribute in the format `#[msg(\"My error message\")]`";

pub fn star_frame_error_impl(mut item: ItemEnum, args: TokenStream) -> TokenStream {
    Paths!(crate_name, prelude);

    let args = StarFrameErrorArgs::parse_arguments(&parse_quote!(#[star_frame_error(#args)]));

    reject_generics(
        &item,
        Some("Generics are not supported for star_frame_error"),
    );

    let ident = &item.ident;
    let offset = args.get_offset();

    let offset = quote!(((#offset as u32) << 16));
    let ix_discriminants = enum_discriminants(item.variants.iter())
        .map(|disc| parse_quote!((#offset + #disc)))
        .collect_vec();

    let messages = item
        .variants
        .iter_mut()
        .zip_eq(ix_discriminants)
        .map(|(v, disc)| {
            if !matches!(v.fields, Fields::Unit) {
                abort!(v.fields, "StarFrameError enums must be unit variants");
            }

            let Some((message_attr,)) = strip_inner_attributes(&mut v.attrs, "msg").collect_tuple()
            else {
                abort!(v, ERROR_MESSAGE_ATTR);
            };

            v.discriminant = Some((parse_quote!(=), disc));
            message_attr.attribute
        });

    let messages = messages
        .map(|attr| {
            let list = attr.meta.require_list().expect_or_abort(ERROR_MESSAGE_ATTR);
            parse2::<LitStr>(list.tokens.clone())
                .expect_or_abort("Failed to parse error message as a string literal")
        })
        .collect_vec();
    let variant_idents = item.variants.iter().map(|v| v.ident.clone()).collect_vec();

    let star_frame_error_impl = quote! {
        #[automatically_derived]
        impl #prelude::StarFrameError for #ident {
            fn code(&self) -> u32 {
                *self as u32
            }
            fn name(&self) -> #crate_name::alloc::borrow::Cow<'static, str> {
                match self {
                    #(Self::#variant_idents => #messages),*
                }
                .into()
            }
        }
    };

    let display_impl = quote! {
        #[automatically_derived]
        impl ::core::fmt::Display for #ident {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                match self {
                    #(Self::#variant_idents => write!(f, #messages)),*
                }
            }
        }
    };

    let idl_impl = (!args.skip_idl).then(|| {
        let error_nodes = item.variants.iter().zip(messages).map(|(v, message)| {
            let name = v.ident.to_string();
            let docs = get_docs(&v.attrs);
            let disc = &v
                .discriminant
                .as_ref()
                .expect("Discriminant has been set")
                .1;
            let message = message.value();
            quote! {
                #prelude::ErrorNode {
                    name: #name.into(),
                    code: (#disc) as usize,
                    message: #message.to_string(),
                    docs: #docs.into(),
                }
            }
        });
        ignore_cfg_module(ident, "_errors_to_idl", quote! {
            use #crate_name::alloc::string::ToString;

            #[cfg(all(feature = "idl", not(target_os = "solana")))]
            #[automatically_derived]
            impl #prelude::ErrorsToIdl for #ident {
                fn errors_to_idl(idl_definition: &mut #prelude::IdlDefinition) -> #prelude::IdlResult<()> {
                    let errors = vec![
                        #(#error_nodes,)*
                    ];
                    idl_definition.errors.extend(errors);
                    Ok(())
                }
            }
        })
    });

    quote!(
        #[repr(u32)]
        #[derive(Copy, Clone, Debug, Eq, PartialEq)]
        #item

        #star_frame_error_impl

        #display_impl

        #idl_impl
    )
}
