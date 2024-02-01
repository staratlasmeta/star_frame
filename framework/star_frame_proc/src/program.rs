use crate::get_crate_name;
use crate::util::Paths;
use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemStruct;

pub(crate) fn program_impl(item: ItemStruct, network: TokenStream) -> TokenStream {
    let Paths {
        pubkey,
        account_info,
        star_frame_program,
        program_result,
        ..
    } = Paths::default();
    let ident = &item.ident;
    let crate_name = get_crate_name();
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
            #crate_name::solana_program::entrypoint!(process_instruction);
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
