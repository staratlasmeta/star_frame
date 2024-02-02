use crate::get_crate_name;
use crate::util::Paths;
use proc_macro2::TokenStream;
use proc_macro_error::abort_call_site;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parse2, parse_quote, Expr, ItemStruct, Token};

struct ProgramArgs {
    list: Punctuated<Expr, Token![,]>,
}
impl Parse for ProgramArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            list: Punctuated::parse_terminated(input)?,
        })
    }
}

pub(crate) fn program_impl(item: ItemStruct, args: TokenStream) -> TokenStream {
    let Paths {
        pubkey,
        account_info,
        star_frame_program,
        program_result,
        ..
    } = Paths::default();
    let ident = &item.ident;
    let crate_name = get_crate_name();

    let args: ProgramArgs =
        parse2(args).unwrap_or_else(|e| abort_call_site!("expected a network: {}", e));

    if args.list.is_empty() {
        abort_call_site!("expected a network");
    }
    let network = &args.list[0];
    let entrypoint = if args.list.len() > 1 && args.list[1] == parse_quote! { no_entrypoint } {
        quote! {}
    } else {
        quote! { #crate_name::solana_program::entrypoint!(process_instruction); }
    };

    quote! {
        #item

        pub type StarFrameDeclaredProgram = #ident;

        #crate_name::static_assertions::assert_impl_all!(
            StarFrameDeclaredProgram: #star_frame_program
        );

        #[doc = r" The const program ID."]
        pub const ID: #pubkey = {
            match #crate_name::program::search_for_network(<#ident as #star_frame_program>::PROGRAM_IDS, #network) {
                Some(id) => id,
                None => {
                    panic!("Program ID not found for network");
                }
            }
        };

        #[doc = r" Returns `true` if given pubkey is the program ID."]
        pub fn check_id(id: &#pubkey) -> bool { id == &ID }

        #[doc = r" Returns the program ID."]
        pub const fn id() -> #pubkey { ID }

        #[cfg(test)]
        #[test]
        fn test_id() { assert!(check_id(&id())); }


        #[cfg(all(not(feature = "no-entrypoint"), any(target_os = "solana", feature = "fake_solana_os")))]
        mod entrypoint {
            use super::*;
            #entrypoint
            fn process_instruction(
                program_id: &#pubkey,
                accounts: &[#account_info],
                instruction_data: &[u8],
            ) -> #program_result {
                #crate_name::entrypoint::try_star_frame_entrypoint::<#ident>(program_id, accounts, instruction_data, #network)
                    .map_err(crate::errors::handle_error)
            }
        }
    }
}
