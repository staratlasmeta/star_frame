use crate::util::{combine_gen, new_generic, new_lifetime, strip_inner_attributes, Paths};
use easy_proc::ArgumentList;
use heck::ToUpperCamelCase;
use itertools::{Either, Itertools};
use proc_macro2::{Ident, TokenStream};
use proc_macro_error2::abort;
use quote::{format_ident, quote};
use syn::{parse_quote, FnArg, ImplItem, ImplItemFn, ItemImpl, LitStr, Receiver, Type, Visibility};

#[derive(ArgumentList)]
pub struct UnsizedImplArgs {
    tag: Option<LitStr>,
}

pub fn unsized_impl_impl(item: ItemImpl, args: TokenStream) -> TokenStream {
    let impl_args: UnsizedImplArgs =
        UnsizedImplArgs::parse_arguments(&parse_quote!(#[unsized_impl(#args)]));
    let tag_str = impl_args
        .tag
        .map(|tag| tag.value().to_upper_camel_case())
        .unwrap_or_default();
    Paths!(prelude, sized);
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

    let bad_path = |self_ty: &Type| {
        abort!(self_ty, "`unsized_impl` can only be used for type paths, i.e., `some_path::SomeType` or `SomeType`")
    };
    let Type::Path(self_ty_path) = self_ty.as_ref() else {
        bad_path(&self_ty)
    };
    if self_ty_path.qself.is_some() {
        bad_path(&self_ty);
    }

    let Some(self_segment) = self_ty_path.path.segments.last() else {
        abort!(
            self_ty_path.path,
            "Self type has no path segments. This shouldn't happen."
        );
    };

    let self_ident = self_segment.ident.clone();

    let impl_fns = item
        .items
        .into_iter()
        .map(|item| match item {
            ImplItem::Fn(item_fn) => item_fn,
            _ => abort!(item, "`unsized_impl` only supports methods"),
        })
        .collect_vec();

    let (mut pub_exclusive_fns, mut priv_exclusive_fns): (Vec<_>, Vec<_>) = impl_fns.into_iter()
        .partition_map(|mut item_fn| {
            if !matches!(item_fn.sig.inputs.first(), Some(FnArg::Receiver(Receiver { reference: Some(..), mutability: Some(..), .. }))) {
                abort!(item_fn.sig, "`unsized_impl` requires all methods take a mutable reference to self argument, i.e., `fn foo(&mut self, ...)`");
            }
            match item_fn.vis {
                Visibility::Public(_) => {
                    item_fn.vis = Visibility::Inherited;
                    Either::Left(item_fn)
                }
                Visibility::Restricted(_) => abort!(
                    item_fn.vis,
                    "`exclusive` functions can only have pub or inherited visibilities"
                ),
                Visibility::Inherited => {
                    Either::Right(item_fn)
                }
            }
        });

    let parent_lt = new_lifetime(&item.generics, Some("parent"));
    let top_lt = new_lifetime(&item.generics, Some("top"));
    let p = new_generic(&item.generics, Some("P"));
    let ptr_lt = quote!(<#self_ty as #prelude::UnsizedType>::Ptr);
    let exclusive_trait_generics = combine_gen!(item.generics; <#parent_lt, #top_lt, #p>
        where Self: #prelude::ExclusiveRecurse + #sized,
    );
    let (impl_gen, ty_gen, where_clause) = exclusive_trait_generics.split_for_impl();
    let impl_for = quote!(#prelude::ExclusiveWrapper<#parent_lt, #top_lt, #ptr_lt, #p>);

    let pub_exclusive_ident = format_ident!("{self_ident}ExclusiveImpl{tag_str}");
    let priv_exclusive_ident = format_ident!("{self_ident}ExclusiveImplPrivate{tag_str}");

    let make_exclusive = |vis: Visibility, trait_ident: Ident, funcs: &mut [ImplItemFn]| {
        let signatures = funcs
            .iter_mut()
            .map(|item| {
                let docs = strip_inner_attributes(&mut item.attrs, "doc")
                    .map(|doc| doc.attribute)
                    .collect_vec();
                let signature = item.sig.clone();
                quote! {
                    #(#docs)*
                    #signature;
                }
            })
            .collect_vec();
        quote! {
            #vis trait #trait_ident #impl_gen #where_clause
            {
                #(#signatures)*
            }

            impl #impl_gen #trait_ident #ty_gen for #impl_for #where_clause
            {
                #(#funcs)*
            }
        }
    };
    let pub_exclusive = (!pub_exclusive_fns.is_empty()).then(|| {
        make_exclusive(
            Visibility::Public(Default::default()),
            pub_exclusive_ident,
            &mut pub_exclusive_fns,
        )
    });
    let priv_exclusive = (!priv_exclusive_fns.is_empty()).then(|| {
        make_exclusive(
            Visibility::Inherited,
            priv_exclusive_ident,
            &mut priv_exclusive_fns,
        )
    });

    quote! {
        #pub_exclusive
        #priv_exclusive
    }
}
