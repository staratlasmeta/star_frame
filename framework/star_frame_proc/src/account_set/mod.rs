use crate::util::Paths;
use easy_proc::{find_attr, ArgumentList};
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use syn::{Attribute, Data, DeriveInput, Ident, Visibility};

mod generics;
mod struct_impl;

#[allow(dead_code)]
struct StrippedDeriveInput {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
}

#[derive(ArgumentList, Default)]
struct AccountSetStructArgs {
    #[argument(presence)]
    skip_default_decode: bool,
    #[argument(presence)]
    skip_default_validate: bool,
    #[argument(presence)]
    skip_default_cleanup: bool,
    #[cfg(feature = "idl")]
    #[argument(presence)]
    skip_default_idl: bool,
}

pub fn derive_account_set_impl(input: DeriveInput) -> TokenStream {
    let paths = Paths::default();

    let account_set_generics = generics::account_set_generics(input.generics);
    let account_set_struct_args = find_attr(&input.attrs, &paths.account_set_ident)
        .map(AccountSetStructArgs::parse_arguments)
        .unwrap_or_default();

    match input.data {
        Data::Struct(s) => struct_impl::derive_account_set_impl_struct(
            paths,
            s,
            account_set_struct_args,
            StrippedDeriveInput {
                attrs: input.attrs,
                vis: input.vis,
                ident: input.ident,
            },
            account_set_generics,
        ),
        Data::Enum(e) => abort!(
            e.enum_token,
            "AccountSet cannot be derived for enums currently, will be supported later"
        ),
        Data::Union(u) => abort!(u.union_token, "AccountSet cannot be derived for unions"),
    }
}
