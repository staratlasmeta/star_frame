use crate::get_crate_name;
use proc_macro2::TokenStream;
use proc_macro_error::*;
use quote::quote;
use syn::*;

pub fn unpackable_impl(derive_input: DeriveInput) -> TokenStream {
    let crate_name = get_crate_name();

    let input = match derive_input.data {
        Data::Struct(input) => input,
        Data::Enum(_) => abort!(derive_input, "Unpackable cannot be derived for enums"),
        Data::Union(_) => abort!(derive_input, "Unpackable cannot be derived for unions"),
    };

    let docs = derive_input
        .attrs
        .iter()
        .filter(|attr| attr.path.is_ident("doc"));
    let name = derive_input
        .attrs
        .iter()
        .find(|attr| attr.path.is_ident("unpackable"))
        .map_or_else(
            || {
                Ident::new(
                    &(derive_input.ident.to_string() + "Unpacked"),
                    derive_input.ident.span(),
                )
            },
            |attr| attr.parse_args::<Ident>().unwrap(),
        );

    let mut fields = input.fields;
    let named = matches!(&fields, Fields::Named(_));
    let mut unpack = Vec::new();
    let mut pack = Vec::new();
    if let Fields::Named(FieldsNamed { named: fields, .. })
    | Fields::Unnamed(FieldsUnnamed {
        unnamed: fields, ..
    }) = &mut fields
    {
        for field in fields {
            if field
                .attrs
                .iter()
                .any(|attr| attr.path.is_ident("packed_sub_struct"))
            {
                let ty = &field.ty;
                field.ty = parse_quote! {
                    <#ty as Unpackable>::Unpacked
                };
                let field_ident = &field.ident;
                unpack.push(if named {
                    quote! {
                        #field_ident: self.#field_ident.unpack()
                    }
                } else {
                    quote! {
                        self.#field_ident.unpack()
                    }
                });
                pack.push(if named {
                    quote! {
                        #field_ident: self.#field_ident.pack()
                    }
                } else {
                    quote! {
                        self.#field_ident.pack()
                    }
                });
            } else {
                let field_ident = &field.ident;
                let push = if named {
                    quote! {
                        #field_ident: self.#field_ident
                    }
                } else {
                    quote! {
                        self.#field_ident
                    }
                };
                unpack.push(push.clone());
                pack.push(push);
            }
            field.attrs.retain(|attr| attr.path.is_ident("doc"));
        }
    }
    let ident = derive_input.ident;
    let vis = derive_input.vis;

    let unpack = if named {
        quote! {
            #name {
                #(#unpack,)*
            }
        }
    } else {
        quote! {
            #name (
                #(#unpack,)*
            )
        }
    };
    let pack = if named {
        quote! {
            #ident {
                #(#pack,)*
            }
        }
    } else {
        quote! {
            #ident (
                #(#pack,)*
            )
        }
    };

    quote! {
        impl #crate_name::Unpackable for #ident {
            type Unpacked = #name;
            fn unpack(self) -> Self::Unpacked {
                #unpack
            }
        }

        #(#docs)*
        #[derive(
            Clone,
            Debug,
            #crate_name::anchor_lang::prelude::AnchorSerialize,
            #crate_name::anchor_lang::prelude::AnchorDeserialize,
        )]
        #vis struct #name #fields

        impl #crate_name::Unpacked for #name {
            type Packed = #ident;
            fn pack(&self) -> Self::Packed {
                #pack
            }
        }
    }
}
