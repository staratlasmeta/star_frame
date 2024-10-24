use crate::util;
use crate::util::Paths;
use easy_proc::find_attr;
use proc_macro2::{Span, TokenStream};
use proc_macro_error::{abort, abort_call_site, OptionExt};
use quote::{quote, ToTokens};
use syn::{parse_quote, DeriveInput, Expr, Fields, LitStr, Type};

pub fn derive_type_to_idl(input: &DeriveInput) -> TokenStream {
    let Paths {
        macro_prelude: prelude,
        declared_program_type,
        idl_ty_program_ident,
        result,
        ..
    } = &Paths::default();

    let associated_program = if let Some(attr) = find_attr(&input.attrs, idl_ty_program_ident) {
        attr.parse_args::<Type>()
            .unwrap_or_else(|e| abort_call_site!("Could not parse program type: {}", e))
    } else {
        declared_program_type.clone()
    };

    let ident = &input.ident;
    let type_docs = &util::get_docs(&input.attrs);
    let type_def = match input.data {
        syn::Data::Struct(ref s) => idl_struct_type_def(s),
        syn::Data::Enum(ref e) => abort!(e.enum_token, "Enums are not supported"),
        syn::Data::Union(ref u) => abort!(u.union_token, "Unions are not supported"),
    };
    let ident_str = LitStr::new(&ident.to_string(), Span::call_site());

    if input.generics.type_params().next().is_some()
        || input.generics.const_params().next().is_some()
    {
        abort!(input.generics, "Generics are not supported");
    }

    let (impl_gen, ty_gen, where_clause) = input.generics.split_for_impl();

    quote! {
        #[automatically_derived]
        impl #impl_gen #prelude::TypeToIdl for #ident #ty_gen #where_clause {
            type AssociatedProgram = #associated_program;
            fn type_to_idl(idl_definition: &mut #prelude::IdlDefinition) -> #result<#prelude::IdlTypeDef> {
                let source = #prelude::item_source::<Self>();
                let type_def = #type_def;
                let idl_type = #prelude::IdlType {
                    info: #prelude::ItemInfo {
                        name: #ident_str.to_string(),
                        description: #type_docs,
                        source: source.clone(),
                    },
                    type_def,
                    generics: vec![],
                };
                let namespace = idl_definition.add_type(idl_type, Self::AssociatedProgram::PROGRAM_ID);
                Ok(#prelude::IdlTypeDef::Defined(#prelude::IdlTypeId {
                    namespace,
                    source,
                    provided_generics: vec![],
                }))
            }
        }
    }
}
fn idl_struct_type_def(s: &syn::DataStruct) -> TokenStream {
    let Paths {
        macro_prelude: prelude,
        ..
    } = &Paths::default();
    let tuple = matches!(s.fields, Fields::Unnamed(_));
    let idl_fields: Vec<TokenStream> = s
        .fields
        .iter()
        .map(|f| {
            let path: Expr = if tuple {
                parse_quote!(None)
            } else {
                let field_name = f
                    .ident
                    .as_ref()
                    .expect_or_abort("No ident on named field?")
                    .to_string();
                let field_name = field_name.trim();

                parse_quote!(Some(#field_name.to_string()))
            };
            let field_type = &f.ty;
            let type_def =
                quote! { <#field_type as #prelude::TypeToIdl>::type_to_idl(idl_definition)? };
            let description = util::get_docs(&f.attrs);
            quote! {
                #prelude::IdlStructField {
                    path: #path,
                    description: #description,
                    type_def: #type_def,
                }
            }
        })
        .collect();

    let type_def = quote! {
        #prelude::IdlTypeDef::Struct(vec![#(#idl_fields),*])
    };

    type_def
}
