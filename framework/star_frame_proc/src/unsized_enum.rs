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
        debug,
        clone,
        copy,
        align1,
        packed_value_checked,
        unsized_type,
        checked_bit_pattern,
        unsized_enum,
        phantom_data,
        non_null,
        size_of,
        enum_ref_wrapper,
        build_pointer,
        derivative,
        partial_eq,
        eq,
        deref,
        ptr,
        framework_serialize,
        result,
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
    let (b_impl_generics, b_ty_generics, _) = b_generics.split_for_impl();
    let mut a_b_generics = input.generics.clone();
    a_b_generics.params.extend([
        GenericParam::Lifetime(LifetimeParam::new(a_lifetime.clone())),
        GenericParam::Lifetime(LifetimeParam::new(b_lifetime.clone())),
    ]);
    let (a_b_impl_generics, a_b_ty_generics, _) = a_b_generics.split_for_impl();
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
        impl #impl_generics #unsized_enum for #ident #ty_generics #where_clause {
            type Discriminant = #discriminant_ident;
            type EnumRefWrapper<'__a> = #ref_wrapper_ident #a_ty_generics;
            type EnumRefMutWrapper<'__a> = #ref_mut_wrapper_ident #a_ty_generics;

            fn discriminant(&self) -> #discriminant_ident {
                self.discriminant.0
            }
        }
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
        impl #a_impl_generics #deref for #ref_wrapper_ident #a_ty_generics #where_clause {
            type Target = #ident #ty_generics;

            fn deref(&self) -> &Self::Target {
                unsafe { &*#ptr::from_raw_parts(self.ptr.as_ptr(), self.meta.byte_len) }
            }
        }
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
    };

    quote! {
        #main_struct
        #discriminant_enum
        #meta_structs
        #ref_structs
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
    fn generate_idents(ident: Ident, args: TokenStream) -> Self {
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
