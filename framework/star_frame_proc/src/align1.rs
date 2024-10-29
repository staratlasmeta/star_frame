use crate::util::{get_crate_name, IdentWithArgs};
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_quote, Data, DataStruct, DataUnion, DeriveInput, Fields, LitInt, Token};

pub fn derive_align1_impl(derive_input: DeriveInput) -> TokenStream {
    let crate_name = get_crate_name();
    match derive_input.data.clone() {
        Data::Struct(DataStruct { fields, .. }) => {
            derive_align1_for_struct(fields, derive_input, &crate_name)
        }
        Data::Union(DataUnion { fields, .. }) => {
            derive_align1_for_struct(Fields::Named(fields), derive_input, &crate_name)
        }
        Data::Enum(e) => {
            // TODO: Derive for repr u8 and unit enums
            for variant in e.variants {
                if variant.fields != Fields::Unit {
                    abort!(variant.fields, "Align1 only supports unit enums");
                }
            }

            abort!(e.enum_token, "Align1 cannot be derived for enums");
        }
    }
}

fn derive_align1_for_struct(
    fields: Fields,
    derive_input: DeriveInput,
    crate_name: &TokenStream,
) -> TokenStream {
    let packed = derive_input.attrs.into_iter().any(|attr| {
        attr.path().is_ident("repr") && {
            let Ok(args) = attr.parse_args_with(|p: ParseStream| {
                p.parse_terminated(IdentWithArgs::<LitInt>::parse, Token![,])
            }) else {
                abort!(attr, "Repr invalid args")
            };
            // args.iter().any(|arg|arg.ident.to_string() == "packed" && {
            //     if let Some(num) = arg.args {
            //
            //     }
            // });
            for arg in args {
                let ident = arg.ident.to_string();
                let arg = arg.args.as_ref().and_then(|a| a.arg.as_ref());
                if &ident == "align" && arg.map_or(false, |align| &align.to_string() != "1") {
                    abort!(arg, "`align` argument must be 1 to implement `Align1`");
                }
                if &ident == "packed" {
                    if arg.map_or(false, |align| &align.to_string() != "1") {
                        abort!(
                            arg,
                            "`packed` argument must be 1 or not present to implement `Align1`"
                        );
                    } else {
                        return true;
                    }
                }
            }
            false
        }
    });

    let ident = derive_input.ident;

    let mut gen = derive_input.generics;
    let wc = gen.make_where_clause();
    if !packed {
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
