use crate::util::Paths;
use crate::{get_crate_name, util};
use easy_proc::find_attr;
use proc_macro2::{Span, TokenStream};
use proc_macro_error::abort_call_site;
use quote::quote;
use syn::{DeriveInput, LitStr, Type};

pub fn derive_account_to_idl_impl(input: &DeriveInput) -> TokenStream {
    let Paths {
        macro_prelude: prelude,
        declared_program_type,
        result,
        ..
    } = &Paths::default();
    //
    // let associated_program = if let Some(attr) = find_attr(&input.attrs, idl_ty_program_ident) {
    //     attr.parse_args::<Type>()
    //         .unwrap_or_else(|e| abort_call_site!("Could not parse program type: {}", e))
    // } else {
    //     declared_program_type.clone()
    // };
    //
    // let crate_name = &get_crate_name();
    // let ident = &input.ident;
    // let ident_str = ident.to_string();
    // let type_docs = &util::get_docs(&input.attrs);
    // TODO - Update 'seeds' once we have a better way to handle seeds
    quote! {
        // #[automatically_derived]
        // impl #account_to_idl for #ident {
        //     type AssociatedProgram = #associated_program;
        //
        //     fn account_to_idl(idl_definition: &mut #prelude::IdlDefinition) -> #result<(#prelude::IdlAccountId)> {
        //         let namespace = if idl_definition.namespace == <Self::AssociatedProgram as #program_to_idl>::idl_namespace() {
        //             let ty = Self::type_to_idl(idl_definition)?;
        //             idl_definition.accounts.insert(
        //                 #ident_str.to_string(),
        //                 #prelude::IdlAccount {
        //                     name: #ident_str.to_string(),
        //                     description: #type_docs,
        //                     discriminant: #crate_name::serde_json::to_value(Self::DISCRIMINANT).expect("Failed to serialize discriminant"),
        //                     ty,
        //                     seeds: None,
        //                     extension_fields: Default::default(),
        //                 },
        //             );
        //             None
        //         } else {
        //             idl_definition.required_idl_definitions.insert(
        //                 <Self::AssociatedProgram as #program_to_idl>::idl_namespace().to_string(),
        //                 #prelude::IdlDefinitionReference {
        //                     version: Self::account_program_versions(),
        //                     namespace: <Self::AssociatedProgram as #program_to_idl>::idl_namespace().to_string(),
        //                 },
        //             );
        //             Some(<Self::AssociatedProgram as #program_to_idl>::idl_namespace().to_string())
        //         };
        //         Ok(#prelude::IdlAccountId {
        //             namespace,
        //             account_id: #ident_str.to_string(),
        //             extension_fields: Default::default(),
        //         })
        //     }
        // }
    }
}
