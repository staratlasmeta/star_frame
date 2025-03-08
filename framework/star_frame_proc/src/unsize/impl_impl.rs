use easy_proc::ArgumentList;
use itertools::Itertools;
use proc_macro2::{Ident, TokenStream};
use proc_macro_error2::{abort, OptionExt};
use quote::{format_ident, quote};
use syn::{
    parse_quote, AngleBracketedGenericArguments, FnArg, ImplItem, ItemImpl, LitStr, PathArguments,
    PathSegment, Type,
};

#[derive(ArgumentList)]
pub struct UnsizedImplArgs {
    _tag: Option<LitStr>,
    ref_ident: Option<Ident>,
    mut_ident: Option<Ident>,
}

pub fn unsized_impl_impl(item: ItemImpl, args: TokenStream) -> TokenStream {
    let args: UnsizedImplArgs =
        UnsizedImplArgs::parse_arguments(&parse_quote!(#[unsized_impl(#args)]));
    // let tag_str = args
    //     .tag
    //     .map(|tag| tag.value().to_upper_camel_case())
    //     .unwrap_or_default();
    // Paths!(prelude);
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

    // let pub_exclusive_impl = format_ident!("{self_ident}ExtensionPub{tag_str}");
    // let priv_exclusive_impl = format_ident!("{self_ident}Extension{tag_str}");

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

    let underscore_angled = {
        let mut angle_generic = angle_bracketed_self.clone();
        angle_generic.args.insert(0, parse_quote!('_));
        PathArguments::AngleBracketed(angle_generic)
    };
    let mut ref_ty = self_ty_path.clone();
    let mut mut_ty = self_ty_path.clone();
    let ref_ident = args
        .ref_ident
        .unwrap_or_else(|| format_ident!("{self_ident}Ref"));
    let mut_ident = args
        .mut_ident
        .unwrap_or_else(|| format_ident!("{self_ident}Mut"));
    *ref_ty
        .path
        .segments
        .last_mut()
        .expect_or_abort("Last segment is None") = PathSegment {
        ident: ref_ident.clone(),
        arguments: underscore_angled.clone(),
    };
    *mut_ty
        .path
        .segments
        .last_mut()
        .expect_or_abort("Last segment is None") = PathSegment {
        ident: mut_ident.clone(),
        arguments: underscore_angled.clone(),
    };

    // println!("{}", self_ty.to_token_stream());
    // println!("{}", ref_ty.to_token_stream());
    // println!("{}", mut_ty.to_token_stream());

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

    let mut self_fns = vec![];
    let mut mut_fns = vec![];
    let mut ref_fns = vec![];

    impl_fns.into_iter()
        .for_each(|item_fn| {
            let Some(FnArg::Receiver(receiver)) = item_fn.sig.inputs.first() else {
                abort!(item_fn.sig, "`unsafe_impl` requires all methods take a self argument, i.e., `fn foo(&self, ...)` or `fn foo(&mut self, ...)` or `fn foo(self, ...)`");
            };

            if receiver.reference.is_none() {
                self_fns.push(item_fn);
            } else if receiver.mutability.is_some() {
                mut_fns.push(item_fn);
            } else {
                ref_fns.push(item_fn);
            }
        });

    let (impl_gen, _, where_clause) = item.generics.split_for_impl();
    let inherent_impls = quote! {
        impl #impl_gen #ref_ty #where_clause {
            #(#ref_fns)*
        }
        impl #impl_gen #mut_ty #where_clause {
            #(#ref_fns)*
            #(#mut_fns)*
        }
    };

    inherent_impls

    // let ref_mut_generics = item
    //     .generics
    //     .combine::<BetterGenerics>(&parse_quote!([<'_>]));

    // println!("{}", ref_mut_generics.to_token_stream());

    //
    // let pub_decls = pub_fns.iter().map(|item| &item.sig).collect_vec();
    // let priv_decls = priv_fns.iter().map(|item| &item.sig).collect_vec();
    //
    // let new_generics = item.generics.combine::<BetterGenerics>(
    //     &parse_quote!([<#new_generic> where #new_generic: #prelude::AsBytes]),
    // );
    //
    // let (impl_gen, ty_gen, where_clause) = new_generics.split_for_impl();
    //
    // let make_impl = |vis: Visibility,
    //                  trait_ident: Ident,
    //                  decls: &[&Signature],
    //                  funcs: &[ImplItemFn]| {
    //     quote! {
    //         #vis trait #trait_ident #impl_gen #where_clause
    //         {
    //             #(#decls;)*
    //         }
    //
    //         impl #impl_gen #trait_ident #ty_gen for #prelude::RefWrapper<#new_generic, <#self_ty as #prelude::UnsizedType>::RefData>
    //             #where_clause
    //         {
    //             #(#funcs)*
    //         }
    //     }
    // };
    // let pub_impl = (!pub_fns.is_empty()).then(|| {
    //     make_impl(
    //         Visibility::Public(Default::default()),
    //         pub_impl_trait,
    //         &pub_decls,
    //         &pub_fns,
    //     )
    // });
    // let priv_impl = (!priv_fns.is_empty()).then(|| {
    //     make_impl(
    //         Visibility::Inherited,
    //         priv_impl_trait,
    //         &priv_decls,
    //         &priv_fns,
    //     )
    // });
    // quote! {
    //     #pub_impl
    //     #priv_impl
    // }
    // quote!()
}
