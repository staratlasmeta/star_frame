use crate::util::{verify_repr, Paths};
use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use proc_macro_error::abort_call_site;
use quote::{format_ident, quote};
use syn::{parse_quote, Field, Fields, Ident, ItemStruct, Lifetime, LitStr, Type, WherePredicate};

struct Idents {
    ident: Ident,
    ref_ident: Ident,
    ref_mut_ident: Ident,
    ref_mod_ident: Ident,
    ref_mut_mod_ident: Ident,
}
impl Idents {
    pub fn generate(main_ident: Ident, _args: TokenStream) -> Self {
        // TODO: Use args to generate the idents
        Self {
            ref_ident: format_ident!("{}Ref", main_ident),
            ref_mut_ident: format_ident!("{}RefMut", main_ident),
            ref_mod_ident: format_ident!("__{}_ref_mod", main_ident.to_string().to_snake_case()),
            ref_mut_mod_ident: format_ident!(
                "__{}_ref_mut_mod",
                main_ident.to_string().to_snake_case()
            ),
            ident: main_ident,
        }
    }
}

pub fn unsized_struct_impl(item: ItemStruct, args: TokenStream) -> TokenStream {
    let reprs = verify_repr(&item.attrs, [format_ident!("C")], true, true);
    let use_packed = reprs.iter().any(|repr| *repr == "packed");
    let vis = item.vis;
    let Idents {
        ident,
        ref_ident,
        ref_mut_ident,
        ref_mod_ident,
        ref_mut_mod_ident,
    } = Idents::generate(item.ident, args);
    let new_attrs = item
        .attrs
        .iter()
        .filter(|attr| !attr.path().is_ident("unsized_type"));

    let fields = &item.fields;
    if fields.is_empty() {
        abort_call_site!(
            "Unsized structs must have at least one field. Use `Pod` for zero-sized types."
        );
    }

    let Paths {
        advance,
        box_ty,
        build_pointer,
        build_pointer_mut,
        clone,
        copy,
        debug,
        deref,
        deref_mut,
        derivative,
        framework_from_bytes,
        framework_from_bytes_mut,
        framework_init,
        framework_serialize,
        non_null,
        panic,
        phantom_data,
        pod,
        pointer_breakup,
        ptr,
        resize_fn,
        result,
        size_of,
        unsized_type,
        ..
    } = Paths::default();

    let main_struct_fields = match fields {
        Fields::Named(named) => {
            let last_index = named.named.len() - 1;
            let fields = named.named.iter().enumerate().map(|(index, field)| {
                if index == last_index {
                    let Field {
                        attrs,
                        ident,
                        colon_token,
                        ty,
                        ..
                    } = field;
                    quote! { #(#attrs)* #ident #colon_token #ty }
                } else {
                    quote! { #field }
                }
            });
            quote! { { #(#fields,)* } }
        }
        Fields::Unnamed(_) => {
            abort_call_site!(
                "Unsized structs cannot have unnamed fields. Use `Pod` for zero-sized types."
            );
        }
        Fields::Unit => unreachable!(),
    };

    let final_field_ty = fields.iter().last().map(|field| &field.ty).unwrap();
    let final_field_vis = fields.iter().last().map(|field| &field.vis).unwrap();
    let final_field_ident = fields
        .iter()
        .last()
        .map(|field| field.ident.as_ref().unwrap())
        .unwrap();

    let pod_field_idents = fields
        .iter()
        .take(fields.len() - 1)
        .map(|field| &field.ident)
        .collect::<Vec<_>>();
    let pod_field_tys = fields
        .iter()
        .take(fields.len() - 1)
        .map(|field| &field.ty)
        .collect::<Vec<_>>();
    let mut unsized_generics = item.generics.clone();
    unsized_generics.make_where_clause().predicates.extend(
        pod_field_tys
            .iter()
            .map(|ty| -> WherePredicate {
                parse_quote! {
                    #ty: 'static + #pod
                }
            })
            .chain([parse_quote! {
                #final_field_ty: #unsized_type
            }]),
    );
    let (_, _, unsized_where_clause) = unsized_generics.split_for_impl();
    let (impl_gen, ty_gen, where_clause) = item.generics.split_for_impl();
    let a_lifetime: Lifetime = parse_quote! { '__a };
    let mut a_generics = item.generics.clone();
    a_generics.params.insert(0, parse_quote! { #a_lifetime });
    let (a_impl_gen, a_ty_gen, _) = a_generics.split_for_impl();
    let arg_ty: Type = parse_quote! { __A };
    let mut arg_generics = unsized_generics.clone();
    arg_generics.params.insert(0, parse_quote! { #arg_ty });
    arg_generics
        .make_where_clause()
        .predicates
        .push(parse_quote! {
            #final_field_ty: #framework_init<#arg_ty>
        });
    let (arg_impl_gen, _, arg_where_clause) = arg_generics.split_for_impl();

    let main_struct = quote! {
        #(#new_attrs)*
        #vis struct #ident #impl_gen #where_clause #main_struct_fields
        #[automatically_derived]
        impl #impl_gen #unsized_type for #ident #ty_gen #unsized_where_clause {
            type RefMeta = <#final_field_ty as #unsized_type>::RefMeta;
            type Ref<#a_lifetime> = #ref_ident #a_ty_gen;
            type RefMut<#a_lifetime> = #ref_mut_ident #a_ty_gen;
        }
    };

    let ref_bound = |x: &TokenStream| {
        LitStr::new(
            &(quote! {
                &#a_lifetime #ident #ty_gen: #x,
                <#final_field_ty as #unsized_type>::Ref<#a_lifetime>: #x,
            })
            .to_string(),
            proc_macro2::Span::call_site(),
        )
    };
    let ref_debug_bound = ref_bound(&debug);
    let ref_clone_bound = ref_bound(&clone);
    let ref_copy_bound = ref_bound(&copy);
    let to_bytes_impl = if use_packed {
        quote! {
            #((&{ self.#pod_field_idents }).to_bytes(output)?;)*
        }
    } else {
        quote! {
            #((&self.#pod_field_idents).to_bytes(output)?;)*
        }
    };
    let ref_struct = quote! {
        #vis use #ref_mod_ident::#ref_ident;
        mod #ref_mod_ident {
            use super::*;

            #[derive(#derivative)]
            #[derivative(Debug(bound = #ref_debug_bound), Clone(bound = #ref_clone_bound), Copy(bound = #ref_copy_bound))]
            pub struct #ref_ident #a_impl_gen #unsized_where_clause {
                __ptr: &#a_lifetime #ident #ty_gen,
                #final_field_vis #final_field_ident: <#final_field_ty as #unsized_type>::Ref<#a_lifetime>,
            }
            #[automatically_derived]
            impl #a_impl_gen #deref for #ref_ident #a_ty_gen #unsized_where_clause {
                type Target = #ident #ty_gen;

                fn deref(&self) -> &Self::Target {
                    self.__ptr
                }
            }
            #[automatically_derived]
            impl #a_impl_gen #framework_serialize for #ref_ident #a_ty_gen #unsized_where_clause {
                fn to_bytes(&self, output: &mut &mut [u8]) -> #result<()> {
                    #to_bytes_impl
                    self.#final_field_ident.to_bytes(output)
                }
            }
            #[automatically_derived]
            unsafe impl #a_impl_gen #framework_from_bytes<#a_lifetime> for #ref_ident #a_ty_gen #unsized_where_clause {
                fn from_bytes(bytes: &mut &#a_lifetime [u8]) -> #result<Self> {
                    let header_bytes = #advance::try_advance(bytes, 0 #(+ #size_of::<#pod_field_tys>())*)?;
                    let remaining =
                        <<#final_field_ty as #unsized_type>::Ref<#a_lifetime> as #framework_from_bytes>::from_bytes(bytes)?;
                    let remaining_metadata: <#final_field_ty as #ptr::Pointee>::Metadata = #ptr::metadata(&*remaining);
                    Ok(Self {
                        __ptr: unsafe {
                            &*#ptr::from_raw_parts(header_bytes.as_ptr().cast(), remaining_metadata)
                        },
                        #final_field_ident: remaining,
                    })
                }
            }
            #[automatically_derived]
            impl #a_impl_gen #pointer_breakup for #ref_ident #a_ty_gen #unsized_where_clause {
                type Metadata = <#final_field_ty as #unsized_type>::RefMeta;

                fn break_pointer(&self) -> (#non_null<()>, Self::Metadata) {
                    (#non_null::from(self.__ptr).cast(), self.#final_field_ident.break_pointer().1)
                }
            }
            #[automatically_derived]
            impl #a_impl_gen #build_pointer for #ref_ident #a_ty_gen #unsized_where_clause {
                unsafe fn build_pointer(pointee: #non_null<()>, metadata: Self::Metadata) -> Self {
                    let remaining_ref = unsafe {
                        <<#final_field_ty as #unsized_type>::Ref<'_> as #build_pointer>::build_pointer(
                            pointee, metadata,
                        )
                    };
                    let remaining_metadata = #ptr::metadata(&*remaining_ref);
                    Self {
                        __ptr: unsafe { &*#ptr::from_raw_parts(pointee.cast().as_ptr(), remaining_metadata) },
                        #final_field_ident: remaining_ref,
                    }
                }
            }
        }
    };

    let ref_mut_debug_bound = LitStr::new(
        &(quote! {
            <#final_field_ty as #unsized_type>::RefMeta: #debug,
        })
        .to_string(),
        proc_macro2::Span::call_site(),
    );
    let extra_ty_generics = &item.generics.params;
    let final_field_ident_ref = format_ident!("{}_ref", final_field_ident);
    let final_field_ident_mut = format_ident!("{}_mut", final_field_ident);
    let ref_mut_struct = quote! {
        #vis use #ref_mut_mod_ident::#ref_mut_ident;
        mod #ref_mut_mod_ident {
            use super::*;

            #[derive(#derivative)]
            #[derivative(Debug(bound = #ref_mut_debug_bound))]
            pub struct #ref_mut_ident #a_impl_gen #unsized_where_clause {
                phantom_tys: #phantom_data<(#extra_ty_generics)>,
                ptr: #non_null<()>,
                meta: <#final_field_ty as #unsized_type>::RefMeta,
                #[derivative(Debug = "ignore")]
                resize: #box_ty<dyn #resize_fn<#a_lifetime, <#final_field_ty as #unsized_type>::RefMeta>>,
            }
            impl #a_impl_gen #ref_mut_ident #a_ty_gen #unsized_where_clause {
                const HEADER_SIZE: usize = 0 #(+ #size_of::<#pod_field_tys>())*;

                #final_field_vis fn #final_field_ident_ref(&self) -> <#final_field_ty as #unsized_type>::Ref<'_> {
                    unsafe {
                        <<#final_field_ty as #unsized_type>::Ref<'_> as #build_pointer>::build_pointer(
                            #non_null::new(self.ptr.cast::<u8>().as_ptr().add(Self::HEADER_SIZE))
                                .unwrap()
                                .cast(),
                            self.meta,
                        )
                    }
                }
                #final_field_vis fn #final_field_ident_mut(&mut self) -> <#final_field_ty as #unsized_type>::RefMut<'_> {
                    unsafe {
                        <<#final_field_ty as #unsized_type>::RefMut<'_> as #build_pointer_mut>::build_pointer_mut(
                            #non_null::new(self.ptr.cast::<u8>().as_ptr().add(Self::HEADER_SIZE))
                                .unwrap()
                                .cast(),
                            self.meta,
                            |new_len, new_meta| {
                                self.meta = new_meta;
                                let out = (self.resize)(new_len + Self::HEADER_SIZE, new_meta)?;
                                self.ptr = out;
                                Ok(
                                    #non_null::new(out.cast::<u8>().as_ptr().add(Self::HEADER_SIZE))
                                        .unwrap()
                                        .cast::<()>(),
                                )
                            },
                        )
                    }
                }
            }
            impl #a_impl_gen #deref for #ref_mut_ident #a_ty_gen #unsized_where_clause {
                type Target = #ident #ty_gen;

                fn deref(&self) -> &Self::Target {
                    unsafe { &*#ptr::from_raw_parts(self.ptr.as_ptr(), #ptr::metadata(&*self.#final_field_ident_ref())) }
                }
            }
            impl #a_impl_gen #deref_mut for #ref_mut_ident #a_ty_gen #unsized_where_clause {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    unsafe {
                        &mut *#ptr::from_raw_parts_mut(self.ptr.as_ptr(), #ptr::metadata(&*self.#final_field_ident_ref()))
                    }
                }
            }
            impl #a_impl_gen #framework_serialize for #ref_mut_ident #a_ty_gen #unsized_where_clause {
                fn to_bytes(&self, output: &mut &mut [u8]) -> #result<()> {
                    #to_bytes_impl
                    self.#final_field_ident_ref().to_bytes(output)
                }
            }
            unsafe impl #a_impl_gen #framework_from_bytes_mut<#a_lifetime> for #ref_mut_ident #a_ty_gen #unsized_where_clause {
                fn from_bytes_mut(
                    bytes: &mut &#a_lifetime mut [u8],
                    resize: impl #resize_fn<#a_lifetime, Self::Metadata>,
                ) -> #result<Self> {
                    let ptr = #non_null::from(#advance::try_advance(
                        bytes,
                        0 #(+ #size_of::<#pod_field_tys>())*,
                    )?)
                    .cast::<()>();
                    let meta =
                        #pointer_breakup::break_pointer(
                            &<<#final_field_ty as #unsized_type>::RefMut<'_,> as #framework_from_bytes_mut>::from_bytes_mut(
                                bytes,
                                |_, _| #panic!("Cannot resize during `from_bytes_mut`"),
                            )?,
                        )
                        .1;
                    Ok(Self {
                        phantom_tys: #phantom_data,
                        ptr,
                        meta,
                        resize: #box_ty::new(resize),
                    })
                }
            }
            impl #a_impl_gen #pointer_breakup for #ref_mut_ident #a_ty_gen #unsized_where_clause {
                type Metadata = <#final_field_ty as #unsized_type>::RefMeta;

                fn break_pointer(&self) -> (#non_null<()>, Self::Metadata) {
                    (self.ptr.cast(), self.meta)
                }
            }
            impl #a_impl_gen #build_pointer_mut<#a_lifetime> for #ref_mut_ident #a_ty_gen #unsized_where_clause {
                unsafe fn build_pointer_mut(
                    pointee: #non_null<()>,
                    metadata: Self::Metadata,
                    resize: impl #resize_fn<#a_lifetime, Self::Metadata>,
                ) -> Self {
                    Self {
                        phantom_tys: #phantom_data,
                        ptr: pointee,
                        meta: metadata,
                        resize: #box_ty::new(resize),
                    }
                }
            }

            #[automatically_derived]
            unsafe impl #arg_impl_gen #framework_init<#arg_ty> for #ident #ty_gen #arg_where_clause {
                const INIT_LENGTH: usize = <#final_field_ty as #framework_init<#arg_ty>>::INIT_LENGTH #(+ #size_of::<#pod_field_tys>())*;

                unsafe fn init<#a_lifetime>(
                    bytes: &#a_lifetime mut [u8],
                    arg: #arg_ty,
                    resize: impl #resize_fn<#a_lifetime, <Self as #unsized_type>::RefMeta>,
                ) -> #result<Self::RefMut<#a_lifetime>> {
                    let header_size = 0usize #(+ #size_of::<#pod_field_tys>())*;
                    let meta = unsafe {
                        #pointer_breakup::break_pointer(&<#final_field_ty as #framework_init<#arg_ty>>::init(
                            &mut bytes[header_size..],
                            arg,
                            |_, _| #panic!("Cannot resize during `init`"),
                        )?).1
                    };
                    Ok(#ref_mut_ident {
                        phantom_tys: #phantom_data,
                        ptr: #non_null::from(&mut *bytes).cast::<()>(),
                        meta,
                        resize: #box_ty::new(resize),
                    })
                }
            }
        }
    };

    quote! {
        #main_struct
        #ref_struct
        #ref_mut_struct
    }
}
