mod unsized_enum;
mod unsized_struct;

use proc_macro2::TokenStream;
use syn::Item;

pub fn unsized_type_impl(item: Item, args: TokenStream) -> TokenStream {
    match item {
        Item::Enum(item) => unsized_enum::unsized_enum_impl(item, args),
        Item::Struct(item) => {
            let out = unsized_struct::unsized_struct_impl(item, args);
            // println!("{}", out);
            out
        }
        _ => panic!("Only enums and structs can be unsized"),
    }
}
