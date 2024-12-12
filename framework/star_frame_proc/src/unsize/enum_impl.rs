use crate::unsize::UnsizedTypeArgs;
use crate::util::enum_discriminants;
use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemEnum;

pub(crate) fn unsized_type_struct_impl(
    item_enum: ItemEnum,
    unsized_args: UnsizedTypeArgs,
) -> TokenStream {
    let discriminants = enum_discriminants(item_enum.variants.iter());
    quote! {}
}
