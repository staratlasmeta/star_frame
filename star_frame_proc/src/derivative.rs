use crate::util::{get_field_types, make_derivative_attribute, Paths};
use itertools::Itertools;
use syn::punctuated::Punctuated;
use syn::{parse_quote, ItemStruct, Path, Token};

pub fn derivative_impl(
    mut item_struct: ItemStruct,
    args: Punctuated<Path, Token![,]>,
) -> ItemStruct {
    Paths!(derivative);
    item_struct.attrs.push(parse_quote!(#[derive(#derivative)]));
    let attributes = make_derivative_attribute(args, &get_field_types(&item_struct).collect_vec());
    item_struct.attrs.push(attributes);
    item_struct
}
