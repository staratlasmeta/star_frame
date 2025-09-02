use crate::util::{ignore_cfg_module, new_generic, reject_generics, Paths};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, DeriveInput};

pub fn derive_instruction_to_idl(input: &DeriveInput) -> TokenStream {
    Paths!(prelude);
    reject_generics(
        input,
        Some("Generics are not supported yet for InstructionToIdl"),
    );

    let ident = &input.ident;

    let mut generics = input.generics.clone();
    let where_clause = generics.make_where_clause();

    let generic_arg = new_generic(&input.generics, None);

    where_clause.predicates.push(
        parse_quote!(<Self as #prelude::StarFrameInstruction>::Accounts<'decode, 'arg>: #prelude::AccountSetToIdl<#generic_arg>),
    );

    ignore_cfg_module(
        ident,
        "_instruction_to_idl",
        quote! {
            #[cfg(all(feature = "idl", not(target_os = "solana")))]
            #[automatically_derived]
            impl<'decode, 'arg, #generic_arg> #prelude::InstructionToIdl<#generic_arg> for #ident #where_clause {
                fn instruction_to_idl(idl_definition: &mut #prelude::IdlDefinition, arg: #generic_arg) -> Result<#prelude::IdlInstructionDef> {
                    let account_set = <<#ident as #prelude::StarFrameInstruction>::Accounts<'decode, 'arg> as #prelude::AccountSetToIdl<#generic_arg>>::account_set_to_idl(idl_definition, arg)?;
                    let type_def = <#ident as #prelude::TypeToIdl>::type_to_idl(idl_definition)?;
                    let type_id = type_def.assert_defined()?.clone();
                    Ok(#prelude::IdlInstructionDef {
                        account_set,
                        type_id,
                    })
                }
            }
        },
    )
}
