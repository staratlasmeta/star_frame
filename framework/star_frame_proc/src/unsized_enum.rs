use crate::util::Paths;
use heck::ToSnakeCase;
use proc_macro2::{Span, TokenStream};
use proc_macro_error::abort;
use quote::{format_ident, quote};
use syn::parse::Parse;
use syn::{
    parse_quote, parse_str, Attribute, Expr, GenericParam, Ident, ItemEnum, Lifetime,
    LifetimeParam, LitStr, Type, Variant,
};

pub fn unsized_enum_impl(input: ItemEnum, args: TokenStream) -> TokenStream {
    let paths = Paths::default();
    verify_repr_u8(&input.attrs);
    let idents = Idents::generate_idents(input.ident, args);
    let last_discriminant = &mut None;
    let variants = input
        .variants
        .iter()
        .map(|var| ParsedVariant::parse_variant(var, last_discriminant))
        .collect::<Vec<_>>();
    let a_lifetime = Lifetime::new("'__a", Span::call_site());
    let b_lifetime = Lifetime::new("'__b", Span::call_site());
    let variant_idents = variants.iter().map(|var| &var.ident).collect::<Vec<_>>();
    let variant_tys = variants.iter().map(|var| &var.ty).collect::<Vec<_>>();
    let variant_discriminants = variants
        .iter()
        .map(|var| &var.discriminant)
        .collect::<Vec<_>>();
    let variant_snake_cases = variants
        .iter()
        .map(|var| &var.snake_case)
        .collect::<Vec<_>>();

    let Idents {
        ident,
        discriminant_ident,
        ref_ident,
        ref_mut_ident,
        ref_wrapper_ident,
        ref_mut_wrapper_ident,
        meta_ident,
        meta_inner_ident,
    } = idents;

    let Paths {
        advance,
        align1,
        box_ty,
        build_pointer,
        build_pointer_mut,
        checked,
        checked_bit_pattern,
        clone,
        copy,
        debug,
        default,
        deref,
        deref_mut,
        derivative,
        enum_ref_mut_wrapper,
        enum_ref_wrapper,
        eq,
        framework_from_bytes,
        framework_from_bytes_mut,
        framework_init,
        framework_serialize,
        non_null,
        packed_value_checked,
        panic,
        partial_eq,
        phantom_data,
        pointer_breakup,
        program_error,
        ptr,
        resize_fn,
        result,
        size_of,
        sol_memset,
        unsized_enum,
        unsized_type,
        ..
    } = &paths;

    let mut a_generics = input.generics.clone();
    a_generics
        .params
        .push(GenericParam::Lifetime(LifetimeParam::new(
            a_lifetime.clone(),
        )));
    let (a_impl_generics, a_ty_generics, _) = a_generics.split_for_impl();
    let mut b_generics = input.generics.clone();
    b_generics
        .params
        .push(GenericParam::Lifetime(LifetimeParam::new(
            b_lifetime.clone(),
        )));
    let (_, b_ty_generics, _) = b_generics.split_for_impl();
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let marker_params = &input.generics.params;

    let main_struct = quote! {
        #[derive(#align1, #derivative)]
        #[derivative(Debug(bound = ""))]
        #[allow(dead_code)]
        pub struct #ident #impl_generics #where_clause {
            phantom_data: #phantom_data<(#marker_params)>,
            discriminant: #packed_value_checked<#discriminant_ident>,
            bytes: [u8],
        }
        #[automatically_derived]
        impl #impl_generics #unsized_enum for #ident #ty_generics #where_clause {
            type Discriminant = #discriminant_ident;
            type EnumRefWrapper<'__a> = #ref_wrapper_ident #a_ty_generics;
            type EnumRefMutWrapper<'__a> = #ref_mut_wrapper_ident #a_ty_generics;

            fn discriminant(&self) -> #discriminant_ident {
                self.discriminant.0
            }
        }
        #[automatically_derived]
        impl #impl_generics #unsized_type for #ident #ty_generics #where_clause {
            type RefMeta = #meta_ident #ty_generics;
            type Ref<'__a> = #ref_wrapper_ident #a_ty_generics;
            type RefMut<'__a> = #ref_mut_wrapper_ident #a_ty_generics;
        }
    };

    let discriminant_enum = quote! {
        #[derive(#copy, #clone, #debug, #partial_eq, #eq)]
        #[repr(u8)]
        pub enum #discriminant_ident {
            #(#variant_idents = #variant_discriminants,)*
        }
        #[automatically_derived]
        unsafe impl #checked_bit_pattern for #discriminant_ident {
            type Bits = u8;

            fn is_valid_bit_pattern(bits: &Self::Bits) -> bool {
                false #(|| #variant_discriminants == *bits)*
            }
        }
    };

    let meta_inner_bound = |x: &TokenStream| {
        LitStr::new(
            &(quote! {
                #(<#variant_tys as #unsized_type>::RefMeta: #x,)*
            })
            .to_string(),
            Span::call_site(),
        )
    };
    let meta_inner_debug_bound = meta_inner_bound(debug);
    let meta_inner_clone_bound = meta_inner_bound(clone);
    let meta_inner_copy_bound = meta_inner_bound(copy);

    let meta_bound = |x: &TokenStream| {
        LitStr::new(
            &(quote! {
                #meta_inner_ident #ty_generics: #x
            })
            .to_string(),
            Span::call_site(),
        )
    };
    let meta_debug_bound = meta_bound(debug);
    let meta_clone_bound = meta_bound(clone);
    let meta_copy_bound = meta_bound(copy);

    let meta_structs = quote! {
        #[derive(#derivative)]
        #[derivative(
            Debug(bound = #meta_inner_debug_bound),
            Clone(bound = #meta_inner_clone_bound),
            Copy(bound = #meta_inner_copy_bound),
        )]
        enum #meta_inner_ident #ty_generics #where_clause {
            #(#variant_idents(<#variant_tys as #unsized_type>::RefMeta),)*
        }
        #[derive(#derivative)]
        #[derivative(
            Debug(bound = #meta_debug_bound),
            Clone(bound = #meta_clone_bound),
            Copy(bound = #meta_copy_bound),
        )]
        pub struct #meta_ident #impl_generics #where_clause {
            byte_len: usize,
            inner: #meta_inner_ident #ty_generics,
        }
    };

    let ref_bound = |x: &TokenStream| {
        LitStr::new(
            &(quote! {
                #(<#variant_tys as #unsized_type>::Ref<#a_lifetime>: #x,)*
            })
            .to_string(),
            Span::call_site(),
        )
    };
    let ref_debug_bound = ref_bound(debug);
    let ref_clone_bound = ref_bound(clone);
    let ref_copy_bound = ref_bound(copy);

    let ref_wrapper_bound = |x: &TokenStream| {
        LitStr::new(
            &(quote! {
                #meta_ident #ty_generics: #x,
            })
            .to_string(),
            Span::call_site(),
        )
    };
    let ref_wrapper_debug_bound = ref_wrapper_bound(debug);
    let ref_wrapper_clone_bound = ref_wrapper_bound(clone);
    let ref_wrapper_copy_bound = ref_wrapper_bound(copy);

    let ref_structs = quote! {
        #[derive(#derivative)]
        #[derivative(
            Debug(bound = #ref_debug_bound),
            Clone(bound = #ref_clone_bound),
            Copy(bound = #ref_copy_bound),
        )]
        pub enum #ref_ident #a_impl_generics #where_clause {
            #(#variant_idents(<#variant_tys as #unsized_type>::Ref<#a_lifetime>),)*
        }

        #[derive(#derivative)]
        #[derivative(
            Debug(bound = #ref_wrapper_debug_bound),
            Clone(bound = #ref_wrapper_clone_bound),
            Copy(bound = #ref_wrapper_copy_bound),
        )]
        pub struct #ref_wrapper_ident #a_impl_generics #where_clause {
            phantom_ref: #phantom_data<&'__a ()>,
            ptr: #non_null<()>,
            meta: #meta_ident #ty_generics,
        }
        #[automatically_derived]
        impl #a_impl_generics #ref_wrapper_ident #a_ty_generics #where_clause {
            #[allow(clippy::size_of_in_element_count)]
            unsafe fn data_ptr(&self) -> #non_null<()> {
                unsafe {
                    #non_null::new_unchecked(
                        self.ptr
                            .as_ptr()
                            .cast::<u8>()
                            .add(#size_of::<u8>())
                            .cast::<()>(),
                    )
                }
            }
        }
        #[automatically_derived]
        impl #a_impl_generics #enum_ref_wrapper for #ref_wrapper_ident #a_ty_generics #where_clause {
            type Ref<#b_lifetime> = #ref_ident #b_ty_generics where Self: #b_lifetime;

            fn value<#b_lifetime>(&#b_lifetime self) -> Self::Ref<#b_lifetime> {
                unsafe {
                    let data_ptr = self.data_ptr();

                    match self.meta.inner {
                        #(
                            <#meta_inner_ident #ty_generics>::#variant_idents(meta) => #ref_ident::#b_ty_generics::#variant_idents(<<#variant_tys as #unsized_type>::Ref<
                                #b_lifetime,
                            > as #build_pointer>::build_pointer(
                                data_ptr, meta
                            )),
                        )*
                    }
                }
            }
        }
        #[automatically_derived]
        impl #a_impl_generics #deref for #ref_wrapper_ident #a_ty_generics #where_clause {
            type Target = #ident #ty_generics;

            fn deref(&self) -> &Self::Target {
                unsafe { &*#ptr::from_raw_parts(self.ptr.as_ptr(), self.meta.byte_len) }
            }
        }
        #[automatically_derived]
        impl #a_impl_generics #framework_serialize for #ref_wrapper_ident #a_ty_generics #where_clause {
            fn to_bytes(&self, output: &mut &mut [u8]) -> #result<()> {
                unsafe {
                    match self.meta.inner {
                        #(
                            <#meta_inner_ident #ty_generics>::#variant_idents(meta) => {
                                const DISCRIMINANT: u8 = #variant_discriminants;
                                DISCRIMINANT.to_bytes(output)?;
                                <<#variant_tys as #unsized_type>::Ref<'_> as #framework_serialize>::to_bytes(
                                    &<<#variant_tys as #unsized_type>::Ref<'_> as #build_pointer>::build_pointer(
                                        self.data_ptr(),
                                        meta,
                                    ),
                                    output,
                                )
                            }
                        )*
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(clippy::unit_arg)]
        unsafe impl #a_impl_generics #framework_from_bytes<#a_lifetime> for #ref_wrapper_ident #a_ty_generics #where_clause {
            fn from_bytes(bytes: &mut & #a_lifetime [u8]) -> #result<Self> {
                let bytes_len = bytes.len();
                let discriminant =
                    #checked::try_from_bytes::<#packed_value_checked<#discriminant_ident>>(
                        &bytes[..#size_of::<#packed_value_checked<#discriminant_ident>>()],
                    )
                    .map_err(|_| #program_error::InvalidAccountData)?;
                let ptr = #non_null::from(
                    #advance::try_advance(bytes, #size_of::<#packed_value_checked<#discriminant_ident>>())?,
                )
                .cast();
                match discriminant.0 {
                    #(
                        #discriminant_ident::#variant_idents => {
                            let sub_ptr =
                                <<#variant_tys as #unsized_type>::Ref<'_> as #framework_from_bytes>::from_bytes(bytes)?;
                            Ok(Self {
                                phantom_ref: #phantom_data,
                                ptr,
                                meta: #meta_ident {
                                    inner: #meta_inner_ident::#variant_idents(#pointer_breakup::break_pointer(&sub_ptr).1),
                                    byte_len: bytes_len - bytes.len(),
                                },
                            })
                        }
                    )*
                }
            }
        }
        #[automatically_derived]
        impl #a_impl_generics #pointer_breakup for #ref_wrapper_ident #a_ty_generics #where_clause {
            type Metadata = #meta_ident #ty_generics;

            #[allow(clippy::unit_arg)]
            fn break_pointer(&self) -> (#non_null<()>, Self::Metadata) {
                (self.ptr, self.meta)
            }
        }
        #[automatically_derived]
        impl #a_impl_generics #build_pointer for #ref_wrapper_ident #a_ty_generics #where_clause {
            unsafe fn build_pointer(pointee: #non_null<()>, metadata: Self::Metadata) -> Self {
                Self {
                    ptr: pointee,
                    meta: metadata,
                    phantom_ref: #phantom_data,
                }
            }
        }
    };

    let ref_mut_debug_bound = LitStr::new(
        &(quote! {
            #(<#variant_tys as #unsized_type>::RefMut<#a_lifetime>: #debug,)*
        })
        .to_string(),
        Span::call_site(),
    );
    let ref_mut_wrapper_debug_bound = LitStr::new(
        &(quote! {
            #meta_ident #ty_generics: #debug,
        })
        .to_string(),
        Span::call_site(),
    );
    let set_names = variant_snake_cases
        .iter()
        .map(|snake_case| format_ident!("set_{}", snake_case));

    let ref_mut_structs = quote! {
        #[derive(#derivative)]
        #[derivative(Debug(bound = #ref_mut_debug_bound))]
        pub enum #ref_mut_ident #a_impl_generics #where_clause {
            #(#variant_idents(<#variant_tys as #unsized_type>::RefMut<#a_lifetime>),)*
        }
        #[derive(#derivative)]
        #[derivative(Debug(bound = #ref_mut_wrapper_debug_bound))]
        pub struct #ref_mut_wrapper_ident #a_impl_generics #where_clause {
            ptr: #non_null<()>,
            meta: #meta_ident #ty_generics,
            #[derivative(Debug = "ignore")]
            resize: #box_ty<dyn #resize_fn<#a_lifetime, #meta_ident #ty_generics>>,
        }
        impl #a_impl_generics #ref_mut_wrapper_ident #a_ty_generics #where_clause {
            unsafe fn data_ptr(&self) -> #non_null<()> {
                unsafe {
                    #non_null::new_unchecked(
                        self.ptr
                            .as_ptr()
                            .cast::<u8>()
                            .add(#size_of::<#packed_value_checked<#discriminant_ident>>())
                            .cast::<()>(),
                    )
                }
            }

            #(
                #[allow(clippy::unit_arg)]
                pub fn #set_names<__A>(&mut self, arg: __A) -> #result<()>
                where
                    #variant_tys: #framework_init<__A>,
                {
                    let new_len =
                        #size_of::<u8>() + <#variant_tys as #framework_init<__A>>::INIT_LENGTH;
                    self.meta.byte_len = <#variant_tys as #framework_init<__A>>::INIT_LENGTH;
                    self.ptr = (self.resize)(new_len, self.meta)?;
                    self.discriminant = #packed_value_checked(#discriminant_ident::#variant_idents);
                    #sol_memset(
                        unsafe {
                            &mut *#ptr::slice_from_raw_parts_mut(
                                self.data_ptr().as_ptr().cast(),
                                <#variant_tys as #framework_init<__A>>::INIT_LENGTH,
                            )
                        },
                        0,
                        self.meta.byte_len,
                    );
                    let init = #pointer_breakup::break_pointer(&unsafe {
                        <#variant_tys as #framework_init<__A>>::init(&mut self.bytes, arg, |_, _| {
                            #panic!("Cannot resize during `set`")
                        })?
                    }).1;
                    let new_meta = #meta_inner_ident::#variant_idents(
                        // Safety: This is effectively moving `init`. The compiler is bugging out on the type.
                        unsafe {
                            (&init as *const <<#variant_tys as #unsized_type>::RefMut<'_> as #pointer_breakup>::Metadata)
                                .cast::<<#variant_tys as #unsized_type>::RefMeta>()
                                .read()
                        },
                    );
                    self.meta.inner = new_meta;
                    self.ptr = (self.resize)(new_len, self.meta)?;
                    Ok(())
                }
            )*
        }
        #[automatically_derived]
        impl #a_impl_generics #enum_ref_wrapper for #ref_mut_wrapper_ident #a_ty_generics #where_clause {
            type Ref<#b_lifetime> = #ref_ident #b_ty_generics where Self: #b_lifetime;

            fn value<#b_lifetime>(& #b_lifetime self) -> Self::Ref<#b_lifetime> {
                unsafe {
                    let data_ptr = self.data_ptr();

                    match self.meta.inner {
                        #(
                            #meta_inner_ident::#variant_idents(meta) => #ref_ident::#b_ty_generics::#variant_idents(
                                <<#variant_tys as #unsized_type>::Ref<#b_lifetime> as #build_pointer>::build_pointer(
                                    data_ptr, meta,
                                ),
                            ),
                        )*
                    }
                }
            }
        }
        #[automatically_derived]
        impl #a_impl_generics #enum_ref_mut_wrapper for #ref_mut_wrapper_ident #a_ty_generics #where_clause {
            type RefMut<#b_lifetime> = #ref_mut_ident #b_ty_generics where Self: #b_lifetime;

            fn value_mut<#b_lifetime>(&#b_lifetime mut self) -> Self::RefMut<#b_lifetime> {
                unsafe {
                    let data_ptr = self.data_ptr();
                    let Self { ptr, meta, resize } = self;
                    match meta.inner {
                        #(
                            #meta_inner_ident::#variant_idents(inner_meta) => #ref_mut_ident::#variant_idents(
                                <<#variant_tys as #unsized_type>::RefMut<#b_lifetime> as #build_pointer_mut>::
                                    build_pointer_mut(
                                        data_ptr, inner_meta, move |new_len, new_meta| {
                                            meta.inner = #meta_inner_ident::#variant_idents(new_meta);
                                            meta.byte_len = new_len + #size_of::<u8>();
                                            *ptr = resize(meta.byte_len, *meta)?;
                                            Ok(#non_null::new(ptr.as_ptr().cast::<u8>().add(#size_of::<u8>()).cast::<()>()).unwrap())
                                        }
                                    ),
                            ),
                        )*
                    }
                }
            }
        }
        #[automatically_derived]
        impl #a_impl_generics #deref for #ref_mut_wrapper_ident #a_ty_generics #where_clause {
            type Target = #ident #ty_generics;

            fn deref(&self) -> &Self::Target {
                unsafe { &*#ptr::from_raw_parts(self.ptr.as_ptr(), self.meta.byte_len) }
            }
        }
        #[automatically_derived]
        impl #a_impl_generics #deref_mut for #ref_mut_wrapper_ident #a_ty_generics #where_clause {
            fn deref_mut(&mut self) -> &mut Self::Target {
                unsafe { &mut *#ptr::from_raw_parts_mut(self.ptr.as_ptr(), self.meta.byte_len) }
            }
        }
        impl #a_impl_generics #framework_serialize for #ref_mut_wrapper_ident #a_ty_generics #where_clause {
            fn to_bytes(&self, output: &mut &mut [u8]) -> #result<()> {
                unsafe {
                    let data_ptr = self.data_ptr();
                    match self.meta.inner {
                        #(
                            #meta_inner_ident::#variant_idents(a) => {
                                const DISCRIMINANT: u8 = #variant_discriminants;
                                DISCRIMINANT.to_bytes(output)?;
                                <<#variant_tys as #unsized_type>::Ref<'_> as #framework_serialize>::to_bytes(
                                    &<<#variant_tys as #unsized_type>::Ref<'_> as #build_pointer>::build_pointer(
                                        data_ptr, a,
                                    ),
                                    output,
                                )
                            }
                        )*
                    }
                }
            }
        }
        impl #a_impl_generics #pointer_breakup for #ref_mut_wrapper_ident #a_ty_generics #where_clause {
            type Metadata = #meta_ident #ty_generics;

            #[allow(clippy::unit_arg)]
            fn break_pointer(&self) -> (#non_null<()>, Self::Metadata) {
                (self.ptr, self.meta)
            }
        }
        #[allow(clippy::unit_arg)]
        unsafe impl #a_impl_generics #framework_from_bytes_mut<#a_lifetime> for #ref_mut_wrapper_ident #a_ty_generics #where_clause {
            fn from_bytes_mut(
                bytes: &mut &#a_lifetime mut [u8],
                resize: impl #resize_fn<#a_lifetime, Self::Metadata>,
            ) -> #result<Self> {
                let bytes_len = bytes.len();
                let discriminant =
                    #checked::try_from_bytes::<#packed_value_checked<#discriminant_ident>>(
                        &bytes[..#size_of::<u8>()],
                    )
                    .map_err(|_| #program_error::InvalidAccountData)?
                    .0;
                let ptr = #non_null::from(
                    #advance::try_advance(bytes, #size_of::<u8>())?,
                )
                .cast();
                match discriminant {
                    #(
                        #discriminant_ident::#variant_idents => {
                            let sub_ptr =
                                <<#variant_tys as #unsized_type>::RefMut<'_> as #framework_from_bytes_mut>::from_bytes_mut(
                                    bytes,
                                    |_, _| panic!("Cannot resize during `from_bytes`"),
                                )?;
                            let broken = #pointer_breakup::break_pointer(&sub_ptr);
                            Ok(Self {
                                ptr,
                                meta: #meta_ident {
                                    inner: #meta_inner_ident::#variant_idents(broken.1),
                                    byte_len: bytes_len - bytes.len(),
                                },
                                resize: Box::new(resize),
                            })
                        }
                    )*
                }
            }
        }
        impl #a_impl_generics #build_pointer_mut<#a_lifetime> for #ref_mut_wrapper_ident #a_ty_generics #where_clause {
            unsafe fn build_pointer_mut(
                pointee: #non_null<()>,
                metadata: Self::Metadata,
                resize: impl #resize_fn<#a_lifetime, Self::Metadata>,
            ) -> Self {
                Self {
                    ptr: pointee,
                    meta: metadata,
                    resize: Box::new(resize),
                }
            }
        }
    };

    let snake_case_ident = format_ident!("{}", ident.to_string().to_snake_case());
    let init_generics = variants
        .iter()
        .map(|variant| {
            let mut generics = input.generics.clone();
            let ParsedVariant { ty, .. } = variant;
            generics.params.push(parse_quote! { __A });
            generics.make_where_clause().predicates.push(parse_quote! {
                #ty: #framework_init<__A>
            });
            generics
        })
        .collect::<Vec<_>>();
    let init_impl_generics = init_generics
        .iter()
        .map(|gen| {
            let (impl_gen, _, _) = gen.split_for_impl();
            impl_gen
        })
        .collect::<Vec<_>>();
    let init_where_clauses = init_generics
        .iter()
        .map(|gen| {
            let (_, _, where_clause) = gen.split_for_impl();
            where_clause
        })
        .collect::<Vec<_>>();
    let a_ty: Type = parse_quote! { __A };

    let init_impls = quote! {
        pub mod #snake_case_ident {
            #(
                #[derive(#copy, #clone, #debug, #default)]
                pub struct #variant_idents;
            )*
        }

        #(
            #[allow(clippy::unit_arg)]
            #[automatically_derived]
            unsafe impl #init_impl_generics #framework_init<(#snake_case_ident::#variant_idents, #a_ty)> for #ident #ty_generics #init_where_clauses {
                const INIT_LENGTH: usize = #size_of::<u8>() + <#variant_tys as #framework_init<#a_ty>>::INIT_LENGTH;

                unsafe fn init<'a>(
                    bytes: &'a mut [u8],
                    (_, arg): (#snake_case_ident::#variant_idents, #a_ty),
                    resize: impl #resize_fn<'a, Self::RefMeta>,
                ) -> #result<Self::RefMut<'a>> {
                    debug_assert_eq!(
                        bytes.len(),
                        <Self as #framework_init<(#snake_case_ident::#variant_idents, #a_ty)>>::INIT_LENGTH
                    );
                    debug_assert!(bytes.iter().all(|b| *b == 0));
                    let ptr = #non_null::from(&*bytes).cast();
                    bytes[0] = #discriminant_ident::#variant_idents as u8;
                    let sub_ptr = unsafe {
                        <#variant_tys as #framework_init<#a_ty>>::init(&mut bytes[#size_of::<u8>()..], arg, |_, _| {
                            #panic!("Cannot resize during `init`")
                        })?
                    };

                    // // Just try uncommenting this if you don't like manual transmute (cast and read)
                    // // Caused by compiler bug.
                    // let broken: (#non_null<()>, ()) = { <&mut T as PointerBreakup>::break_pointer(&sub_ptr) };
                    let broken = { <<#variant_tys as #unsized_type>::RefMut<'_> as #pointer_breakup>::break_pointer(&sub_ptr) };
                    Ok(#ref_mut_wrapper_ident {
                        ptr,
                        meta: #meta_ident {
                            // // Just try uncommenting this if you don't like manual transmute (cast and read)
                            // // Caused by compiler bug.
                            // inner: TestEnumMetaInner::A(broken.1),
                            inner: #meta_inner_ident::#variant_idents(unsafe {
                                (&broken.1
                                    as *const <<#variant_tys as #unsized_type>::RefMut<'_> as #pointer_breakup>::Metadata)
                                    .cast::<<#variant_tys as #unsized_type>::RefMeta>()
                                    .read()
                            }),
                            byte_len: <Self as #framework_init<(#snake_case_ident::#variant_idents, #a_ty)>>::INIT_LENGTH,
                        },
                        resize: Box::new(resize),
                    })
                }
            }
        )*
    };

    quote! {
        #main_struct
        #discriminant_enum
        #meta_structs
        #ref_structs
        #ref_mut_structs
        #init_impls
    }
}

fn verify_repr_u8(attrs: &[Attribute]) {
    let repr = attrs.iter().find(|attr| attr.path().is_ident("repr"));
    if let Some(repr) = repr {
        let repr_ty = repr
            .parse_args_with(Type::parse)
            .unwrap_or_else(|e| abort!(repr, "Could not parse repr type: {}", e));
        if repr_ty != parse_str("u8").unwrap() {
            abort!(repr_ty, "Only u8 is supported as repr type");
        }
    }
}

struct Idents {
    ident: Ident,
    discriminant_ident: Ident,
    ref_ident: Ident,
    ref_mut_ident: Ident,
    ref_wrapper_ident: Ident,
    ref_mut_wrapper_ident: Ident,
    meta_ident: Ident,
    meta_inner_ident: Ident,
}
impl Idents {
    fn generate_idents(ident: Ident, _args: TokenStream) -> Self {
        // TODO: Use args for ident replacement
        Self {
            discriminant_ident: format_ident!("{}Discriminant", ident),
            ref_ident: format_ident!("{}Ref", ident),
            ref_mut_ident: format_ident!("{}RefMut", ident),
            ref_wrapper_ident: format_ident!("{}RefWrapper", ident),
            ref_mut_wrapper_ident: format_ident!("{}RefMutWrapper", ident),
            meta_ident: format_ident!("{}Meta", ident),
            meta_inner_ident: format_ident!("{}MetaInner", ident),
            ident,
        }
    }
}

struct ParsedVariant {
    ident: Ident,
    snake_case: String,
    discriminant: Expr,
    ty: Type,
}
impl ParsedVariant {
    fn parse_variant(variant: &Variant, last_discriminant: &mut Option<Expr>) -> Self {
        let ty_attr = variant
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("variant_type"))
            .unwrap_or_else(|| abort!(variant, "Variant must have a `#[variant_type]` attribute"));
        let ty = ty_attr
            .parse_args_with(Type::parse)
            .unwrap_or_else(|e| abort!(ty_attr, "Could not parse variant type: {}", e));

        Self {
            ident: variant.ident.clone(),
            snake_case: variant.ident.to_string().to_snake_case(),
            discriminant: match &variant.discriminant {
                None => {
                    let discriminant: Expr = last_discriminant
                        .take()
                        .map(|d| parse_quote! { (#d) + 1 })
                        .unwrap_or_else(|| parse_quote! { 0 });
                    *last_discriminant = Some(discriminant.clone());
                    discriminant
                }
                Some((_, expr)) => {
                    last_discriminant.replace(expr.clone());
                    expr.clone()
                }
            },
            ty,
        }
    }
}
