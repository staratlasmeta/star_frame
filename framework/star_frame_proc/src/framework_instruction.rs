use crate::util;
use crate::util::Paths;
use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::abort;
use quote::{quote, ToTokens};
use syn::{Attribute, Data, DeriveInput, LitStr, Visibility};

#[allow(dead_code)]
struct StrippedDeriveInput {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
}

pub fn derive_framework_instruction_impl(input: DeriveInput) -> TokenStream {
    let paths = Paths::default();

    match input.data {
        Data::Struct(s) => derive_framework_instruction_impl_struct(
            paths,
            s,
            StrippedDeriveInput {
                attrs: input.attrs,
                vis: input.vis,
                ident: input.ident,
            },
        ),
        Data::Enum(e) => abort!(
            e.enum_token,
            "FrameworkInstruction cannot be derived for enums"
        ),
        Data::Union(u) => abort!(
            u.union_token,
            "FrameworkInstruction cannot be derived for unions"
        ),
    }
}

fn derive_framework_instruction_impl_struct(
    paths: Paths,
    data_struct: syn::DataStruct,
    input: StrippedDeriveInput,
) -> TokenStream {
    let Paths {
        result,
        instruction_to_idl,
        idl_definition,
        idl_field,
        idl_type_def,
        idl_instruction_def,
        framework_instruction,
        type_to_idl,
        ..
    } = paths;

    let filter_variable_sized_arrays = |ty: &syn::Type| -> bool {
        if matches!(ty, syn::Type::Slice(_type_slice)) {
            return false;
        }
        true
    };

    let filtered_fields = data_struct
        .fields
        .iter()
        .filter(|field| filter_variable_sized_arrays(&field.ty))
        .collect::<Vec<_>>();

    let ident = &input.ident;

    let field_name = filtered_fields
        .iter()
        .enumerate()
        .map(|(index, field)| {
            field
                .ident
                .as_ref()
                .map(ToTokens::to_token_stream)
                .unwrap_or_else(|| syn::Index::from(index).into_token_stream())
        })
        .collect::<Vec<_>>();
    let field_type = filtered_fields
        .iter()
        .map(|field| &field.ty)
        .collect::<Vec<_>>();

    let field_docs: Vec<LitStr> = filtered_fields
        .iter()
        .map(|field| LitStr::new(&util::get_docs(&field.attrs), Span::call_site()))
        .collect();

    let field_str = field_name
        .iter()
        .map(|field_name| LitStr::new(&field_name.to_string(), Span::call_site()))
        .collect::<Vec<_>>();

    let out = quote! {
        #[automatically_derived]
        // TODO - Could these lifetimes ever be something else?
        impl <'a> #instruction_to_idl<'a, ()> for &'a #ident {
            fn instruction_to_idl(
                idl_definition: &mut #idl_definition,
                // TODO - Use idl struct args to pass in arg
                arg: (),
            ) -> #result<#idl_instruction_def> {
                #(let #field_name = <#field_type as #type_to_idl>::type_to_idl(idl_definition)?;)*
        Ok(#idl_instruction_def {
            account_set: <Self as #framework_instruction<'a>>::Accounts::account_set_to_idl(
                idl_definition,
                arg,
            )?,
            data: #idl_type_def::Struct(vec![#(
                #idl_field {
                            name: #field_str.to_string(),
                            description: #field_docs.to_string(),
                            path_id: #field_str.to_string(),
                            type_def: #field_name,
                            extension_fields: Default::default(),
                        },
                    )*
            ]),
        })
            }
        }
    };
    out
}
