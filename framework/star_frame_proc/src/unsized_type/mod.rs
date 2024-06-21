use easy_proc::proc_macro_error::abort_call_site;
use proc_macro2::TokenStream;
use proc_macro_error::{abort, ResultExt};
use quote::{format_ident, quote, ToTokens};
use syn::parse::Nothing;
use syn::{parse_quote, Field, Item};

use crate::util::{strip_inner_attributes, Paths};

pub fn unsized_type_impl(item: Item, args: TokenStream) -> TokenStream {
    syn::parse2::<Nothing>(args.clone()).expect_or_abort("`unsized_type` takes no arguments");

    match item {
        Item::Struct(struct_item) => unsized_type_struct_impl(struct_item),
        Item::Enum(_enum_item) => {
            abort!(
                args,
                "unsized_type cannot be applied to enums yet. It will be supported in the future. (soonTM)"
            )
        }
        _ => {
            abort!(
                args,
                "unsized_type can only be applied to structs and enums"
            )
        }
    }
}

fn unsized_type_struct_impl(mut struct_item: syn::ItemStruct) -> TokenStream {
    let unsized_start =
        strip_inner_attributes(&mut struct_item, "unsized_start").collect::<Vec<_>>();
    if unsized_start.is_empty() {
        abort!(struct_item, "No `unsized_start` attribute found");
    }
    if unsized_start.len() > 1 {
        abort!(
            unsized_start[1].attribute,
            "`unsized_start` can only start once!"
        );
    }

    let first_unsized = unsized_start[0].index;

    if first_unsized == 0 {
        abort!(
            struct_item,
            "No sized fields found before the unsized field. Figure this shit out later"
        );
    }

    let all_fields = struct_item.fields.iter().collect::<Vec<_>>();
    let (sized_fields, unsized_fields) = all_fields.split_at(first_unsized);

    let struct_ident = struct_item.ident.clone();

    let sized_ident = format_ident!("{}Sized", struct_ident);
    let sized_struct = quote! {
        #[derive(Debug, Copy, Clone, CheckedBitPattern, Zeroable, Align1, NoUninit, PartialEq, Eq)]
        #[repr(C, packed)]
        pub struct #sized_ident {
            #(#sized_fields),*
        }
    };

    let Paths {
        combined_unsized,
        unsized_type,
        deref,
        deref_mut,
        ..
    } = Default::default();

    let combined_inner = combine_unsized(unsized_fields);
    let inner_ident = format_ident!("{}Inner", struct_ident);
    let combined_inner = quote!(
        type #inner_ident = #combined_unsized<#sized_ident, #combined_inner>;
    );

    let main_struct = quote! {
        #[derive(Debug, Align1)]
        #[repr(transparent)]
        pub struct #struct_ident(#inner_ident);
    };

    let meta_ident = format_ident!("{}Meta", struct_ident);
    let meta_struct = quote! {
        #[derive(Debug, Copy, Clone)]
        #[repr(transparent)]
        pub struct #meta_ident(<#inner_ident as #unsized_type>::RefMeta);
    };

    let ref_ident = format_ident!("{}Ref", struct_ident);
    let ref_struct = quote! {
        #[derive(Debug, Copy, Clone)]
        #[repr(transparent)]
        pub struct #ref_ident(<#inner_ident as #unsized_type>::RefData);
    };

    let owned_ident = format_ident!("{}Owned", struct_ident);
    let owned_fields = unsized_fields
        .iter()
        .map(|field| {
            let mut new_field = (*field).clone();
            let field_ty = &field.ty;
            new_field.ty = parse_quote!(<#field_ty as #unsized_type>::Owned);
            new_field
        })
        .collect::<Vec<_>>();

    let owned_struct = quote! {
        #[derive(Debug)]
        pub struct #owned_ident {
            sized_struct: <#sized_ident as #unsized_type>::Owned,
            #(#owned_fields),*
        }

        impl #deref for #owned_ident
        {
            type Target = <#sized_ident as #unsized_type>::Owned;
            fn deref(&self) -> &Self::Target {
                &self.sized_struct
            }
        }

        impl #deref_mut for #owned_ident
        {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.sized_struct
            }
        }
    };

    quote! {
        #sized_struct

        #combined_inner

        #main_struct

        #meta_struct

        #ref_struct

        #owned_struct
    }
}

fn combine_unsized(fields: &[&Field]) -> TokenStream {
    if fields.is_empty() {
        abort_call_site!("Tried to combine no fields!");
    }
    if fields.len() == 1 {
        let field_ty = &fields[0].ty;
        quote!(#field_ty)
    } else {
        let half_mark = fields.len().div_ceil(2);
        let first = combine_unsized(&fields[0..half_mark]);
        let second = combine_unsized(&fields[half_mark..]);
        let Paths {
            combined_unsized, ..
        } = Paths::default();
        quote!(#combined_unsized < #first, #second >)
    }
}
