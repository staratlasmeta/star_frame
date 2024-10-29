use crate::util;
use crate::util::{
    discriminant_vec, enum_discriminants, get_repr, reject_generics, IntegerRepr, Paths,
};
use easy_proc::{find_attr, ArgumentList};
use proc_macro2::{Span, TokenStream};
use proc_macro_error::{abort, OptionExt};
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_quote, Attribute, DataStruct, DataUnion, DeriveInput, Expr, Fields, LitStr, Type};

#[derive(Debug, ArgumentList, Default)]
pub struct TypeToIdlArgs {
    pub program: Option<Type>,
}

pub fn derive_type_to_idl(input: DeriveInput) -> TokenStream {
    let Paths {
        type_to_idl_args_ident,
        ..
    } = &Paths::default();

    let args = find_attr(&input.attrs, type_to_idl_args_ident)
        .map(TypeToIdlArgs::parse_arguments)
        .unwrap_or_default();

    derive_type_to_idl_inner(&input, args)
}

pub fn derive_type_to_idl_inner(input: &DeriveInput, args: TypeToIdlArgs) -> TokenStream {
    let Paths {
        macro_prelude: prelude,
        declared_program_type,
        result,
        ..
    } = &Paths::default();

    let associated_program = args.program.unwrap_or(declared_program_type.clone());

    // todo: support generics maybe?
    reject_generics(input, Some("Generics are not supported yet for TypeToIdl"));
    let ident = &input.ident;
    let ident_str = LitStr::new(&ident.to_string(), Span::call_site());
    let type_docs = &util::get_docs(&input.attrs);
    let type_def = match &input.data {
        syn::Data::Struct(DataStruct { fields, .. }) => idl_struct_type_def(fields),
        syn::Data::Enum(data_enum) => idl_enum_type_def(data_enum, &input.attrs),
        syn::Data::Union(DataUnion { union_token, .. }) => {
            abort!(union_token, "Unions are not supported for TypeToIdl")
        }
    };

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

fn idl_struct_type_def(fields: &Fields) -> TokenStream {
    let Paths {
        macro_prelude: prelude,
        ..
    } = &Paths::default();
    let tuple = matches!(fields, Fields::Unnamed(_));
    let idl_fields: Vec<TokenStream> = fields
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
                let field_name = LitStr::new(field_name.trim(), f.ident.span());

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

    quote! {
        #prelude::IdlTypeDef::Struct(vec![#(#idl_fields),*])
    }
}

fn idl_enum_type_def(data_enum: &syn::DataEnum, attributes: &[Attribute]) -> TokenStream {
    let Paths {
        macro_prelude: prelude,
        ..
    } = &Paths::default();
    let repr = get_repr(attributes);

    if !matches!(repr.repr.as_integer(), Some(IntegerRepr::U8)) {
        abort!(
            repr,
            "Enums must be `#[repr(u8)]` to be used with TypeToIdl"
        );
    }
    let repr = IntegerRepr::U8;

    let discriminants = enum_discriminants(data_enum.variants.iter());

    let idl_variants: Vec<_> = data_enum
        .variants
        .iter()
        .zip(discriminants)
        .map(|(v, ref d)| {
            let name = v.ident.to_string();
            let description = util::get_docs(&v.attrs);
            let type_def = if matches!(v.fields, Fields::Unit) {
                quote!(None)
            } else {
                let def = idl_struct_type_def(&v.fields);
                quote!(Some(#def))
            };
            let discriminant = discriminant_vec(d, repr);
            quote! {
                #prelude::IdlEnumVariant {
                    name: #name.to_string(),
                    discriminant: #discriminant,
                    description: #description,
                    type_def: #type_def,
                }
            }
        })
        .collect();
    quote! {
        #prelude::IdlTypeDef::Enum(vec![#(#idl_variants),*])
    }
}
