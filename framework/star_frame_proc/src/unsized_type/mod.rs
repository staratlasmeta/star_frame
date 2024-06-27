use easy_proc::proc_macro_error::abort_call_site;
use heck::ToUpperCamelCase;
use proc_macro2::Ident;
use proc_macro2::TokenStream;
use proc_macro_error::{abort, ResultExt};
use quote::{format_ident, quote, ToTokens};
use syn::parse::Nothing;
use syn::{parse_quote, Field, Item, ItemStruct};

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

#[derive(Debug, Clone)]
pub struct UnsizedTypeContext {
    pub item_struct: ItemStruct,
    pub sized_fields: Vec<Field>,
    pub unsized_fields: Vec<Field>,
    // pub generics: Vec<syn::GenericParam>,
}

impl UnsizedTypeContext {
    fn parse(mut item_struct: ItemStruct) -> Self {
        let unsized_start =
            strip_inner_attributes(&mut item_struct, "unsized_start").collect::<Vec<_>>();
        if unsized_start.is_empty() {
            abort!(item_struct, "No `unsized_start` attribute found");
        }
        if unsized_start.len() > 1 {
            abort!(
                unsized_start[1].attribute,
                "`unsized_start` can only start once!"
            );
        }
        let first_unsized = unsized_start[0].index;
        let all_fields = item_struct.fields.iter().cloned().collect::<Vec<_>>();
        let (sized_fields, unsized_fields) = all_fields.split_at(first_unsized);
        Self {
            item_struct,
            sized_fields: sized_fields.to_vec(),
            unsized_fields: unsized_fields.to_vec(),
        }
    }

    fn sized_ident(&self) -> Option<Ident> {
        if self.sized() {
            Some(format_ident!("{}Sized", self.item_struct.ident))
        } else {
            None
        }
    }

    fn sized(&self) -> bool {
        !self.sized_fields.is_empty()
    }
}

fn unsized_type_struct_impl(item_struct: syn::ItemStruct) -> TokenStream {
    let Paths {
        macro_prelude: prelude,
        deref,
        deref_mut,
        result,
        checked,
        ..
    } = Default::default();
    let context = UnsizedTypeContext::parse(item_struct);
    let sized_ident = context.sized_ident();
    let sized_ident = sized_ident.as_ref();

    let UnsizedTypeContext {
        sized_fields,
        unsized_fields,
        item_struct,
    } = context;

    let struct_ident = item_struct.ident.clone();

    let inner_ident = format_ident!("{struct_ident}Inner");
    let meta_ident = format_ident!("{struct_ident}Meta");
    let ref_ident = format_ident!("{struct_ident}Ref");
    let owned_ident = format_ident!("{struct_ident}Owned");
    let init_struct_ident = format_ident!("{struct_ident}Init");
    let sized_field_ident = format_ident!("sized_struct");

    let sized_field_idents = field_idents(&sized_fields);
    let unsized_field_idents = field_idents(&unsized_fields);
    let unsized_field_types: Vec<_> = unsized_fields.iter().map(|f| f.ty.clone()).collect();
    
    let combined_inner = combine_with_sized(sized_ident, &unsized_field_types, combine_unsized);
    
    let owned_fields = sized_ident
        .iter()
        .map(|sized_ident| {
            parse_quote!(
                #sized_field_ident: <#sized_ident as #prelude::UnsizedType>::Owned
            )
        })
        .chain(unsized_fields.iter().cloned().map(|mut field| {
            let field_ty = field.ty.clone();
            field.ty = parse_quote!(<#field_ty as #prelude::UnsizedType>::Owned);
            field
        }))
        .collect::<Vec<Field>>();
    
    let owned_field_idents = field_idents(&owned_fields);

    let unsized_field_generics = unsized_field_idents
        .iter()
        .map(|i| {
            let ident_string = i.to_string().to_upper_camel_case();
            let struct_ident_string = struct_ident.to_string().to_upper_camel_case();
            format_ident!("{struct_ident_string}{ident_string}")
        })
        .collect::<Vec<_>>();
    let combined_generics =
        combine_with_sized(sized_ident, &unsized_field_generics, with_parenthesis);

    let sized_field_accesses = sized_ident.map(|sized_ident| {
        quote! {
             #sized_ident {
                #(#sized_field_idents: arg.#sized_field_idents),*
            }
        }
    });
    let unsized_field_accesses = unsized_field_idents
        .iter()
        .map(|i| quote!(arg.#i))
        .collect::<Vec<_>>();
    let field_accesses = combine_with_sized(
        sized_field_accesses,
        &unsized_field_accesses,
        with_parenthesis,
    );
    
    let init_struct_with_generics = quote!(#init_struct_ident<#(#unsized_field_generics),*>);

    let combined_names = combine_with_sized(sized_ident.map(|_| &sized_field_ident), &unsized_field_idents, with_parenthesis);

    let sized_stuff = sized_ident.map(|sized_ident|
        quote! {
            #[derive(Debug, Copy, Clone, CheckedBitPattern, Zeroable, Align1, NoUninit, PartialEq, Eq)]
            #[repr(C, packed)]
            pub struct #sized_ident {
                #(#sized_fields),*
            }
            impl<__S: #prelude::AsBytes> #prelude::RefDeref<__S> for #ref_ident
            {
                type Target = #sized_ident;
                fn deref(wrapper: &#prelude::RefWrapper<__S, Self>) -> &Self::Target {
                    let bytes = wrapper.sup().as_bytes().expect("Invalid bytes");
                    #checked::from_bytes::<#sized_ident>(bytes)
                }
            }
    
            impl<__S: #prelude::AsMutBytes> #prelude::RefDerefMut<__S> for #ref_ident
            {
                fn deref_mut(wrapper: &mut #prelude::RefWrapper<__S, Self>) -> &mut Self::Target {
                    let bytes = unsafe { wrapper.sup_mut() }
                        .as_mut_bytes()
                        .expect("Invalid bytes");
                    #checked::from_bytes_mut::<#sized_ident>(bytes)
                }
            }
            
            impl #deref for #owned_ident
            {
                type Target = <#sized_ident as #prelude::UnsizedType>::Owned;
                fn deref(&self) -> &Self::Target {
                    &self.#sized_field_ident
                }
            }
    
            impl #deref_mut for #owned_ident
            {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.#sized_field_ident
                }
            }
        }
    );

    quote! {
        pub type #inner_ident = #combined_inner;

        #[derive(Debug, Align1)]
        #[repr(transparent)]
        pub struct #struct_ident(#inner_ident);

        #[derive(Debug, Copy, Clone)]
        #[repr(transparent)]
        pub struct #meta_ident(<#inner_ident as #prelude::UnsizedType>::RefMeta);

        #[derive(Debug, Copy, Clone)]
        #[repr(transparent)]
        pub struct #ref_ident(<#inner_ident as #prelude::UnsizedType>::RefData);

        #sized_stuff

        #[derive(Debug)]
        pub struct #owned_ident {
            #(#owned_fields),*
        }

        unsafe impl<__S: #prelude::AsBytes> #prelude::RefBytes<__S> for #ref_ident
        {
            fn bytes(wrapper: &#prelude::RefWrapper<__S, Self>) -> #result<&[u8]> {
                wrapper.sup().as_bytes()
            }
        }

        unsafe impl<__S: #prelude::AsMutBytes> #prelude::RefBytesMut<__S> for #ref_ident
        {
            fn bytes_mut(wrapper: &mut #prelude::RefWrapper<__S, Self>) -> #result<&mut [u8]> {
                unsafe { wrapper.sup_mut().as_mut_bytes() }
            }
        }

        unsafe impl<__S> #prelude::RefResize<__S, <#inner_ident as #prelude::UnsizedType>::RefMeta> for #ref_ident
        where
            __S: #prelude::Resize<#meta_ident>,
        {
            unsafe fn resize(
                wrapper: &mut #prelude::RefWrapper<__S, Self>,
                new_byte_len: usize,
                new_meta: <#inner_ident as #prelude::UnsizedType>::RefMeta,
            ) -> #result<()> {
                unsafe {
                    wrapper.r_mut().0 = #prelude::CombinedRef::new(new_meta);
                    wrapper
                        .sup_mut()
                        .resize(new_byte_len, #meta_ident(new_meta))
                }
            }

            unsafe fn set_meta(
                wrapper: &mut #prelude::RefWrapper<__S, Self>,
                new_meta: <#inner_ident as #prelude::UnsizedType>::RefMeta,
            ) -> #result<()> {
                unsafe {
                    wrapper.r_mut().0 = #prelude::CombinedRef::new(new_meta);
                    wrapper.sup_mut().set_meta(#meta_ident(new_meta))
                }
            }
        }

        impl #prelude::UnsizedInit<#prelude::Zeroed> for #struct_ident
        {
            const INIT_BYTES: usize = <#inner_ident as #prelude::UnsizedInit<#prelude::Zeroed>>::INIT_BYTES;

            unsafe fn init<__S: #prelude::AsMutBytes>(
                super_ref: __S,
                arg: #prelude::Zeroed,
            ) -> #result<(#prelude::RefWrapper<__S, Self::RefData>, Self::RefMeta)> {
                unsafe {
                    let (r, m) = <#inner_ident as #prelude::UnsizedInit<#prelude::Zeroed>>::init(super_ref, arg)?;
                    Ok((r.wrap_r(|_, r| #ref_ident(r)), #meta_ident(m)))
                }
            }
        }

        #[derive(Copy, Clone, Debug)]
        pub struct #init_struct_with_generics {
            #(#sized_fields,)*
            #(pub #unsized_field_idents: #unsized_field_generics),*
        }

        impl<#(#unsized_field_generics),*> #prelude::UnsizedInit<#init_struct_with_generics> for #struct_ident
        where
            #inner_ident: #prelude::UnsizedInit<#combined_generics>,
            #( #unsized_field_types: #prelude::UnsizedInit<#unsized_field_generics>,)*
        {
            const INIT_BYTES: usize = <#inner_ident as #prelude::UnsizedInit<#combined_generics>>::INIT_BYTES;

            unsafe fn init<__S: #prelude::AsMutBytes>(
                super_ref: __S,
                arg: #init_struct_with_generics,
            ) -> #result<(#prelude::RefWrapper<__S, Self::RefData>, Self::RefMeta)> {
                unsafe {
                    let (r, m) = <#inner_ident as #prelude::UnsizedInit<#combined_generics>>::init(
                        super_ref,
                        #field_accesses,
                    )?;
                    Ok((r.wrap_r(|_, r| #ref_ident(r)), #meta_ident(m)))
                }
            }
        }

        unsafe impl #prelude::UnsizedType for #struct_ident {
            type RefData = #ref_ident;
            type RefMeta = #meta_ident;
            type Owned = #owned_ident;
            type IsUnsized = <#inner_ident as #prelude::UnsizedType>::IsUnsized;

            unsafe fn from_bytes<__S: #prelude::AsBytes>(
                super_ref: __S
            ) -> #result<#prelude::FromBytesReturn<__S, Self::RefData, Self::RefMeta>> {
                unsafe {
                    Ok(
                        <#inner_ident as #prelude::UnsizedType>::from_bytes(super_ref)?
                            .map_ref(|_, r| #ref_ident(r))
                            .map_meta(#meta_ident)
                    )
                }
            }

            fn owned<__S: #prelude::AsBytes>(r: #prelude::RefWrapper<__S, Self::RefData>) -> #result <Self::Owned> {
                let #combined_names = <#inner_ident as #prelude::UnsizedType>::owned(unsafe { r.wrap_r(|_, r| r.0) })?;
                Ok(#owned_ident {
                    #(#owned_field_idents),*
                })
            }
        }
    }
}

fn field_idents(fields: &[Field]) -> Vec<syn::Ident> {
    fields
        .iter()
        .map(|field| field.ident.clone().expect("Unnamed field"))
        .collect()
}

fn combine_with_sized(
    sized: Option<impl ToTokens>,
    stuff: &[impl ToTokens],
    combine: impl Fn(&TokenStream, &TokenStream) -> TokenStream + Copy,
) -> TokenStream {
    let combined = combine_streams(stuff, combine);
    if let Some(sized) = sized {
        combine(&sized.to_token_stream(), &combined)
    } else {
        combined
    }
}

fn combine_streams(
    stuff: &[impl ToTokens],
    combine: impl Fn(&TokenStream, &TokenStream) -> TokenStream + Copy,
) -> TokenStream {
    if stuff.is_empty() {
        abort_call_site!("Tried to combine nothing!")
    }
    if stuff.len() == 1 {
        let stuff = &stuff[0];
        quote!(#stuff)
    } else {
        let half_mark = stuff.len().div_ceil(2);
        let first = combine_streams(&stuff[0..half_mark], combine);
        let second = combine_streams(&stuff[half_mark..], combine);
        combine(&first, &second)
    }
}

fn with_parenthesis(first: &TokenStream, second: &TokenStream) -> TokenStream {
    quote!((#first, #second))
}

fn combine_unsized(first: &TokenStream, second: &TokenStream) -> TokenStream {
    let Paths { macro_prelude, .. } = Paths::default();
    quote!(#macro_prelude::CombinedUnsized<#first, #second>)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combine() {
        let to_combine = vec![quote!(A)];
        let combined = combine_streams(&to_combine, with_parenthesis);
        assert_eq!(combined.to_string(), "A");

        let to_combine = vec![quote!(A), quote!(B)];
        let combined = combine_streams(&to_combine, with_parenthesis);
        assert_eq!(combined.to_string(), "(A , B)");

        let to_combine = vec![quote!(A), quote!(B), quote!(C), quote!(D), quote!(E)];
        let combined = combine_streams(&to_combine, with_parenthesis);
        assert_eq!(combined.to_string(), "(((A , B) , C) , (D , E))");

        let to_combine = vec![
            quote!(A),
            quote!(B),
            quote!(C),
            quote!(D),
            quote!(E),
            quote!(F),
        ];
        let combined = combine_streams(&to_combine, with_parenthesis);
        assert_eq!(combined.to_string(), "(((A , B) , C) , ((D , E) , F))");
    }

    #[test]
    fn test_combine_with_sized() {
        let to_combine = vec![quote!(A), quote!(B), quote!(C)];
        let sized = Some(quote!(Sized));
        let combined = combine_with_sized(sized.as_ref(), &to_combine, with_parenthesis);

        assert_eq!(combined.to_string(), "(Sized , ((A , B) , C))");
    }
}
