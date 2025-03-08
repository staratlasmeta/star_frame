use crate::util::{
    new_generic, new_lifetime, strip_inner_attributes, BetterGenerics, CombineGenerics, Paths,
};
use easy_proc::ArgumentList;
use heck::ToUpperCamelCase;
use itertools::Itertools;
use proc_macro2::{Ident, TokenStream};
use proc_macro_error2::{abort, OptionExt};
use quote::{format_ident, quote};
use syn::{
    parse_quote, AngleBracketedGenericArguments, FnArg, ImplItem, ImplItemFn, ItemImpl, Lifetime,
    LitStr, PathArguments, PathSegment, Receiver, Signature, Type, Visibility,
};

#[derive(ArgumentList)]
pub struct UnsizedImplArgs {
    tag: Option<LitStr>,
    ref_ident: Option<Ident>,
    mut_ident: Option<Ident>,
}

pub fn unsized_impl_impl(item: ItemImpl, args: TokenStream) -> TokenStream {
    let args: UnsizedImplArgs =
        UnsizedImplArgs::parse_arguments(&parse_quote!(#[unsized_impl(#args)]));
    let tag_str = args
        .tag
        .map(|tag| tag.value().to_upper_camel_case())
        .unwrap_or_default();
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

    let pub_exclusive_ident = format_ident!("{self_ident}ExtensionPub{tag_str}");
    let priv_exclusive_ident = format_ident!("{self_ident}Extension{tag_str}");

    let angle_bracketed_self = match &self_segment.arguments {
        PathArguments::None => AngleBracketedGenericArguments {
            colon2_token: None,
            lt_token: Default::default(),
            args: Default::default(),
            gt_token: Default::default(),
        },
        PathArguments::AngleBracketed(angle_bracketed) => angle_bracketed.clone(),
        PathArguments::Parenthesized(paren) => {
            abort!(
                paren,
                "Parenthesized path arguments are not allowed in type paths"
            );
        }
    };

    let underscore_lifetime: Lifetime = parse_quote!('_);
    let new_last_segment = |ident: Ident, lifetime: Lifetime| {
        let mut new_ty_path = self_ty_path.clone();
        let mut angle_generic = angle_bracketed_self.clone();
        angle_generic.args.insert(0, parse_quote!(#lifetime));
        let arguments = PathArguments::AngleBracketed(angle_generic);
        *new_ty_path
            .path
            .segments
            .last_mut()
            .expect_or_abort("Last segment is None") = PathSegment { ident, arguments };
        new_ty_path
    };
    let ref_ty = new_last_segment(
        args.ref_ident
            .unwrap_or_else(|| format_ident!("{self_ident}Ref")),
        underscore_lifetime.clone(),
    );
    let mut_ident = args
        .mut_ident
        .unwrap_or_else(|| format_ident!("{self_ident}Mut"));
    let mut_ty = new_last_segment(mut_ident.clone(), underscore_lifetime.clone());

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

    let mut exclusive_pub_fns = vec![];
    let mut exclusive_fns = vec![];
    let mut mut_fns = vec![];
    let mut ref_fns = vec![];
    impl_fns.into_iter()
        .for_each(|mut item_fn| {
            let Some(FnArg::Receiver(Receiver { reference: Some(..), mutability, .. })) = item_fn.sig.inputs.first() else {
                abort!(item_fn.sig, "`unsafe_impl` requires all methods take a reference to self argument, i.e., `fn foo(&self, ...)` or `fn foo(&mut self, ...)`");
            };

            let has_exclusive = strip_inner_attributes(&mut item_fn.attrs, &format_ident!("exclusive")).collect_vec();
            let has_exclusive = has_exclusive.first();
            let skip_mut = strip_inner_attributes(&mut item_fn.attrs, &format_ident!("skip_mut")).collect_vec();
            let skip_mut = skip_mut.first();

            if skip_mut.is_some() && has_exclusive.is_some() {
                abort!(skip_mut.unwrap().attribute, "`unsafe_impl` cannot have both exclusive and skip_mut");
            }

            if mutability.is_some() {
                if has_exclusive.is_some() {
                    match item_fn.vis {
                        Visibility::Public(_) => {
                            item_fn.vis = Visibility::Inherited;
                            exclusive_pub_fns.push(item_fn);
                        }
                        Visibility::Restricted(_) => abort!(item_fn.vis, "`exclusive` functions can only have pub or inherited visibilities"),
                        Visibility::Inherited => {
                            exclusive_fns.push(item_fn);
                        }
                    }
                } else {
                    mut_fns.push(item_fn);
                }
            } else {
                if has_exclusive.is_some() { abort!(has_exclusive.unwrap().attribute, "`exclusive` can only be on `&mut self` inherent functions"); }
                if skip_mut.is_none() {
                    mut_fns.push(item_fn.clone());
                }
                ref_fns.push(item_fn);
            }
        });

    let (impl_gen, _, where_clause) = item.generics.split_for_impl();
    let inherent_impls = quote! {
        impl #impl_gen #ref_ty #where_clause {
            #(#ref_fns)*
        }
        impl #impl_gen #mut_ty #where_clause {
            #(#mut_fns)*
        }
    };

    let pub_decls = exclusive_pub_fns.iter().map(|item| &item.sig).collect_vec();
    let priv_decls = exclusive_fns.iter().map(|item| &item.sig).collect_vec();

    let b_lt = new_lifetime(&item.generics, Some("b"));
    let a_lt = new_lifetime(&item.generics, Some("a"));
    let info_lt = new_lifetime(&item.generics, Some("info"));
    let o = new_generic(&item.generics, Some("O"));
    let a = new_generic(&item.generics, Some("A"));

    let exclusive_trait_generics = item.generics.combine::<BetterGenerics>(&parse_quote!([
        <#b_lt, #a_lt, #info_lt, #o, #a> where
            #o: #prelude::UnsizedType + ?Sized,
            #a: #prelude::UnsizedTypeDataAccess<#info_lt>
    ]));

    let (impl_gen, ty_gen, where_clause) = exclusive_trait_generics.split_for_impl();

    let mut_ty_a = new_last_segment(mut_ident, a_lt.clone());

    let make_exclusive = |vis: Visibility,
                          trait_ident: Ident,
                          decls: &[&Signature],
                          funcs: &[ImplItemFn]| {
        quote! {
            #vis trait #trait_ident #impl_gen #where_clause
            {
                #(#decls;)*
            }

            impl #impl_gen #trait_ident #ty_gen for #prelude::ExclusiveWrapperBorrowed<#b_lt, #a_lt, #info_lt, #mut_ty_a, #o, #a>
                #where_clause
            {
                #(#funcs)*
            }
        }
    };
    let pub_exclusive = (!exclusive_pub_fns.is_empty()).then(|| {
        make_exclusive(
            Visibility::Public(Default::default()),
            pub_exclusive_ident,
            &pub_decls,
            &exclusive_pub_fns,
        )
    });
    let priv_exclusive = (!exclusive_fns.is_empty()).then(|| {
        make_exclusive(
            Visibility::Inherited,
            priv_exclusive_ident,
            &priv_decls,
            &exclusive_fns,
        )
    });
    quote! {
        #inherent_impls
        #pub_exclusive
        #priv_exclusive
    }
}
