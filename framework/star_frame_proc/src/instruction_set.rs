use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemEnum;

pub fn instruction_set_impl(item: ItemEnum, _args: TokenStream) -> TokenStream {
    quote! { #item }
}
