use crate::util::{BetterGenerics, Paths};
use easy_proc::{find_attr, ArgumentList};
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use syn::{Attribute, Data, DeriveInput, Ident, Visibility};

mod generics;
mod struct_impl;

#[allow(dead_code)]
#[derive(Debug)]
struct StrippedDeriveInput {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
}

#[derive(ArgumentList, Default, Debug)]
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
    generics: Option<BetterGenerics>,
}

#[derive(ArgumentList, Debug, Clone, Default)]
struct SingleAccountSetFieldArgs {
    #[argument(presence)]
    skip_signed_account: bool,
    #[argument(presence)]
    skip_writable_account: bool,
    #[argument(presence)]
    skip_has_program_account: bool,
    #[argument(presence)]
    skip_has_owner_program: bool,
    #[argument(presence)]
    skip_has_seeds: bool,
    #[argument(presence)]
    skip_can_set_seeds: bool,
    #[argument(presence)]
    skip_can_init_account: bool,
}

pub fn derive_account_set_impl(input: DeriveInput) -> TokenStream {
    let paths = Paths::default();

    let account_set_generics = generics::account_set_generics(input.generics);
    let account_set_struct_args = find_attr(&input.attrs, &paths.account_set_ident)
        .map(AccountSetStructArgs::parse_arguments)
        .unwrap_or_default();

    if let Some(attr) = find_attr(&input.attrs, &paths.single_account_set_ident) {
        abort!(
            attr,
            "`{}` can only be applied to a non-skipped struct field",
            paths.single_account_set_ident
        );
    };

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
