use crate::util;
use crate::util::Paths;
use easy_proc::find_attr;
use proc_macro2::{Span, TokenStream};
use proc_macro_error::{abort, abort_call_site, OptionExt};
use quote::{quote, ToTokens};
use syn::{DeriveInput, Fields, LitStr, Type};

pub fn derive_type_to_idl(input: &DeriveInput) -> TokenStream {
    let Paths {
        type_to_idl,
        idl_definition,
        idl_type_def,
        semver,
        declared_program_type,
        idl_ty_program_ident,
        program_to_idl,
        result,
        idl_definition_ref,
        idl_type,
        idl_type_id,
        ..
    } = &Paths::default();

    let associated_program = if let Some(attr) = find_attr(&input.attrs, idl_ty_program_ident) {
        attr.parse_args::<Type>()
            .unwrap_or_else(|e| abort_call_site!("Could not parse program type: {}", e))
    } else {
        declared_program_type.clone()
    };

    let ident = &input.ident;
    let type_docs = LitStr::new(&util::get_docs(&input.attrs), Span::call_site());
    let type_def = match input.data {
        syn::Data::Struct(ref s) => idl_struct_type_def(s),
        syn::Data::Enum(ref e) => abort!(e.enum_token, "Enums are not supported"),
        syn::Data::Union(ref u) => abort!(u.union_token, "Unions are not supported"),
    };
    let ident_str = LitStr::new(&ident.to_string(), Span::call_site());

    quote! {
        #[automatically_derived]
        impl #type_to_idl for #ident {
            type AssociatedProgram = #associated_program;
            fn type_to_idl(idl_definition: &mut #idl_definition) -> #result<#idl_type_def> {
                let namespace = if idl_definition.namespace == <Self::AssociatedProgram as #program_to_idl>::idl_namespace() {
                    let type_def = #type_def;
                    idl_definition.add_type_if_missing(#ident_str, || #idl_type {
                        name: #ident_str.to_string(),
                        description: #type_docs.to_string(),
                        type_def,
                        generics: vec![],
                    });
                    None
                } else {
                    idl_definition.required_idl_definitions.insert(
                        <Self::AssociatedProgram as #program_to_idl>::idl_namespace().to_string(),
                        #idl_definition_ref {
                            namespace: <Self::AssociatedProgram as #program_to_idl>::idl_namespace().to_string(),
                            version: #semver::Wildcard,
                        },
                    );
                    Some(<Self::AssociatedProgram as #program_to_idl>::idl_namespace().to_string())
                };
                Ok(#idl_type_def::IdlType(#idl_type_id {
                    namespace,
                    type_id: #ident_str.to_string(),
                    provided_generics: vec![],
                }))
            }
        }
    }
}
fn idl_struct_type_def(s: &syn::DataStruct) -> TokenStream {
    let Paths {
        type_to_idl,
        idl_field,
        idl_type_def,
        ..
    } = &Paths::default();
    let tuple = matches!(s.fields, Fields::Unnamed(_));
    let idl_fields: Vec<TokenStream> = s
        .fields
        .iter()
        .enumerate()
        .map(|(index, f)| {
            let (name, description, path_id) = if tuple {
                (
                    LitStr::new("", Span::call_site()),
                    LitStr::new("", Span::call_site()),
                    quote!(#index),
                )
            } else {
                let field_name = f
                    .ident
                    .as_ref()
                    .expect_or_abort("No ident on named field?")
                    .to_token_stream();
                let name = LitStr::new(&field_name.to_string(), Span::call_site());
                let description = LitStr::new(&util::get_docs(&f.attrs), Span::call_site());
                let path_id = name.to_token_stream();
                (name, description, path_id)
            };
            let field_type = &f.ty;
            let type_def = quote! { <#field_type as #type_to_idl>::type_to_idl(idl_definition)? };
            quote! {
                #idl_field {
                    name: #name.to_string(),
                    description: #description.to_string(),
                    path_id: #path_id.to_string(),
                    type_def: #type_def,
                }
            }
        })
        .collect();

    let type_def = quote! {
        #idl_type_def::Struct(vec![#(#idl_fields),*])
    };

    type_def
}
