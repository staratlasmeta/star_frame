use crate::util::{
    generate_fields_are_trait, get_field_types, strip_inner_attributes, BetterGenerics,
    CombineGenerics, GetFields, Paths,
};
use easy_proc::ArgumentList;
use heck::ToUpperCamelCase;
use proc_macro2::Ident;
use proc_macro2::TokenStream;
use proc_macro_error::{abort, abort_call_site};
use quote::{format_ident, quote, ToTokens};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{
    parse_quote, Attribute, Field, GenericParam, Generics, ImplGenerics, Item, ItemStruct,
    TypeParam, WhereClause, WherePredicate,
};

pub fn unsized_type_impl(item: Item, args: TokenStream) -> TokenStream {
    // syn::parse2::<Nothing>(args.clone()).expect_or_abort("`unsized_type` takes no arguments");
    match item {
        Item::Struct(struct_item) => unsized_type_struct_impl(struct_item, args),
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

#[derive(ArgumentList, Default, Debug, Clone)]
pub struct UnsizedTypeArgs {
    #[argument(default)]
    pub sized_generics: BetterGenerics,
    #[argument(default)]
    pub unsized_generics: BetterGenerics,
}

impl UnsizedTypeArgs {
    fn combined_generics(&self) -> Generics {
        let sized_generics = self.sized_generics.clone().into_inner();
        let unsized_generics = self.unsized_generics.clone().into_inner();

        sized_generics.combine(unsized_generics)
    }
}

#[derive(Debug, Clone)]
pub struct UnsizedTypeContext {
    pub item_struct: ItemStruct,
    pub sized_fields: Vec<Field>,
    pub unsized_fields: Vec<Field>,
    pub args: UnsizedTypeArgs,
    // pub generics: Vec<syn::GenericParam>,
}

impl UnsizedTypeContext {
    fn parse(mut item_struct: ItemStruct, args: TokenStream) -> Self {
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

        if matches!(item_struct.fields, syn::Fields::Unnamed(_)) {
            abort!(item_struct.fields, "Unnamed fields are not supported")
        }

        let first_unsized = unsized_start[0].index;
        let all_fields = item_struct.fields.iter().cloned().collect::<Vec<_>>();
        let (sized_fields, unsized_fields) = all_fields.split_at(first_unsized);

        let args_attr: Attribute = parse_quote!(#[unsized_type(#args)]);
        let args = UnsizedTypeArgs::parse_arguments(&args_attr);

        Self {
            item_struct,
            args,
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

fn derive_bytemucks(sized_struct: &ItemStruct, sized_generics: &Generics) -> TokenStream {
    let Paths {
        bytemuck,
        derivative,
        ..
    } = Default::default();
    let (impl_generics, type_generics, where_clause) = sized_generics.split_for_impl();
    let struct_ident = sized_struct.ident.clone();

    let bit_ident = format_ident!("{}Bits", struct_ident);
    let mut bit_fields = sized_struct.fields.clone();
    bit_fields.iter_mut().for_each(|f| {
        let field_ty = &f.ty;
        f.ty = parse_quote!(<#field_ty as #bytemuck::CheckedBitPattern>::Bits);
    });

    let bit_checks = sized_struct.fields.iter().map(|f| {
        let field_ty = &f.ty;
        let field_ident = f.ident.as_ref().expect("Unnamed field");
        quote! {
            <#field_ty as #bytemuck::CheckedBitPattern>::is_valid_bit_pattern(&{ bits.#field_ident })
        }
    }).collect::<Vec<_>>();

    let ensure_no_uninit =
        generate_fields_are_trait(sized_struct, parse_quote!(#bytemuck::NoUninit));

    let ensure_checked =
        generate_fields_are_trait(sized_struct, parse_quote!(#bytemuck::CheckedBitPattern));

    let derivitive_bounds = get_field_types(&bit_fields)
        .map(|ty| quote!(#ty: Debug))
        .collect::<Vec<_>>();
    let derivitive_bounds = quote!(#(#derivitive_bounds)*).to_string();
    let bit_fields = bit_fields.iter();
    quote! {
        #ensure_no_uninit
        #ensure_checked

        unsafe impl #impl_generics NoUninit for #struct_ident #type_generics #where_clause {}

        #[repr(C, packed)]
        #[derive(Clone, Copy, #bytemuck::AnyBitPattern, #derivative)]
        #[derivative(Debug(bound = #derivitive_bounds))]
        pub struct #bit_ident #type_generics #where_clause {
            #(#bit_fields),*
        }

        unsafe impl #impl_generics #bytemuck::CheckedBitPattern for #struct_ident #type_generics #where_clause {
            type Bits = #bit_ident #type_generics;
            #[inline]
            #[allow(clippy::double_comparisons)]
            fn is_valid_bit_pattern(bits: &Self::Bits) -> bool {
                #(#bit_checks &&)* true
            }
        }
    }
}

fn unsized_type_struct_impl(item_struct: ItemStruct, args: TokenStream) -> TokenStream {
    let Paths {
        macro_prelude: prelude,
        deref,
        deref_mut,
        result,
        checked,
        size_of,
        ..
    } = Default::default();
    let context = UnsizedTypeContext::parse(item_struct, args);
    let sized_ident = context.sized_ident();

    let UnsizedTypeContext {
        sized_fields,
        unsized_fields,
        item_struct,
        args,
    } = context;

    let combined_generics = args.combined_generics();
    let (combined_impl_generics, combined_type_generics, combined_where) =
        combined_generics.split_for_impl();
    let (_, sized_type_generics, sized_where) = args.sized_generics.split_for_impl();

    let struct_ident = item_struct.ident.clone();
    let struct_type = quote!(#struct_ident #combined_type_generics);
    let inner_ident = format_ident!("{struct_ident}Inner");
    let inner_type = quote!(#inner_ident #combined_type_generics);
    let meta_ident = format_ident!("{struct_ident}Meta");
    let meta_type = quote!(#meta_ident #combined_type_generics);
    let ref_ident = format_ident!("{struct_ident}Ref");
    let ref_type = quote!(#ref_ident #combined_type_generics);
    let owned_ident = format_ident!("{struct_ident}Owned");
    let owned_type = quote!(#owned_ident #combined_type_generics);
    let sized_ident = sized_ident.as_ref();
    let sized_type = sized_ident.map(|sized_ident| {
        quote! {
            #sized_ident #sized_type_generics
        }
    });
    let sized_type = sized_type.as_ref();

    let sized_field_ident = format_ident!("sized_struct");
    let init_struct_ident = format_ident!("{struct_ident}Init");

    let sized_field_idents = field_idents(&sized_fields);
    let unsized_field_idents = field_idents(&unsized_fields);
    let unsized_field_types: Vec<_> = unsized_fields.iter().map(|f| f.ty.clone()).collect();

    let combined_inner = combine_with_sized(sized_type, &unsized_field_types, combine_unsized);

    let owned_fields = sized_type
        .iter()
        .map(|sized_type| {
            parse_quote!(
                #sized_field_ident: <#sized_type as #prelude::UnsizedType>::Owned
            )
        })
        .chain(unsized_fields.iter().cloned().map(|mut field| {
            let field_ty = field.ty.clone();
            field.ty = parse_quote!(<#field_ty as #prelude::UnsizedType>::Owned);
            field
        }))
        .collect::<Vec<Field>>();

    let owned_field_idents = field_idents(&owned_fields);

    let combined_names = combine_with_sized(
        sized_ident.map(|_| &sized_field_ident),
        &unsized_field_idents,
        with_parenthesis,
    );

    let s = quote!(__S);
    let as_bytes_generic: TypeParam = parse_quote!(#s: #prelude::AsBytes);
    let mut as_bytes_combined = combined_generics.clone();
    let as_bytes_impl_generics =
        impl_generics_with(&mut as_bytes_combined, as_bytes_generic.clone());

    let as_mut_bytes_generic: TypeParam = parse_quote!(#s: #prelude::AsMutBytes);
    let mut as_mut_bytes_combined = combined_generics.clone();
    let as_mut_bytes_impl_generics =
        impl_generics_with(&mut as_mut_bytes_combined, as_mut_bytes_generic.clone());

    let ref_resize_generic: TypeParam = parse_quote!(#s: #prelude::Resize<#meta_type>);
    let mut ref_resize_combined = combined_generics.clone();
    let ref_resize_impl_generics =
        impl_generics_with(&mut ref_resize_combined, ref_resize_generic.clone());

    let init_zeroed_generics = Generics {
        where_clause: Some(
            parse_quote!(where #inner_type: #prelude::UnsizedInit<#prelude::Zeroed>),
        ),
        ..Default::default()
    };

    let init_zeroed_generics = combined_generics.combine(init_zeroed_generics);
    let (_, _, init_zeroed_where) = init_zeroed_generics.split_for_impl();

    let init_generic_idents: Vec<_> = unsized_field_idents
        .iter()
        .map(|i| {
            let ident_string = i.to_string().to_upper_camel_case();
            let struct_ident_string = struct_ident.to_string().to_upper_camel_case();
            format_ident!("{struct_ident_string}{ident_string}")
        })
        .collect();

    let init_type_params: Punctuated<GenericParam, _> = init_generic_idents
        .iter()
        .map::<GenericParam, _>(|i| parse_quote!(#i))
        .collect();

    let unsized_init_type_generics =
        combine_with_sized(sized_type, &init_generic_idents, with_parenthesis);

    let unsized_init_where = WhereClause {
        predicates: init_generic_idents
            .iter()
            .zip(unsized_field_types.iter())
            .map(|(generic, field_type)| parse_quote!(#field_type: #prelude::UnsizedInit<#generic>))
            .chain(std::iter::once::<WherePredicate>(
                parse_quote!(#inner_type: #prelude::UnsizedInit<#unsized_init_type_generics>),
            ))
            .collect(),
        where_token: Default::default(),
    };

    let unsized_init_generics = Generics {
        params: init_type_params,
        where_clause: Some(unsized_init_where),
        ..Default::default()
    };

    let combined_unsized_init_generics = combined_generics.combine(unsized_init_generics.clone());
    let (unsized_init_impl_generics, _, init_where_clause) =
        combined_unsized_init_generics.split_for_impl();

    let init_struct_generics = args.sized_generics.combine(unsized_init_generics.clone());
    let (_, init_struct_type_generics, _) = init_struct_generics.split_for_impl();

    let init_struct_type = quote!(#init_struct_ident #init_struct_type_generics);

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

    let init_with_struct = quote! {
        #[derive(Copy, Clone, Debug)]
        pub struct #init_struct_type #sized_where {
            #(#sized_fields,)*
            #(pub #unsized_field_idents: #init_generic_idents),*
        }

        impl #unsized_init_impl_generics #prelude::UnsizedInit<#init_struct_type> for #struct_type #init_where_clause
        {
            const INIT_BYTES: usize = <#inner_type as #prelude::UnsizedInit<#unsized_init_type_generics>>::INIT_BYTES;

            unsafe fn init<#as_mut_bytes_generic>(
                super_ref: #s,
                arg: #init_struct_type,
            ) -> #result<(#prelude::RefWrapper<#s, Self::RefData>, Self::RefMeta)> {
                unsafe {
                    let (r, m) = <#inner_type as #prelude::UnsizedInit<#unsized_init_type_generics>>::init(
                        super_ref,
                        #field_accesses,
                    )?;
                    Ok((r.wrap_r(|_, r| #ref_ident(r)), #meta_ident(m)))
                }
            }
        }
    };

    // We can only derive CheckedBitPattern and NoUninit if there are no generics. This restriction may be relaxed in the future in bytemuck
    let sized_bytemuck_derives = if args.sized_generics.params.is_empty() {
        quote!(CheckedBitPattern, NoUninit)
    } else {
        Default::default()
    };

    let sized_struct: ItemStruct = parse_quote! {
        #[derive(Debug, Copy, Clone, Zeroable, Align1, PartialEq, Eq, #sized_bytemuck_derives)]
        #[repr(C, packed)]
        pub struct #sized_type #sized_where {
            #(#sized_fields),*
        }
    };

    let sized_bytemuck_impls = if !args.sized_generics.params.is_empty() {
        derive_bytemucks(&sized_struct, &args.sized_generics)
    } else {
        Default::default()
    };

    let sized_stuff = sized_type.map(|sized_type| {
        quote! {
            #sized_struct
            #sized_bytemuck_impls

            impl #as_bytes_impl_generics #prelude::RefDeref<#s> for #ref_type #combined_where
            {
                type Target = #sized_type;
                fn deref(wrapper: &#prelude::RefWrapper<#s, Self>) -> &Self::Target {
                    let bytes = wrapper.sup().as_bytes().expect("Invalid bytes");
                    let bytes = &bytes[..#size_of::<#sized_type>()];
                    #checked::from_bytes::<#sized_type>(bytes)
                }
            }

            impl #as_mut_bytes_impl_generics #prelude::RefDerefMut<#s> for #ref_type #combined_where
            {
                fn deref_mut(wrapper: &mut #prelude::RefWrapper<#s, Self>) -> &mut Self::Target {
                    let bytes = unsafe { wrapper.sup_mut() }
                        .as_mut_bytes()
                        .expect("Invalid bytes");
                    let bytes = &mut bytes[..#size_of::<#sized_type>()];
                    #checked::from_bytes_mut::<#sized_type>(bytes)
                }
            }

            impl #combined_impl_generics #deref for #owned_type #combined_where
            {
                type Target = <#sized_type as #prelude::UnsizedType>::Owned;
                fn deref(&self) -> &Self::Target {
                    &self.#sized_field_ident
                }
            }

            impl #combined_impl_generics #deref_mut for #owned_type #combined_where
            {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.#sized_field_ident
                }
            }
        }
    });

    quote! {
        pub type #inner_type = #combined_inner;

        #[derive(Debug, Align1)]
        #[repr(transparent)]
        pub struct #struct_type(#inner_type) #combined_where;

        #[derive(Debug, Copy, Clone)]
        #[repr(transparent)]
        pub struct #meta_type(<#inner_type as #prelude::UnsizedType>::RefMeta) #combined_where;

        #[derive(Debug, Copy, Clone)]
        #[repr(transparent)]
        pub struct #ref_type(<#inner_type as #prelude::UnsizedType>::RefData) #combined_where;

        #sized_stuff

        #[derive(Debug)]
        pub struct #owned_type #combined_where {
            #(#owned_fields),*
        }

        unsafe impl #as_bytes_impl_generics #prelude::RefBytes<#s> for #ref_type #combined_where
        {
            fn bytes(wrapper: &#prelude::RefWrapper<#s, Self>) -> #result<&[u8]> {
                wrapper.sup().as_bytes()
            }
        }

        unsafe impl #as_mut_bytes_impl_generics #prelude::RefBytesMut<#s> for #ref_type #combined_where
        {
            fn bytes_mut(wrapper: &mut #prelude::RefWrapper<#s, Self>) -> #result<&mut [u8]> {
                unsafe { wrapper.sup_mut().as_mut_bytes() }
            }
        }

        unsafe impl #ref_resize_impl_generics #prelude::RefResize<#s, <#inner_type as #prelude::UnsizedType>::RefMeta> for #ref_type #combined_where
        {
            unsafe fn resize(
                wrapper: &mut #prelude::RefWrapper<#s, Self>,
                new_byte_len: usize,
                new_meta: <#inner_type as #prelude::UnsizedType>::RefMeta,
            ) -> #result<()> {
                unsafe {
                    wrapper.r_mut().0 = #prelude::CombinedRef::new(new_meta);
                    wrapper
                        .sup_mut()
                        .resize(new_byte_len, #meta_ident(new_meta))
                }
            }

            unsafe fn set_meta(
                wrapper: &mut #prelude::RefWrapper<#s, Self>,
                new_meta: <#inner_type as #prelude::UnsizedType>::RefMeta,
            ) -> #result<()> {
                unsafe {
                    wrapper.r_mut().0 = #prelude::CombinedRef::new(new_meta);
                    wrapper.sup_mut().set_meta(#meta_ident(new_meta))
                }
            }
        }

        impl #combined_impl_generics #prelude::UnsizedInit<#prelude::Zeroed> for #struct_type #init_zeroed_where
        {
            const INIT_BYTES: usize = <#inner_type as #prelude::UnsizedInit<#prelude::Zeroed>>::INIT_BYTES;

            unsafe fn init<#as_mut_bytes_generic>(
                super_ref: #s,
                arg: #prelude::Zeroed,
            ) -> #result<(#prelude::RefWrapper<#s, Self::RefData>, Self::RefMeta)> {
                unsafe {
                    let (r, m) = <#inner_type as #prelude::UnsizedInit<#prelude::Zeroed>>::init(super_ref, arg)?;
                    Ok((r.wrap_r(|_, r| #ref_ident(r)), #meta_ident(m)))
                }
            }
        }

        #init_with_struct

        unsafe impl #combined_impl_generics #prelude::UnsizedType for #struct_type #combined_where {
            type RefData = #ref_type;
            type RefMeta = #meta_type;
            type Owned = #owned_type;
            type IsUnsized = <#inner_type as #prelude::UnsizedType>::IsUnsized;

            unsafe fn from_bytes<#as_bytes_generic>(
                super_ref: #s
            ) -> #result<#prelude::FromBytesReturn<#s, Self::RefData, Self::RefMeta>> {
                unsafe {
                    Ok(
                        <#inner_type as #prelude::UnsizedType>::from_bytes(super_ref)?
                            .map_ref(|_, r| #ref_ident(r))
                            .map_meta(#meta_ident)
                    )
                }
            }

            fn owned<#as_bytes_generic>(r: #prelude::RefWrapper<#s, Self::RefData>) -> #result <Self::Owned> {
                let #combined_names = <#inner_type as #prelude::UnsizedType>::owned(unsafe { r.wrap_r(|_, r| r.0) })?;
                Ok(#owned_ident {
                    #(#owned_field_idents),*
                })
            }
        }
    }
}

fn impl_generics_with(combined_generics: &mut Generics, with: TypeParam) -> ImplGenerics {
    combined_generics.params.push(GenericParam::Type(with));
    combined_generics.split_for_impl().0
}

fn field_idents(fields: &[Field]) -> Vec<Ident> {
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
