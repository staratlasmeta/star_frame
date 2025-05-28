use crate::util::{combine_gen, new_generic, new_lifetime, strip_inner_attributes, Paths};
use easy_proc::ArgumentList;
use heck::ToUpperCamelCase;
use itertools::Itertools;
use proc_macro2::{Ident, TokenStream};
use proc_macro_error2::{abort, OptionExt};
use quote::{format_ident, quote};
use syn::{
    parse_quote, AngleBracketedGenericArguments, FnArg, ImplItem, ImplItemFn, ItemImpl, Lifetime,
    LitStr, PathArguments, PathSegment, Receiver, Type, Visibility,
};

#[derive(ArgumentList)]
pub struct UnsizedImplArgs {
    tag: Option<LitStr>,
    ref_ident: Option<Ident>,
    mut_ident: Option<Ident>,
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

    let ptr_lt = new_lifetime(&item.generics, Some("ptr"));
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
        impl_args
            .ref_ident
            .unwrap_or_else(|| format_ident!("{self_ident}Ref")),
        ptr_lt.clone(),
    );
    let mut_ident = impl_args
        .mut_ident
        .unwrap_or_else(|| format_ident!("{self_ident}Mut"));
    let mut_ty = new_last_segment(mut_ident.clone(), ptr_lt.clone());

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
                    exclusive_fns.push(item_fn);
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

    let ref_mut_generics = combine_gen!(item.generics; <#ptr_lt>);
    let (impl_gen, _, where_clause) = ref_mut_generics.split_for_impl();
    let ref_impls = (!ref_fns.is_empty()).then(|| {
        quote! {
            impl #impl_gen #ref_ty #where_clause {
                #(#ref_fns)*
            }
        }
    });
    let mut_impls = (!mut_fns.is_empty()).then(|| {
        quote! {
            impl #impl_gen #mut_ty #where_clause {
                #(#mut_fns)*
            }
        }
    });

    let parent_lt = new_lifetime(&item.generics, Some("parent"));
    let top_lt = new_lifetime(&item.generics, Some("top"));
    let p = new_generic(&item.generics, Some("P"));
    let mut_ut = quote!(<#self_ty as #prelude::UnsizedType>::Mut<#top_lt>);
    let exclusive_trait_generics = combine_gen!(item.generics; <#parent_lt, #top_lt, #p>
        where Self: #prelude::ExclusiveRecurse + #sized,
    );
    let (impl_gen, ty_gen, where_clause) = exclusive_trait_generics.split_for_impl();
    let impl_for = quote!(#prelude::ExclusiveWrapper<#parent_lt, #top_lt, #mut_ut, #p>);

    let pub_exclusive_ident = format_ident!("{self_ident}ExclusiveImpl{tag_str}");
    let priv_exclusive_ident = format_ident!("{self_ident}ExclusiveImplPrivate{tag_str}");
    let mut pub_exclusive_fns = vec![];
    let mut priv_exclusive_fns = vec![];

    for mut exclusive_fn in exclusive_fns {
        match exclusive_fn.vis {
            Visibility::Public(_) => {
                exclusive_fn.vis = Visibility::Inherited;
                pub_exclusive_fns.push(exclusive_fn);
            }
            Visibility::Restricted(_) => abort!(
                exclusive_fn.vis,
                "`exclusive` functions can only have pub or inherited visibilities"
            ),
            Visibility::Inherited => {
                priv_exclusive_fns.push(exclusive_fn);
            }
        }
    }

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
        #ref_impls
        #mut_impls
        #pub_exclusive
        #priv_exclusive
    }
}
