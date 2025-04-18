use crate::idl::{derive_type_to_idl_inner, TypeToIdlArgs};
use crate::util::{ignore_cfg_module, reject_attributes, reject_generics, Paths};
use easy_proc::{find_attr, ArgumentList};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{parse_quote, DeriveInput};

pub fn derive_instruction_to_idl(input: &DeriveInput) -> TokenStream {
    let Paths {
        instruction_to_idl_args_ident,
        type_to_idl_args_ident,
        prelude,
        ..
    } = &Paths::default();
    reject_generics(
        input,
        Some("Generics are not supported yet for InstructionToIdl"),
    );

    let ident = &input.ident;

    let args = find_attr(&input.attrs, instruction_to_idl_args_ident)
        .map(TypeToIdlArgs::parse_arguments)
        .unwrap_or_default();

    reject_attributes(&input.attrs, type_to_idl_args_ident, None);
    let type_to_idl_derivation = derive_type_to_idl_inner(input, args);
    let mut generics = input.generics.clone();
    let where_clause = generics.make_where_clause();

    let generic_arg: Ident = format_ident!("__A");

    where_clause.predicates.push(
        parse_quote!(<Self as #prelude::StarFrameInstruction>::Accounts<'b, 'c, 'info>: #prelude::AccountSetToIdl<'info, #generic_arg>),
    );

    let ix_to_idl_impl = ignore_cfg_module(
        ident,
        "_instruction_to_idl",
        quote! {
            #[cfg(all(feature = "idl", not(target_os = "solana")))]
            #[automatically_derived]
            impl<'b, 'c, 'info, #generic_arg> #prelude::InstructionToIdl<#generic_arg> for #ident #where_clause {
                fn instruction_to_idl(idl_definition: &mut #prelude::IdlDefinition, arg: #generic_arg) -> Result<#prelude::IdlInstructionDef> {
                    let account_set = <<#ident as #prelude::StarFrameInstruction>::Accounts<'b, 'c, 'info> as #prelude::AccountSetToIdl<'info, #generic_arg>>::account_set_to_idl(idl_definition, arg)?;
                    let type_def = <#ident as #prelude::TypeToIdl>::type_to_idl(idl_definition)?;
                    let type_id = type_def.assert_defined()?.clone();
                    Ok(#prelude::IdlInstructionDef {
                        account_set,
                        type_id,
                    })
                }
            }
        },
    );

    quote! {
        #type_to_idl_derivation
        #ix_to_idl_impl
    }
}
