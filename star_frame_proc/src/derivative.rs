use crate::util::{get_field_types, make_derivative_attribute, Paths};
use itertools::Itertools;
use proc_macro_error2::{OptionExt, ResultExt};
use syn::punctuated::Punctuated;
use syn::{parse_quote, ItemStruct, Path, Token};

pub fn derivative_impl(
    mut item_struct: ItemStruct,
    args: Punctuated<Path, Token![,]>,
) -> ItemStruct {
    Paths!(derivative);
    if !item_struct.attrs.iter().any(|attr| {
        if attr.path().is_ident("derive") {
            let derive_list: Result<Punctuated<Path, Token![,]>, _> =
                attr.parse_args_with(Punctuated::parse_terminated);
            let derive_list = derive_list.unwrap_or_abort();
            return derive_list.iter().any(|path| {
                path.segments
                    .last()
                    .expect_or_abort("Path should have at least one segment")
                    .ident
                    == "Derivative"
            });
        }
        false
    }) {
        item_struct.attrs.push(parse_quote!(#[derive(#derivative)]));
    }
    let attributes = make_derivative_attribute(args, &get_field_types(&item_struct).collect_vec());
    item_struct.attrs.push(attributes);
    item_struct
}
