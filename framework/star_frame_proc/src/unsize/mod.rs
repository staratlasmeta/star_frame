use std::fmt::{Display, Formatter};
use std::str::FromStr;

use easy_proc::ArgumentList;
use heck::ToUpperCamelCase;
use itertools::Itertools;
use proc_macro2::Ident;
use proc_macro2::TokenStream;
use proc_macro_error::{abort, abort_call_site};
use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Bracket;
use syn::{
    bracketed, parse2, parse_quote, Attribute, Field, GenericParam, Generics, ImplGenerics, Item,
    ItemStruct, Meta, Token, TypeParam, WhereClause, WherePredicate,
};

use crate::util::{
    add_derivative_attributes, generate_fields_are_trait, get_field_types,
    make_derivative_attribute, strip_inner_attributes, type_generic_idents, BetterGenerics,
    CombineGenerics, Paths,
};

#[derive(Debug, Clone, Default)]
pub struct UnsizedAttributeMetas {
    _bracket: Bracket,
    attributes: Punctuated<Meta, Token![,]>,
}

impl Parse for UnsizedAttributeMetas {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attributes;
        Ok(Self {
            _bracket: bracketed!(attributes in input),
            attributes: attributes.parse_terminated(Meta::parse, Token![,])?,
        })
    }
}

// todo: figure out what args this may need
// todo: derives for each new struct. allow disabling unnecessary default derives
#[derive(ArgumentList, Debug, Clone)]
pub struct UnsizedTypeArgs {
    #[argument(default)]
    pub owned_attributes: UnsizedAttributeMetas,
}

pub fn unsized_type_impl(item: Item, args: TokenStream) -> TokenStream {
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

#[derive(Debug, Clone)]
pub struct UnsizedTypeContext {
    pub item_struct: ItemStruct,
    pub sized_fields: Vec<Field>,
    pub unsized_fields: Vec<Field>,
    pub args: UnsizedTypeArgs,
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
        let unsized_args = UnsizedTypeArgs::parse_arguments(&args_attr);

        const LIFETIME_ERROR: &str = "Lifetimes are not allowed in unsized types";
        if !item_struct.generics.lifetimes().collect_vec().is_empty() {
            abort!(item_struct.generics, LIFETIME_ERROR)
        }

        if !item_struct.generics.const_params().collect_vec().is_empty() {
            abort!(
                item_struct.generics,
                "Const generics are not allowed in unsized types (yet)"
            )
        }

        // if !unsized_args
        //     .unsized_generics
        //     .lifetimes()
        //     .collect_vec()
        //     .is_empty()
        // {
        //     abort!(args, LIFETIME_ERROR)
        // }

        Self {
            item_struct,
            sized_fields: sized_fields.to_vec(),
            unsized_fields: unsized_fields.to_vec(),
            args: unsized_args,
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

fn derive_bytemucks(sized_struct: &ItemStruct) -> TokenStream {
    let Paths {
        bytemuck,
        derivative,
        ..
    } = Default::default();
    let (impl_generics, type_generics, where_clause) = sized_struct.generics.split_for_impl();
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

    let validate_fields_are_trait = generate_fields_are_trait(
        sized_struct,
        parse_quote!(#bytemuck::NoUninit + #bytemuck::Zeroable + #bytemuck::CheckedBitPattern),
    );

    let derivative_attribute = make_derivative_attribute(
        parse_quote!(Debug, Copy, Clone),
        &get_field_types(&bit_fields).collect_vec(),
    );

    let bit_fields = bit_fields.iter();
    let bytemuck_print = bytemuck.to_string().replace(" :: ", "::");
    let zeroable_bit_safety = format!("# Safety\nThis is safe because all fields are [`{bytemuck_print}::CheckedBitPattern::Bits`], which requires [`{bytemuck_print}::AnyBitPattern`], which requires [`{bytemuck_print}::Zeroable`]");
    let any_bit_pattern_safety = format!("# Safety\nThis is safe because all fields are [`{bytemuck_print}::CheckedBitPattern::Bits`], which requires [`{bytemuck_print}::AnyBitPattern`]");
    let no_uninit_safety = format!("# Safety\nThis is safe because the struct is `#[repr(C, packed)` (no padding bytes) and all fields are [`{bytemuck_print}::NoUninit`]");
    let zeroable_safety =
        format!("# Safety\nThis is safe because all fields are [`{bytemuck_print}::Zeroable`]");
    let checked_safety = format!(
        "# Safety\nThis is safe because all fields in [`Self::Bits`] are [`{bytemuck_print}::CheckedBitPattern::Bits`] and share the same repr. The checks are correctly (hopefully) and automatically generated by the macro."
    );

    quote! {

        #validate_fields_are_trait

        #[doc = #zeroable_safety]
        unsafe impl #impl_generics #bytemuck::Zeroable for #struct_ident #type_generics #where_clause {}

        #[doc = #no_uninit_safety]
        unsafe impl #impl_generics #bytemuck::NoUninit for #struct_ident #type_generics #where_clause {}

        #[repr(C, packed)]
        #[derive(#derivative)]
        #derivative_attribute
        pub struct #bit_ident #impl_generics #where_clause {
            #(#bit_fields),*
        }

        #[doc = #zeroable_bit_safety]
        unsafe impl #impl_generics #bytemuck::Zeroable for #bit_ident #type_generics #where_clause {}

        #[doc = #any_bit_pattern_safety]
        unsafe impl #impl_generics #bytemuck::AnyBitPattern for #bit_ident #type_generics #where_clause {}

        #[doc = #checked_safety]
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

fn unsized_type_struct_impl(item_struct: ItemStruct, _args: TokenStream) -> TokenStream {
    let Paths {
        macro_prelude: prelude,
        deref,
        deref_mut,
        result,
        checked,
        size_of,
        phantom_data,
        bytemuck,
        derivative,
        ..
    } = Default::default();
    let context = UnsizedTypeContext::parse(item_struct, _args);
    let sized_ident = context.sized_ident();

    let UnsizedTypeContext {
        mut sized_fields,
        unsized_fields,
        item_struct,
        args: unsized_args,
    } = context;

    let combined_generics = item_struct.generics.clone();

    let generic_idents: Vec<_> = type_generic_idents(&combined_generics);
    if !generic_idents.is_empty() {
        sized_fields.push(
            parse_quote!(pub __phantom_generics: #phantom_data<fn() -> (#(#generic_idents),*)>),
        );
    }

    // todo: figure out these bounds potentially
    // WithUnsizedGenericsInner<A, D>: UnsizedType<
    //         RefMeta = CombinedUnsizedRefMeta<WithUnsizedGenericsSized<A, D>, D>,
    //         RefData = CombinedRef<WithUnsizedGenericsSized<A, D>, D>,
    //         Owned = (
    //             <WithUnsizedGenericsSized<A, D> as UnsizedType>::Owned,
    //             <D as UnsizedType>::Owned,
    //         ),
    //     >,
    //     <WithUnsizedGenericsInner<A, D> as UnsizedType>::IsUnsized:
    //         LengthAccess<WithUnsizedGenerics<A, D>>,
    //
    // if item_struct.fields.len() > 1 {
    //     combined_generics.make_where_clause().predicates.extend([
    //         parse_quote!(#inner_type: #prelude::UnsizedType<
    //             RefMeta = #meta_type,
    //         >
    //         ),
    //     ]);
    // }

    let (combined_impl_generics, combined_type_generics, combined_where) =
        combined_generics.split_for_impl();

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
            #sized_ident #combined_type_generics
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
    let combine_resize = if item_struct.fields.len() > 1 {
        quote! {
            wrapper.r_mut().0 = #prelude::CombinedRef::new(new_meta);
        }
    } else {
        Default::default()
    };

    let extension_ident = format_ident!("{}Ext", struct_ident);
    let unsized_ext_idents = unsized_field_idents
        .iter()
        .map(|i| {
            let ident_string = i.to_string().to_upper_camel_case();
            format_ident!("{struct_ident}{ident_string}")
        })
        .collect::<Vec<_>>();

    let r = quote!(__R);
    let extension_type_generics = type_generic_idents(&combined_generics);
    let combined_extension_type_generics = quote!(#(#extension_type_generics),*);
    let all_extension_type_generics = quote!(#r, #combined_extension_type_generics);

    let root_ident = format_ident!("{struct_ident}Root");
    let root_type = quote!(#root_ident<#all_extension_type_generics>);

    let root_inner_type =
        quote!(#prelude::RefWrapper<#r, <#inner_type as #prelude::UnsizedType>::RefData>);

    let root_type_def = sized_type
        .map(|sized_type| {
            let unsized_combined = combine_streams(&unsized_field_types, combine_unsized);
            quote!(#prelude::RefWrapperU<#root_inner_type, #sized_type, #unsized_combined>)
        })
        .unwrap_or(root_inner_type);

    let (ext_type_defs, path_methods): (Vec<_>, Vec<_>) = unsized_ext_idents
        .iter()
        .enumerate()
        .map(|(index, _ident)| {
            let (ext_type_def, paths) = make_ext_type(&root_type, &unsized_field_types, index);
            let path_methods = CombinePath::make_method_chain(&paths);
            (ext_type_def, path_methods)
        })
        .unzip();

    let ext_generics: BetterGenerics = parse2(
        quote! { [<#r> where #r: #prelude::RefWrapperTypes<Ref = #ref_type> + #prelude::AsBytes] },
    )
    .expect("Shouldn't fail to parse better generics for ext type");

    let ext_generics = combined_generics.combine(ext_generics.into_inner());

    let (ext_impl_generics, _, ext_where) = ext_generics.split_for_impl();

    let root_method = sized_type.map(|_| quote!(.u()?));

    let extension_trait = quote! {
        type #root_type = #root_type_def;

        #(
            pub type #unsized_ext_idents<#all_extension_type_generics> = #ext_type_defs;
        )*

        pub trait #extension_ident #combined_impl_generics: Sized + #prelude::RefWrapperTypes
        #combined_where
        {
            #(
                fn #unsized_field_idents(self) -> #result<#unsized_ext_idents<Self, #combined_extension_type_generics>>;
            )*
        }

        impl #ext_impl_generics #extension_ident #combined_type_generics for #r #ext_where {
            #(
                fn #unsized_field_idents(self) -> #result<#unsized_ext_idents<Self, #combined_extension_type_generics>> {
                    let r = self.r().0;
                    let res = unsafe { #prelude::RefWrapper::new(self, r) } #root_method #path_methods;
                    Ok(res)
                }
            )*
        }
    };

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
            format_ident!("{struct_ident_string}{ident_string}Init")
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
    let (unsized_init_impl_generics, unsized_init_struct_type_generics, init_where_clause) =
        combined_unsized_init_generics.split_for_impl();

    let init_struct_type = quote!(#init_struct_ident #unsized_init_struct_type_generics);

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
        pub struct #init_struct_ident #unsized_init_impl_generics #init_where_clause {
            #(#sized_fields,)*
            #(pub #unsized_field_idents: #init_generic_idents,)*
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

    let sized_stuff = sized_ident.map(|sized_ident| {
        let sized_bytemuck_derives = combined_generics.params.is_empty().then_some(
            quote!(#bytemuck::CheckedBitPattern, #bytemuck::NoUninit, #bytemuck::Zeroable),
        );

        let mut sized_struct: ItemStruct = parse_quote! {
            #[derive(Align1, #derivative, #sized_bytemuck_derives)]
            #[repr(C, packed)]
            pub struct #sized_ident #combined_impl_generics #combined_where {
                #(#sized_fields),*
            }
        };

        add_derivative_attributes(
            &mut sized_struct,
            parse_quote!(Copy, Clone, Debug, PartialEq, Eq),
        );

        // We can only derive CheckedBitPattern and NoUninit if there are no generics. This restriction may be relaxed in the future in bytemuck
        let sized_bytemuck_impls = combined_generics
            .params
            .first()
            .map(|_| derive_bytemucks(&sized_struct));

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

    let mut main_struct: ItemStruct = parse_quote! {
        #[derive(Align1, #derivative)]
        #[repr(transparent)]
        pub struct #struct_ident #combined_impl_generics (#inner_type) #combined_where;
    };
    add_derivative_attributes(&mut main_struct, parse_quote!(Debug));

    let mut meta_struct: ItemStruct = parse_quote! {
        #[derive(#derivative)]
        #[repr(transparent)]
        pub struct #meta_ident #combined_impl_generics (<#inner_type as #prelude::UnsizedType>::RefMeta) #combined_where;
    };
    add_derivative_attributes(&mut meta_struct, parse_quote!(Debug, Copy, Clone));

    let mut ref_struct: ItemStruct = parse_quote! {
        #[derive(Copy, Clone, #derivative)]
        #[repr(transparent)]
        pub struct #ref_ident #combined_impl_generics (<#inner_type as #prelude::UnsizedType>::RefData) #combined_where;
    };
    add_derivative_attributes(&mut ref_struct, parse_quote!(Debug));

    let additional_attributes = unsized_args.owned_attributes.attributes.iter();
    let mut owned_struct = parse_quote! {
        #[derive(#derivative)]
        #(
            #[#additional_attributes]
        )*
        pub struct #owned_ident #combined_impl_generics  #combined_where {
            #(#owned_fields),*
        }
    };
    add_derivative_attributes(&mut owned_struct, parse_quote!(Debug));

    quote! {
        #[allow(type_alias_bounds)]
        pub type #inner_ident #combined_impl_generics = #combined_inner;

        #main_struct

        #meta_struct

        #ref_struct

        #sized_stuff

        #owned_struct

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
                unsafe {
                    #combine_resize
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
            type RefMeta = #meta_type;
            type RefData = #ref_type;
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

        #extension_trait
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

fn with_parenthesis(first: &TokenStream, second: &TokenStream) -> TokenStream {
    quote!((#first, #second))
}

fn combine_unsized(first: &TokenStream, second: &TokenStream) -> TokenStream {
    let Paths { macro_prelude, .. } = Paths::default();
    quote!(#macro_prelude::CombinedUnsized<#first, #second>)
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
    let Paths { macro_prelude, .. } = Paths::default();
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
    let ty = make_ext_type_inner(root, fields, &paths, &macro_prelude);
    (ty, paths)
}

fn split_items<T>(items: &[T]) -> (&[T], &[T]) {
    items.split_at(items.len().div_ceil(2))
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
