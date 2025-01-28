use std::fmt::{Display, Formatter};
use std::str::FromStr;

use crate::unsize::{account, UnsizedTypeArgs};
use crate::util::{
    add_derivative_attributes, generate_fields_are_trait, get_field_idents, get_field_types,
    make_derivative_attribute, new_generic, phantom_generics_ident, phantom_generics_type,
    reject_attributes, restrict_attributes, strip_inner_attributes, type_generic_idents,
    BetterGenerics, CombineGenerics, Paths,
};
use heck::ToUpperCamelCase;
use itertools::Itertools;
use proc_macro2::Ident;
use proc_macro2::TokenStream;
use proc_macro_error::{abort, abort_call_site, OptionExt};
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse_quote, Field, Generics, ImplGenerics, ItemStruct, ItemType, Type, TypeGenerics,
    Visibility,
};

#[allow(non_snake_case)]
macro_rules! UnsizedStructContext {
    ($expr:expr => $($name:ident $(: $rename:ident)? $(,)?)*) => {
        let UnsizedStructContext {
            $($name $(: $rename)? ,)*
            ..
        } = $expr;
    };
}

macro_rules! some_or_return {
    ($sized:ident) => {
        let Some($sized) = $sized else {
            return None;
        };
    };
}

pub(crate) fn unsized_type_struct_impl(
    item_struct: ItemStruct,
    unsized_args: UnsizedTypeArgs,
) -> TokenStream {
    let context = UnsizedStructContext::parse(item_struct);
    // println!("After context!");
    let main_struct = context.main_struct();
    // println!("After main_struct!");
    let inner_struct = context.inner_type();
    // println!("After inner_struct!");
    let ref_struct = context.ref_struct();
    // println!("After ref_struct!");
    let meta_struct = context.meta_struct();
    // println!("After meta_struct!");
    let owned_struct = context.owned_struct(&unsized_args);
    // println!("After owned_struct!");
    let sized_struct = context.sized_struct();
    // println!("After sized_struct!");
    let sized_bytemuck_derives = context.sized_bytemuck_derives();
    // println!("After sized_bytemuck_derives!");
    let sized_ref_deref = context.sized_ref_deref();
    // println!("After sized_ref_deref!");
    let ref_bytes_impl = context.ref_bytes_impl();
    // println!("After ref_bytes_impl!");
    let resize_impl = context.ref_resize_impl();
    // println!("After ref_resize_impl!");
    let unsized_type_impl = context.unsized_type_impl();
    // println!("After unsized_type_impl!");
    let default_init_impl = context.unsized_init_default_impl();
    // println!("After unsized_init_impl!");
    let init_struct_impl = context.unsized_init_struct_impl();
    // println!("After init_struct_impl!");
    let extension_impl = context.extension_impl();
    // println!("After extension_impl!");

    let account_impl = account::account_impl(&context.account_item_struct.into(), &unsized_args);

    quote! {
        #main_struct
        #inner_struct
        #ref_struct
        #meta_struct
        #owned_struct
        #sized_struct
        #sized_bytemuck_derives
        #sized_ref_deref
        #ref_bytes_impl
        #resize_impl
        #unsized_type_impl
        #default_init_impl
        #init_struct_impl
        #extension_impl
        #account_impl
    }
}

#[derive(Clone)]
pub struct UnsizedStructContext {
    item_struct: ItemStruct,
    vis: Visibility,
    struct_ident: Ident,
    struct_type: Type,
    inner_ident: Ident,
    inner_type: Type,
    meta_ident: Ident,
    meta_type: Type,
    ref_ident: Ident,
    ref_type: Type,
    owned_ident: Ident,
    owned_type: Type,
    sized_ident: Option<Ident>,
    sized_type: Option<Type>,
    account_item_struct: ItemStruct,
    sized_fields: Vec<Field>,
    unsized_fields: Vec<Field>,
    owned_fields: Vec<Field>,
    sized_field_idents: Vec<Ident>,
    sized_field_types: Vec<Type>,
    unsized_field_idents: Vec<Ident>,
    unsized_field_types: Vec<Type>,
}

impl UnsizedStructContext {
    fn parse(mut item_struct: ItemStruct) -> Self {
        let unsized_start =
            strip_inner_attributes(&mut item_struct, "unsized_start").collect::<Vec<_>>();
        reject_attributes(
            &item_struct.attrs,
            &Paths::default().type_to_idl_args_ident,
            None,
        );
        let account_item_struct = item_struct.clone();
        strip_inner_attributes(&mut item_struct, &Paths::default().type_to_idl_args_ident)
            .for_each(drop);
        restrict_attributes(&item_struct, &["unsized_start", "type_to_idl", "doc"]);
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
        let mut sized_fields = sized_fields.to_vec();
        let phantom_generics_ident = phantom_generics_ident();
        let phantom_generics_type = phantom_generics_type(&item_struct);
        let vis = item_struct.vis.clone();
        if let Some(ref generic_ty) = phantom_generics_type {
            if !sized_fields.is_empty() {
                sized_fields.push(parse_quote!(#vis #phantom_generics_ident: #generic_ty));
            }
        }
        let unsized_fields = unsized_fields.to_vec();
        let type_generics = &item_struct.generics.split_for_impl().1;
        let struct_ident = item_struct.ident.clone();
        let struct_type = parse_quote!(#struct_ident #type_generics);
        let inner_ident = format_ident!("{struct_ident}Inner");
        let inner_type = parse_quote!(#inner_ident #type_generics);
        let meta_ident = format_ident!("{struct_ident}Meta");
        let meta_type = parse_quote!(#meta_ident #type_generics);
        let ref_ident = format_ident!("{struct_ident}Ref");
        let ref_type = parse_quote!(#ref_ident #type_generics);
        let owned_ident = format_ident!("{struct_ident}Owned");
        let owned_type = parse_quote!(#owned_ident #type_generics);
        let sized_ident = if !sized_fields.is_empty() {
            Some(format_ident!("{}Sized", item_struct.ident))
        } else {
            None
        };
        let sized_type = sized_ident.as_ref().map(|sized_ident| {
            parse_quote! {
                #sized_ident #type_generics
            }
        });

        let sized_field_idents = get_field_idents(&sized_fields).cloned().collect_vec();
        let sized_field_types = get_field_types(&sized_fields).cloned().collect_vec();
        let unsized_field_idents = get_field_idents(&unsized_fields).cloned().collect_vec();
        let unsized_field_types = get_field_types(&unsized_fields).cloned().collect_vec();

        let owned_fields = sized_fields
            .iter()
            .cloned()
            .chain(unsized_fields.iter().cloned().map(|mut field| {
                let field_ty = field.ty.clone();
                Paths!(prelude);
                field.ty = parse_quote!(<#field_ty as #prelude::UnsizedType>::Owned);
                field
            }))
            .collect::<Vec<Field>>();
        Self {
            item_struct,
            vis,
            struct_ident,
            struct_type,
            inner_ident,
            inner_type,
            meta_ident,
            meta_type,
            ref_ident,
            ref_type,
            owned_ident,
            owned_type,
            sized_ident,
            sized_type,
            account_item_struct,
            sized_fields,
            unsized_fields,
            owned_fields,
            sized_field_idents,
            sized_field_types,
            unsized_field_idents,
            unsized_field_types,
        }
    }

    fn generics(&self) -> &Generics {
        &self.item_struct.generics
    }

    fn split_for_impl(&self) -> (ImplGenerics, TypeGenerics, Option<&syn::WhereClause>) {
        self.item_struct.generics.split_for_impl()
    }

    fn main_struct(&self) -> ItemStruct {
        Paths!(prelude, derivative, debug);
        UnsizedStructContext!(self => vis, struct_ident, inner_type);
        let (impl_gen, _, where_clause) = self.split_for_impl();
        let mut main_struct: ItemStruct = parse_quote! {
            #[derive(#prelude::Align1, #derivative)]
            #[repr(transparent)]
            #vis struct #struct_ident #impl_gen (#inner_type) #where_clause;
        };
        add_derivative_attributes(&mut main_struct, parse_quote!(#debug));
        main_struct
    }

    fn ref_struct(&self) -> ItemStruct {
        Paths!(prelude, copy, clone, derivative, debug);
        UnsizedStructContext!(self => vis, ref_ident, inner_type);
        let (impl_gen, _, where_clause) = self.split_for_impl();

        let mut ref_struct: ItemStruct = parse_quote! {
            #[derive(#copy, #clone, #derivative)]
            #[repr(transparent)]
            #vis struct #ref_ident #impl_gen (<#inner_type as #prelude::UnsizedType>::RefData) #where_clause;
        };
        add_derivative_attributes(&mut ref_struct, parse_quote!(#debug));
        ref_struct
    }

    fn meta_struct(&self) -> ItemStruct {
        Paths!(prelude, derivative, debug, copy, clone);
        UnsizedStructContext!(self => vis, meta_ident, inner_type);
        let (impl_gen, _, where_clause) = self.split_for_impl();
        let mut meta_struct: ItemStruct = parse_quote! {
            #[derive(#derivative)]
            #[repr(transparent)]
            #vis struct #meta_ident #impl_gen (<#inner_type as #prelude::UnsizedType>::RefMeta) #where_clause;
        };
        add_derivative_attributes(&mut meta_struct, parse_quote!(#debug, #copy, #clone));
        meta_struct
    }

    fn inner_type(&self) -> ItemType {
        UnsizedStructContext!(self => vis, inner_ident, sized_type, unsized_field_types);
        let (impl_gen, ..) = self.split_for_impl();
        let combined_inner =
            combine_with_sized(sized_type.clone(), unsized_field_types, combine_unsized);
        parse_quote! {
            #[allow(type_alias_bounds)]
            #vis type #inner_ident #impl_gen = #combined_inner;
        }
    }

    fn owned_struct(&self, args: &UnsizedTypeArgs) -> ItemStruct {
        Paths!(derivative, debug);
        UnsizedStructContext!(self => vis, owned_ident, owned_fields);
        let additional_attributes = args.owned_attributes.attributes.iter();

        let (impl_gen, _, where_clause) = self.split_for_impl();

        let mut owned_struct: ItemStruct = parse_quote! {
            #[derive(#derivative)]
            #(
                #[#additional_attributes]
            )*
            #vis struct #owned_ident #impl_gen #where_clause {
                #(#owned_fields,)*
            }
        };
        add_derivative_attributes(&mut owned_struct, parse_quote!(#debug));
        owned_struct
    }

    fn sized_struct(&self) -> Option<ItemStruct> {
        Paths!(prelude, derivative, debug, bytemuck, copy, clone, partial_eq, eq);
        UnsizedStructContext!(self => vis, sized_ident, sized_fields);
        some_or_return!(sized_ident);

        let sized_bytemuck_derives = self.generics().params.is_empty().then_some(
            quote!(#bytemuck::CheckedBitPattern, #bytemuck::NoUninit, #bytemuck::Zeroable),
        );
        let (impl_gen, _, where_clause) = self.split_for_impl();
        let mut sized_struct: ItemStruct = parse_quote! {
            #[derive(#prelude::Align1, #derivative, #sized_bytemuck_derives)]
            #[repr(C, packed)]
            #vis struct #sized_ident #impl_gen #where_clause {
                #(#sized_fields),*
            }
        };
        add_derivative_attributes(
            &mut sized_struct,
            parse_quote!(#copy, #clone, #debug, #partial_eq, #eq),
        );
        Some(sized_struct)
    }

    fn sized_bytemuck_derives(&self) -> Option<TokenStream> {
        Paths!(bytemuck, derivative, debug, copy, clone);
        UnsizedStructContext!(self => vis, sized_fields, sized_ident, sized_field_idents, sized_field_types);
        some_or_return!(sized_ident);
        if self.generics().params.is_empty() {
            return None;
        }
        let (impl_generics, type_generics, where_clause) = self.split_for_impl();

        let bit_ident = format_ident!("{}Bits", sized_ident);
        let bit_field_types = sized_field_types
            .iter()
            .map::<Type, _>(|ty| parse_quote!(<#ty as #bytemuck::CheckedBitPattern>::Bits))
            .collect_vec();

        let derivative_attribute =
            make_derivative_attribute(parse_quote!(#debug, #copy, #clone), &bit_field_types);

        let validate_fields_are_trait = generate_fields_are_trait(
            sized_fields,
            self.generics(),
            parse_quote!(#bytemuck::NoUninit + #bytemuck::Zeroable + #bytemuck::CheckedBitPattern),
        );

        let bytemuck_print = bytemuck.to_string().replace(" :: ", "::");
        let zeroable_bit_safety = format!("# Safety\nThis is safe because all fields are [`{bytemuck_print}::CheckedBitPattern::Bits`], which requires [`{bytemuck_print}::AnyBitPattern`], which requires [`{bytemuck_print}::Zeroable`]");
        let any_bit_pattern_safety = format!("# Safety\nThis is safe because all fields are [`{bytemuck_print}::CheckedBitPattern::Bits`], which requires [`{bytemuck_print}::AnyBitPattern`]");
        let no_uninit_safety = format!("# Safety\nThis is safe because the struct is `#[repr(C, packed)` (no padding bytes) and all fields are [`{bytemuck_print}::NoUninit`]");
        let zeroable_safety =
            format!("# Safety\nThis is safe because all fields are [`{bytemuck_print}::Zeroable`]");
        let checked_safety = format!(
            "# Safety\nThis is safe because all fields in [`Self::Bits`] are [`{bytemuck_print}::CheckedBitPattern::Bits`] and share the same repr. The checks are correctly (hopefully) and automatically generated by the macro."
        );

        Some(quote! {
            #validate_fields_are_trait

            #[doc = #zeroable_safety]
            unsafe impl #impl_generics #bytemuck::Zeroable for #sized_ident #type_generics #where_clause {}

            #[doc = #no_uninit_safety]
            unsafe impl #impl_generics #bytemuck::NoUninit for #sized_ident #type_generics #where_clause {}

            #[repr(C, packed)]
            #[derive(#derivative)]
            #derivative_attribute
            #vis struct #bit_ident #impl_generics #where_clause {
                #(#vis #sized_field_idents: #bit_field_types),*
            }

            #[doc = #zeroable_bit_safety]
            unsafe impl #impl_generics #bytemuck::Zeroable for #bit_ident #type_generics #where_clause {}

            #[doc = #any_bit_pattern_safety]
            unsafe impl #impl_generics #bytemuck::AnyBitPattern for #bit_ident #type_generics #where_clause {}

            #[doc = #checked_safety]
            unsafe impl #impl_generics #bytemuck::CheckedBitPattern for #sized_ident #type_generics #where_clause {
                type Bits = #bit_ident #type_generics;
                #[inline]
                #[allow(clippy::double_comparisons)]
                fn is_valid_bit_pattern(bits: &Self::Bits) -> bool {
                    #(
                        <#sized_field_types as #bytemuck::CheckedBitPattern>
                        ::is_valid_bit_pattern(&{ bits.#sized_field_idents }) &&
                    )* true
                }
            }
        })
    }

    fn sized_ref_deref(&self) -> Option<TokenStream> {
        Paths!(size_of, checked, prelude);
        UnsizedStructContext!(self => sized_type, ref_type,);
        some_or_return!(sized_type);
        let s = new_generic(self.generics());
        let as_bytes_generics = self
            .generics()
            .combine::<BetterGenerics>(&parse_quote!([<#s> where #s: #prelude::AsBytes]));
        let as_bytes_mut_generics = self
            .generics()
            .combine::<BetterGenerics>(&parse_quote!([<#s> where #s: #prelude::AsMutBytes]));

        let (as_bytes_impl, _, as_bytes_where) = as_bytes_generics.split_for_impl();
        let (as_mut_bytes_impl, _, as_mut_bytes_where) = as_bytes_mut_generics.split_for_impl();

        Some(quote! {
            impl #as_bytes_impl #prelude::RefDeref<#s> for #ref_type #as_bytes_where
            {
                type Target = #sized_type;
                fn deref(wrapper: &#prelude::RefWrapper<#s, Self>) -> &Self::Target {
                    use #prelude::RefWrapperTypes;
                    let bytes = wrapper.sup().as_bytes().expect("Invalid bytes");
                    let bytes = &bytes[..#size_of::<#sized_type>()];
                    #checked::from_bytes::<#sized_type>(bytes)
                }
            }

            impl #as_mut_bytes_impl #prelude::RefDerefMut<#s> for #ref_type #as_mut_bytes_where
            {
                fn deref_mut(wrapper: &mut #prelude::RefWrapper<#s, Self>) -> &mut Self::Target {
                    use #prelude::RefWrapperMutExt;
                    let bytes = unsafe { wrapper.sup_mut() }
                        .as_mut_bytes()
                        .expect("Invalid bytes");
                    let bytes = &mut bytes[..#size_of::<#sized_type>()];
                    #checked::from_bytes_mut::<#sized_type>(bytes)
                }
            }
        })
    }

    fn ref_bytes_impl(&self) -> TokenStream {
        Paths!(prelude, result);
        UnsizedStructContext!(self => ref_type);
        let s = new_generic(self.generics());
        let as_bytes_generics = self
            .generics()
            .combine::<BetterGenerics>(&parse_quote!([<#s> where #s: #prelude::AsBytes]));
        let as_bytes_mut_generics = self
            .generics()
            .combine::<BetterGenerics>(&parse_quote!([<#s> where #s: #prelude::AsMutBytes]));

        let (as_bytes_impl, _, as_bytes_where) = as_bytes_generics.split_for_impl();
        let (as_mut_bytes_impl, _, as_mut_bytes_where) = as_bytes_mut_generics.split_for_impl();

        quote! {
            unsafe impl #as_bytes_impl #prelude::RefBytes<#s> for #ref_type #as_bytes_where
            {
                fn bytes(wrapper: &#prelude::RefWrapper<#s, Self>) -> #result<&[u8]> {
                    use #prelude::{RefWrapperTypes, AsBytes};
                    wrapper.sup().as_bytes()
                }
            }

            unsafe impl #as_mut_bytes_impl #prelude::RefBytesMut<#s> for #ref_type #as_mut_bytes_where
            {
                fn bytes_mut(wrapper: &mut #prelude::RefWrapper<#s, Self>) -> #result<&mut [u8]> {
                    use #prelude::{RefWrapperMutExt, AsMutBytes};
                    unsafe { wrapper.sup_mut().as_mut_bytes() }
                }
            }
        }
    }

    fn ref_resize_impl(&self) -> TokenStream {
        Paths!(prelude, result);
        UnsizedStructContext!(self => inner_type, ref_type, meta_ident, item_struct, meta_type);
        let combine_resize = if item_struct.fields.len() > 1 {
            quote! {
                wrapper.r_mut().0 = #prelude::CombinedRef::new(new_meta);
            }
        } else {
            quote!()
        };
        let s = new_generic(self.generics());
        let ref_resize_generics = self.generics().combine::<BetterGenerics>(
            &parse_quote!([<#s> where #s: #prelude::Resize<#meta_type>]),
        );

        let (ref_resize_impl_gen, _, ref_resize_where) = ref_resize_generics.split_for_impl();

        quote! {
            unsafe impl #ref_resize_impl_gen #prelude::RefResize<#s, <#inner_type as #prelude::UnsizedType>::RefMeta> for #ref_type #ref_resize_where
            {
                unsafe fn resize(
                    wrapper: &mut #prelude::RefWrapper<#s, Self>,
                    new_byte_len: usize,
                    new_meta: <#inner_type as #prelude::UnsizedType>::RefMeta,
                ) -> #result<()> {
                    use #prelude::RefWrapperMutExt;
                    unsafe {
                        #combine_resize
                        wrapper
                            .sup_mut()
                            .resize(new_byte_len, #meta_ident(new_meta))
                    }
                }

                unsafe fn set_meta(
                    wrapper: &mut #prelude::RefWrapper<#s, Self>,
                    new_meta: <#inner_type as #prelude::UnsizedType>::RefMeta,
                ) -> #result<()> {
                    use #prelude::RefWrapperMutExt;
                    unsafe {
                        #combine_resize
                        wrapper.sup_mut().set_meta(#meta_ident(new_meta))
                    }
                }
            }
        }
    }

    fn unsized_type_impl(&self) -> TokenStream {
        Paths!(prelude, result);
        UnsizedStructContext!(self => inner_type, ref_type, meta_ident, unsized_field_idents,
            sized_field_idents, struct_type, meta_type, owned_type, ref_ident, owned_ident, sized_ident,
            owned_fields
        );
        let s = new_generic(self.generics());
        let (impl_gen, _, where_clause) = self.split_for_impl();

        let sized_owned_destructure = sized_ident.as_ref().map(|sized_ident| {
            quote! {
                 #sized_ident {
                    #(#sized_field_idents),*
                }
            }
        });
        let combined_names = combine_with_sized(
            sized_owned_destructure,
            unsized_field_idents,
            with_parenthesis,
        );

        let owned_field_idents = get_field_idents(owned_fields);

        quote! {
            unsafe impl #impl_gen #prelude::UnsizedType for #struct_type #where_clause {
                type RefMeta = #meta_type;
                type RefData = #ref_type;
                type Owned = #owned_type;
                type IsUnsized = <#inner_type as #prelude::UnsizedType>::IsUnsized;

                fn from_bytes<#s: #prelude::AsBytes>(
                    super_ref: #s,
                ) -> #result<#prelude::FromBytesReturn<#s, Self::RefData, Self::RefMeta>> {
                    unsafe {
                        Ok(
                            <#inner_type as #prelude::UnsizedType>::from_bytes(super_ref)?
                                .map_ref(|_, r| #ref_ident(r))
                                .map_meta(#meta_ident)
                        )
                    }
                }

                unsafe fn from_bytes_and_meta<#s: #prelude::AsBytes>(
                    super_ref: #s,
                    meta: Self::RefMeta,
                ) -> #result<#prelude::FromBytesReturn<#s, Self::RefData, Self::RefMeta>> {
                    Ok(
                        unsafe {
                            <#inner_type as #prelude::UnsizedType>::from_bytes_and_meta(super_ref, meta.0)?
                                .map_ref(|_, r| #ref_ident(r))
                                .map_meta(#meta_ident)
                        }
                    )
                }

                fn owned<#s: #prelude::AsBytes>(r: #prelude::RefWrapper<#s, Self::RefData>) -> #result <Self::Owned> {
                    let #combined_names = <#inner_type as #prelude::UnsizedType>::owned(unsafe { r.wrap_r(|_, r| r.0) })?;
                    Ok(#owned_ident {
                        #(#owned_field_idents),*
                    })
                }
            }
        }
    }

    fn unsized_init_default_impl(&self) -> TokenStream {
        Paths!(prelude, result);
        UnsizedStructContext!(self => inner_type, meta_ident, ref_ident, struct_type);
        let s = new_generic(self.generics());
        let init_zeroed_generics = self.generics().combine::<BetterGenerics>(
            &parse_quote!([where #inner_type: #prelude::UnsizedInit<#prelude::DefaultInit>]),
        );
        let (init_zeroed_impl, _, init_zeroed_where) = init_zeroed_generics.split_for_impl();
        quote! {
            impl #init_zeroed_impl #prelude::UnsizedInit<#prelude::DefaultInit> for #struct_type #init_zeroed_where {
                const INIT_BYTES: usize = <#inner_type as #prelude::UnsizedInit<#prelude::DefaultInit>>::INIT_BYTES;

                unsafe fn init<#s: #prelude::AsMutBytes>(
                    super_ref: #s,
                    arg: #prelude::DefaultInit,
                ) -> #result<(#prelude::RefWrapper<#s, Self::RefData>, Self::RefMeta)> {
                    unsafe {
                        let (r, m) = <#inner_type as #prelude::UnsizedInit<#prelude::DefaultInit>>::init(super_ref, arg)?;
                        Ok((r.wrap_r(|_, r| #ref_ident(r)), #meta_ident(m)))
                    }
                }
            }
        }
    }

    fn unsized_init_struct_impl(&self) -> TokenStream {
        Paths!(prelude, result, copy, clone, debug);
        UnsizedStructContext!(self => vis, unsized_fields, inner_type, meta_ident, ref_ident, sized_ident, sized_field_idents,
            struct_type, unsized_field_types, unsized_field_idents, struct_ident, sized_type, sized_fields);
        let init_struct_ident = format_ident!("{struct_ident}Init");
        let s = new_generic(self.generics());
        let init_generic_idents: Vec<_> = unsized_field_idents
            .iter()
            .map(|i| {
                let ident_string = i.to_string().to_upper_camel_case();
                let struct_ident_string = struct_ident.to_string().to_upper_camel_case();
                format_ident!("{struct_ident_string}{ident_string}Init")
            })
            .collect();

        let unsized_init_type_generics =
            combine_with_sized(sized_type.clone(), &init_generic_idents, with_parenthesis);

        let init_generics = self.generics().combine::<BetterGenerics>(&parse_quote!([
            <#(#init_generic_idents),*> where
                #(#unsized_field_types: #prelude::UnsizedInit<#init_generic_idents>,)*
                #inner_type: #prelude::UnsizedInit<#unsized_init_type_generics>
        ]));
        let (unsized_init_impl_generics, unsized_init_struct_type_generics, init_where_clause) =
            init_generics.split_for_impl();

        let sized_field_accesses = sized_ident.as_ref().map(|sized_ident| {
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

        let init_struct_type = quote!(#init_struct_ident #unsized_init_struct_type_generics);
        let extra_generic_field = sized_fields
            .is_empty()
            .then(|| {
                phantom_generics_type(self.generics()).map(|ty| {
                    let ident = phantom_generics_ident();
                    quote!(#vis #ident: #ty,)
                })
            })
            .flatten();

        let unsized_field_vis = unsized_fields.iter().map(|field| &field.vis);

        quote! {
            #[derive(#copy, #clone, #debug)]
            #vis struct #init_struct_ident #unsized_init_impl_generics #init_where_clause {
                #(#sized_fields,)*
                #extra_generic_field
                #(#unsized_field_vis #unsized_field_idents: #init_generic_idents,)*
            }

            impl #unsized_init_impl_generics #prelude::UnsizedInit<#init_struct_type> for #struct_type #init_where_clause
            {
                const INIT_BYTES: usize = <#inner_type as #prelude::UnsizedInit<#unsized_init_type_generics>>::INIT_BYTES;

                unsafe fn init<#s: #prelude::AsMutBytes>(
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
        }
    }

    fn extension_impl(&self) -> TokenStream {
        Paths!(prelude, result);
        UnsizedStructContext!(self =>
            vis,
            struct_ident,
            ref_type,
            unsized_fields,
            inner_type,
            sized_type,
            unsized_field_types
        );

        let pub_extension_ident = format_ident!("{}PubExt", struct_ident);
        let priv_extension_ident = format_ident!("{}Ext", struct_ident);

        let r = quote!(__R);
        let extension_type_generics = type_generic_idents(self.generics());
        let combined_extension_type_generics = quote!(#(#extension_type_generics),*);
        let all_extension_type_generics = quote!(#r, #combined_extension_type_generics);

        let root_ident = format_ident!("{struct_ident}Root");
        let root_type = quote!(#root_ident<#all_extension_type_generics>);

        let root_inner_type =
            quote!(#prelude::RefWrapper<#r, <#inner_type as #prelude::UnsizedType>::RefData>);
        let root_type_def = sized_type
            .as_ref()
            .map(|sized_type| {
                let unsized_combined = combine_streams(unsized_field_types, combine_unsized);
                quote!(#prelude::RefWrapperU<#root_inner_type, #sized_type, #unsized_combined>)
            })
            .unwrap_or(root_inner_type);

        let (pub_unsized_fields, priv_unsized_fields): (Vec<_>, Vec<_>) = unsized_fields
            .iter()
            .enumerate()
            .partition(|(_, field)| match field.vis {
                Visibility::Public(_) => true,
                Visibility::Inherited => false,
                Visibility::Restricted(_) => {
                    abort!(field.vis, "Unsized fields must be `pub` or private")
                }
            });
        let ext_generics: BetterGenerics = parse_quote! { [<#r> where #r: #prelude::RefWrapperTypes<Ref = #ref_type> + #prelude::AsBytes] };

        let ext_generics = self.generics().combine(&ext_generics);

        let (ext_impl_generics, _, ext_where) = ext_generics.split_for_impl();
        let (impl_gen, ty_gen, where_clause) = self.split_for_impl();

        let root_method = sized_type.as_ref().map(|_| quote!(.u()?));

        let make_ext_trait = |vis: &Visibility,
                              fields: Vec<(usize, &Field)>,
                              extension_ident: &Ident| {
            let only_fields = fields.iter().map(|(_, f)| *f).collect::<Vec<_>>();
            let field_idents = get_field_idents(&only_fields).collect_vec();
            let unsized_ext_idents = fields
                .iter()
                .map(|(_, f)| {
                    let ident_string = f
                        .ident
                        .as_ref()
                        .expect_or_abort("Missing field ident. This shouldn't happen")
                        .to_string()
                        .to_upper_camel_case();
                    format_ident!("{struct_ident}{ident_string}")
                })
                .collect::<Vec<_>>();
            let (ext_type_defs, path_methods): (Vec<_>, Vec<_>) = fields
                .iter()
                .map(|(index, _)| {
                    let (ext_type_def, paths) =
                        make_ext_type(&root_type, unsized_field_types, *index);
                    let path_methods = CombinePath::make_method_chain(&paths);
                    (ext_type_def, path_methods)
                })
                .unzip();
            quote! {
                #(
                    #vis type #unsized_ext_idents<#all_extension_type_generics> = #ext_type_defs;
                )*

                #vis trait #extension_ident #impl_gen: Sized + #prelude::RefWrapperTypes
                #where_clause
                {
                    #(
                        fn #field_idents(self) -> #result<#unsized_ext_idents<Self, #combined_extension_type_generics>>;
                    )*
                }

                impl #ext_impl_generics #extension_ident #ty_gen for #r #ext_where {
                    #(
                        fn #field_idents(self) -> #result<#unsized_ext_idents<Self, #combined_extension_type_generics>> {
                            let r = self.r().0;
                            let res = unsafe { #prelude::RefWrapper::new(self, r) } #root_method #path_methods;
                            Ok(res)
                        }
                    )*
                }
            }
        };

        let pub_trait = (!pub_unsized_fields.is_empty())
            .then(|| make_ext_trait(&parse_quote!(pub), pub_unsized_fields, &pub_extension_ident));
        let priv_trait = (!priv_unsized_fields.is_empty()).then(|| {
            make_ext_trait(
                &Visibility::Inherited,
                priv_unsized_fields,
                &priv_extension_ident,
            )
        });

        quote! {
            #vis type #root_type = #root_type_def;
            #pub_trait
            #priv_trait
        }
    }
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

fn with_parenthesis(first: &TokenStream, second: &TokenStream) -> TokenStream {
    quote!((#first, #second))
}

fn combine_unsized(first: &TokenStream, second: &TokenStream) -> TokenStream {
    let Paths { prelude, .. } = Paths::default();
    quote!(#prelude::CombinedUnsized<#first, #second>)
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
enum CombinePath {
    U,
    T,
}

impl CombinePath {
    fn to_wrapper_type(self, macro_prelude: &TokenStream) -> TokenStream {
        match self {
            CombinePath::U => quote!(#macro_prelude::RefWrapperU),
            CombinePath::T => quote!(#macro_prelude::RefWrapperT),
        }
    }

    fn to_method(self) -> String {
        match self {
            CombinePath::U => "u()",
            CombinePath::T => "t()",
        }
        .into()
    }

    fn make_method_chain(paths: &[Self]) -> TokenStream {
        if paths.is_empty() {
            TokenStream::default()
        } else {
            let method_vec = paths
                .iter()
                .copied()
                .map(CombinePath::to_method)
                .collect::<Vec<_>>();
            let method_string = format!(".{}?", method_vec.join("?."));
            TokenStream::from_str(&method_string).expect("Should be a valid function call chain")
        }
    }
}

impl Display for CombinePath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CombinePath::U => write!(f, "U"),
            CombinePath::T => write!(f, "T"),
        }
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
        let (first, second) = split_items(stuff);
        let first_stream = combine_streams(first, combine);
        let second_stream = combine_streams(second, combine);
        combine(&first_stream, &second_stream)
    }
}

fn get_combine_paths(mut length: usize, target: usize) -> Vec<CombinePath> {
    let mut paths = vec![];
    let mut offset = 0;
    while length > 1 {
        let half = length.div_ceil(2);
        if target < half + offset {
            // look at the first half
            paths.push(CombinePath::T);
            length = half;
        } else {
            // look at the second half
            paths.push(CombinePath::U);
            length -= half;
            offset += half;
        }
    }
    paths
}

fn make_ext_type(
    root: &impl ToTokens,
    fields: &[impl ToTokens],
    index: usize,
) -> (TokenStream, Vec<CombinePath>) {
    let paths = get_combine_paths(fields.len(), index);
    let Paths { prelude, .. } = Paths::default();
    fn make_ext_type_inner(
        super_ref: &impl ToTokens,
        fields: &[impl ToTokens],
        paths: &[CombinePath],
        macro_prelude: &TokenStream,
    ) -> TokenStream {
        if paths.is_empty() {
            return super_ref.to_token_stream();
        }
        let (t, u) = split_items(fields);

        let first = paths.first().expect("Paths isn't empty");

        let first_ref = first.to_wrapper_type(macro_prelude);

        let combined_t = combine_streams(t, combine_unsized);
        let combined_u = combine_streams(u, combine_unsized);

        let new_super = quote!(#first_ref<#super_ref, #combined_t, #combined_u>);
        let new_paths = &paths[1..];

        match first {
            CombinePath::T => make_ext_type_inner(&new_super, t, new_paths, macro_prelude),
            CombinePath::U => make_ext_type_inner(&new_super, u, new_paths, macro_prelude),
        }
    }
    let ty = make_ext_type_inner(root, fields, &paths, &prelude);
    (ty, paths)
}

fn split_items<T>(items: &[T]) -> (&[T], &[T]) {
    items.split_at(items.len().div_ceil(2))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

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

    #[test]
    fn test_paths() {
        use CombinePath::*;
        let paths = get_combine_paths(5, 0);
        // [ 0 , 1 , 2 , 3 , 4 ]
        // [ 0 , 1 , 2], [ 3 , 4 ] => T
        // [ 0 , 1 ], [ 2 ] => T
        // [ 0 ], [ 1 ] => T
        assert_eq!(paths, vec![T, T, T]);

        let paths = get_combine_paths(5, 1);
        // [ 0 , 1 , 2 , 3 , 4 ]
        // [ 0 , 1 , 2], [ 3 , 4 ] => T
        // [ 0 , 1 ], [ 2 ] => T
        // [ 0 ], [ 1 ] => U
        assert_eq!(paths, vec![T, T, U]);

        let paths = get_combine_paths(5, 4);
        // [ 0 , 1 , 2 , 3 , 4 ]
        // [ 0 , 1 , 2], [ 3 , 4 ] => U
        // [ 3 ] , [ 4 ] => U
        assert_eq!(paths, vec![U, U]);

        let paths = get_combine_paths(5, 2);
        // [ 0 , 1 , 2 , 3 , 4 ]
        // [ 0 , 1 , 2], [ 3 , 4 ] => T
        // [ 0 , 1 ] , [ 2 ] => U
        assert_eq!(paths, vec![T, U]);

        let paths = get_combine_paths(1, 0);
        assert_eq!(paths, vec![]);
    }
}
