use crate::idl::{derive_type_to_idl_inner, TypeToIdlArgs};
use crate::util::{ensure_data_struct, reject_generics, Paths};
use easy_proc::{find_attr, ArgumentList};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{parse_quote, Attribute, DeriveInput, Visibility};

#[allow(dead_code)]
struct StrippedDeriveInput {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
}

pub fn derive_instruction_to_idl(input: DeriveInput) -> TokenStream {
    let Paths {
        instruction_to_idl_args_ident,
        macro_prelude: prelude,
        ..
    } = &Paths::default();
    reject_generics(
        &input,
        Some("Generics are not supported yet for InstructionToIdl"),
    );

    ensure_data_struct(&input, None);
    let ident = &input.ident;

    let args = find_attr(&input.attrs, instruction_to_idl_args_ident)
        .map(TypeToIdlArgs::parse_arguments)
        .unwrap_or_default();

    let type_to_idl_derivation = derive_type_to_idl_inner(&input, args);
    let mut generics = input.generics.clone();
    let where_clause = generics.make_where_clause();

    let generic_arg: Ident = format_ident!("__A");

    where_clause.predicates.push(
        parse_quote!(<Self as #prelude::StarFrameInstruction>::Accounts<'b, 'c, 'info>: #prelude::AccountSetToIdl<'info, #generic_arg>),
    );

    quote! {
        #type_to_idl_derivation

        #[automatically_derived]
        impl<'b, 'c, 'info, #generic_arg> #prelude::InstructionToIdl<#generic_arg> for #ident #where_clause {
            fn instruction_to_idl(idl_definition: &mut #prelude::IdlDefinition, arg: #generic_arg) -> Result<#prelude::IdlInstructionDef> {
                let account_set = <<#ident as #prelude::StarFrameInstruction>::Accounts<'b, 'c, 'info> as #prelude::AccountSetToIdl<'info, #generic_arg>>::account_set_to_idl(idl_definition, arg)?;
                let data = <#ident as #prelude::TypeToIdl>::type_to_idl(idl_definition)?;
                Ok(#prelude::IdlInstructionDef {
                    account_set,
                    definition: data,
                })
            }
        }
    }
}
