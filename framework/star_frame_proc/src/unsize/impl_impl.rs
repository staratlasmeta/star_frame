use crate::util::{new_generic, BetterGenerics, CombineGenerics, Paths};
use itertools::{Either, Itertools};
use proc_macro2::{Ident, TokenStream};
use proc_macro_error::abort;
use quote::{format_ident, quote};
use syn::{
    parse_quote, FnArg, ImplItem, ImplItemFn, ItemImpl, Signature, Type, Visibility, WherePredicate,
};

pub fn unsized_impl_impl(item: ItemImpl, _args: TokenStream) -> TokenStream {
    Paths!(prelude);
    if let Some(trait_) = item.trait_ {
        abort!(
            trait_.1,
            "`unsized_impl` can only be used for inherent impls"
        );
    }
    if !item.attrs.is_empty() {
        abort!(
            item.attrs[0],
            "`unsized_impl` cannot have attributes on the impl block"
        );
    }
    if let Some(unsafety) = item.unsafety {
        abort!(unsafety, "Inherent implementations cannot be unsafe");
    }

    if let Some(defaultness) = item.defaultness {
        abort!(defaultness, "Inherent implementations cannot be default");
    }
    let self_ty = item.self_ty;
    let Type::Path(self_ty) = self_ty.as_ref() else {
        abort!(self_ty, "`unsafe_impl` can only be used for type paths, i.e., `some_path::SomeType` or `SomeType`");
    };

    let Some(trait_name) = self_ty.path.segments.last() else {
        abort!(
            self_ty.path,
            "Self type has no path segments. This shouldn't happen."
        );
    };

    let pub_impl_trait = format_ident!("{}PubImpl", trait_name.ident);
    let priv_impl_trait = format_ident!("{}Impl", trait_name.ident);
    let new_generic = new_generic(&item.generics);

    let mut_predicate: WherePredicate =
        parse_quote!(#new_generic: #prelude::Resize<<#self_ty as #prelude::UnsizedType>::RefMeta>);

    let impl_fns = item
        .items
        .into_iter()
        .map(|item| match item {
            ImplItem::Fn(item_fn) => item_fn,
            _ => abort!(item, "`unsafe_impl` only supports methods"),
        })
        .collect_vec();

    if let Some(duplicate) = impl_fns.iter().duplicates_by(|item| &item.sig.ident).next() {
        abort!(
            duplicate.sig.ident,
            "Duplicate method name found in `unsized_impl`"
        );
    }

    let (pub_fns, priv_fns): (Vec<_>, Vec<_>) = impl_fns.into_iter()
        .partition_map(|mut item_fn| {
            let Some(FnArg::Receiver(receiver)) = item_fn.sig.inputs.first() else {
                abort!(item_fn.sig, "`unsafe_impl` requires all methods take a self argument, i.e., `fn foo(&self, ...)` or `fn foo(&mut self, ...)`");
            };

            let vis = item_fn.vis.clone();
            item_fn.vis = Visibility::Inherited;

            if receiver.mutability.is_some() {
                item_fn
                    .sig
                    .generics
                    .make_where_clause()
                    .predicates
                    .push(mut_predicate.clone());
            }

            match vis {
                Visibility::Restricted(_) => abort!(vis, "Only `pub` or private functions are supported for `unsized_impl`"),
                Visibility::Public(_) => Either::Left(item_fn),
                Visibility::Inherited => Either::Right(item_fn),
            }
        });

    let pub_decls = pub_fns.iter().map(|item| &item.sig).collect_vec();
    let priv_decls = priv_fns.iter().map(|item| &item.sig).collect_vec();

    let new_generics = item.generics.combine::<BetterGenerics>(
        &parse_quote!([<#new_generic> where #new_generic: #prelude::AsBytes]),
    );

    let (impl_gen, ty_gen, where_clause) = new_generics.split_for_impl();

    let make_impl = |vis: Visibility,
                     trait_ident: Ident,
                     decls: &[&Signature],
                     funcs: &[ImplItemFn]| {
        quote! {
            #vis trait #trait_ident #impl_gen #where_clause
            {
                #(#decls;)*
            }

            impl #impl_gen #trait_ident #ty_gen for #prelude::RefWrapper<#new_generic, <#self_ty as #prelude::UnsizedType>::RefData>
                #where_clause
            {
                #(#funcs)*
            }
        }
    };
    let pub_impl = (!pub_fns.is_empty()).then(|| {
        make_impl(
            Visibility::Public(Default::default()),
            pub_impl_trait,
            &pub_decls,
            &pub_fns,
        )
    });
    let priv_impl = (!priv_fns.is_empty()).then(|| {
        make_impl(
            Visibility::Inherited,
            priv_impl_trait,
            &priv_decls,
            &priv_fns,
        )
    });
    quote! {
        #pub_impl
        #priv_impl
    }
}
