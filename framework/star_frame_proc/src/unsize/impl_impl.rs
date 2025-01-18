use itertools::Itertools;
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::{format_ident, quote};
use syn::{parse_quote, FnArg, ImplItem, ItemImpl, Type, WherePredicate};

use crate::util::{new_generic, BetterGenerics, CombineGenerics, Paths};

pub fn unsized_impl_impl(item: ItemImpl, _args: TokenStream) -> TokenStream {
    Paths!(prelude);
    if let Some(trait_) = item.trait_ {
        abort!(trait_.1, "todo");
    }
    if !item.attrs.is_empty() {
        abort!(item.attrs[0], "todo");
    }
    if let Some(unsafety) = item.unsafety {
        abort!(unsafety, "todo");
    }

    if let Some(defaultness) = item.defaultness {
        abort!(defaultness, "todo");
    }
    let self_ty = item.self_ty;
    let Type::Path(self_ty) = self_ty.as_ref() else {
        abort!(self_ty, "todo");
    };

    let Some(trait_name) = self_ty.path.segments.last() else {
        abort!(self_ty.path, "");
    };

    let impl_trait_name = format_ident!("{}ImplTrait", trait_name.ident);
    let new_generic = new_generic(&item.generics);

    let mut_predicate: WherePredicate =
        parse_quote!(#new_generic: #prelude::Resize<<#self_ty as #prelude::UnsizedType>::RefMeta>);

    let functions = item
        .items
        .into_iter()
        .map(|item| {
            let ImplItem::Fn(mut item_fn) = item else {
                abort!(item, "todo");
            };
            let Some(first_input) = item_fn.sig.inputs.first() else {
                abort!(item_fn.sig, "todo");
            };
            let FnArg::Receiver(receiver) = first_input else {
                abort!(first_input, "todo");
            };
            if receiver.mutability.is_some() {
                item_fn
                    .sig
                    .generics
                    .make_where_clause()
                    .predicates
                    .push(mut_predicate.clone());
            }
            item_fn
        })
        .collect_vec();

    let fn_decls = functions.iter().map(|item| &item.sig);

    let new_generics = item.generics.combine::<BetterGenerics>(
        &parse_quote!([<#new_generic> where #new_generic: #prelude::AsBytes]),
    );

    let (impl_gen, ty_gen, where_clause) = new_generics.split_for_impl();

    quote! {
        pub trait #impl_trait_name #impl_gen #where_clause
        {
            #(#fn_decls;)*
        }

        impl #impl_gen #impl_trait_name #ty_gen for #prelude::RefWrapper<#new_generic, <#self_ty as #prelude::UnsizedType>::RefData>
            #where_clause
        {
            #(#functions)*
        }
    }
}
