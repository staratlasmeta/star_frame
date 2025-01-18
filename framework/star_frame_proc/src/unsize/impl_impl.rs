use proc_macro2::TokenStream;
use proc_macro_error::abort;
use syn::ItemImpl;

pub fn unsized_impl_impl(item: ItemImpl, args: TokenStream) -> TokenStream {
    if let Some(trait_) = item.trait_ {
        abort!(trait_.1, "todo");
    }
    
    let 
}
