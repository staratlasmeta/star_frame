use crate::{
    unsize::{account::account_impl, UnsizedTypeArgs},
    util::{
        combine_gen, get_doc_attributes, get_repr, new_generic, new_lifetime,
        phantom_generics_type, restrict_attributes, strip_inner_attributes, IntegerRepr, Paths,
        Representation,
    },
};
use heck::{ToShoutySnakeCase, ToSnakeCase};
use itertools::Itertools;
use proc_macro2::{Ident, TokenStream};
use proc_macro_error2::{abort, ResultExt};
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse2, parse_quote, AngleBracketedGenericArguments, Attribute, Fields, Generics, ItemEnum,
    Lifetime, Type, Visibility,
};

#[allow(non_snake_case)]
macro_rules! UnsizedEnumContext {
    ($expr:expr => $($name:ident $(: $rename:ident)? $(,)?)*) => {
        let UnsizedEnumContext {
            $($name $(: $rename)? ,)*
            ..
        } = $expr;
    };
}

pub(crate) fn unsized_type_enum_impl(
    item_enum: ItemEnum,
    unsized_args: UnsizedTypeArgs,
) -> TokenStream {
    let context = UnsizedEnumContext::parse(item_enum, unsized_args);
    let enum_struct = context.enum_struct();
    let discriminant_enum = context.discriminant_enum();
    let owned_enum = context
        .args
        .owned_type
        .is_none()
        .then(|| context.owned_enum());
    let ref_enum = context.ref_enum();
    let mut_enum = context.mut_enum();
    let as_shared_impl = context.as_shared_impl();
    let from_owned_impl = context.from_owned_impl();
    let unsized_type_mut_impl = context.unsized_type_mut_impl();
    let unsized_type_impl = context.unsized_type_impl();
    let unsized_init_default_impl = context.unsized_init_default_impl();
    let unsized_init_struct_impls = context.unsized_init_struct_impl();
    let extension_impl = context.extension_impl();
    let idl_impl = context.idl_impl();

    quote! {
        #enum_struct
        #discriminant_enum
        #owned_enum
        #ref_enum
        #mut_enum
        #as_shared_impl
        #from_owned_impl
        #unsized_type_mut_impl
        #unsized_type_impl
        #unsized_init_default_impl
        #unsized_init_struct_impls
        #extension_impl
        #idl_impl
    }
}

pub struct UnsizedEnumContext {
    item_enum: ItemEnum,
    vis: Visibility,
    generics: Generics,
    enum_ident: Ident,
    repr: Representation,
    enum_type: Type,
    discriminant_ident: Ident,
    discriminant_values: Vec<TokenStream>,
    ref_ident: Ident,
    ref_type: Type,
    mut_ident: Ident,
    mut_type: Type,
    owned_ident: Ident,
    owned_type: Type,
    variant_idents: Vec<Ident>,
    variant_docs: Vec<Vec<Attribute>>,
    filtered_variant_types: Vec<Type>,
    variant_types: Vec<Option<Type>>,
    ref_mut_generics: Generics,
    top_lt: Lifetime,
    args: UnsizedTypeArgs,
    integer_repr: IntegerRepr,
}

impl UnsizedEnumContext {
    fn parse(item_enum: ItemEnum, args: UnsizedTypeArgs) -> Self {
        restrict_attributes(&item_enum, &["default_init", "doc"]);
        let top_lt = new_lifetime(&item_enum.generics, Some("top"));
        let ref_mut_generics = combine_gen!(item_enum.generics; <#top_lt>);
        let ref_mut_type_generics = ref_mut_generics.split_for_impl().1;
        let type_generics = item_enum.generics.split_for_impl().1;
        let enum_ident = item_enum.ident.clone();
        let enum_type = parse_quote!(#enum_ident #type_generics);
        let discriminant_ident = format_ident!("{enum_ident}Discriminants");
        let ref_ident = format_ident!("{enum_ident}Ref");
        let ref_type = parse_quote!(#ref_ident #ref_mut_type_generics);
        let mut_ident = format_ident!("{enum_ident}Mut");
        let mut_type = parse_quote!(#mut_ident #ref_mut_type_generics);
        let owned_ident = format_ident!("{enum_ident}Owned");
        let owned_type = parse_quote!(#owned_ident #type_generics);
        let repr = get_repr(&item_enum.attrs);
        let integer_repr = repr.repr.as_integer().unwrap_or_else(|| {
            abort!(
                item_enum,
                "Unsized enums must have an integer representation, like `#[repr(u8)]`"
            );
        });

        if !args.sized_attributes.attributes.is_empty() {
            abort!(
                args.sized_attributes,
                "Unsized enums may not have `sized_attriubtes`"
            );
        }

        let variant_types = item_enum
            .variants
            .iter()
            .map::<Option<Type>, _>(|variant| {
                const UNIT_ERROR: &str = "Unsized enums must be unit variants, or single tuples";
                match &variant.fields {
                    Fields::Named(fields_named) => {
                        abort!(fields_named, UNIT_ERROR)
                    }
                    Fields::Unnamed(fields) => {
                        if fields.unnamed.len() != 1 {
                            abort!(fields, UNIT_ERROR)
                        }
                        let field = &fields.unnamed[0];
                        restrict_attributes(&field.attrs, &[]);
                        Some(field.ty.clone())
                    }
                    Fields::Unit => None,
                }
            })
            .collect_vec();

        if variant_types.is_empty() {
            abort!(item_enum, "Unsized enums must have at least one variant");
        }

        let variant_idents = item_enum
            .variants
            .iter()
            .map(|variant| variant.ident.clone())
            .collect_vec();
        let variant_docs = item_enum
            .variants
            .iter()
            .map(|variant| get_doc_attributes(&variant.attrs))
            .collect_vec();

        let discriminant_values = item_enum
            .variants
            .iter()
            .map(|variant| {
                variant
                    .discriminant
                    .as_ref()
                    .map(|(eq, expr)| quote! { #eq #expr })
                    .unwrap_or_default()
            })
            .collect_vec();

        let filtered_variant_types = variant_types.iter().flatten().cloned().collect_vec();

        Self {
            vis: item_enum.vis.clone(),
            generics: item_enum.generics.clone(),
            ref_mut_generics,
            top_lt,
            discriminant_values,
            repr,
            integer_repr,
            item_enum,
            enum_ident,
            enum_type,
            discriminant_ident,
            ref_ident,
            ref_type,
            mut_ident,
            mut_type,
            owned_ident,
            owned_type,
            variant_idents,
            variant_docs,
            variant_types,
            filtered_variant_types,
            args,
        }
    }

    fn map_variants(
        &self,
        some: impl Fn(&Ident, &Type) -> TokenStream,
        none: impl Fn(&Ident) -> TokenStream,
    ) -> Vec<TokenStream> {
        self.variant_types
            .iter()
            .zip_eq(self.variant_idents.iter())
            .map(|(variant, variant_ident)| {
                if let Some(variant) = variant {
                    some(variant_ident, variant)
                } else {
                    none(variant_ident)
                }
            })
            .collect_vec()
    }

    fn init_ident(&self, variant_ident: &Ident) -> Ident {
        let enum_ident = &self.enum_ident;
        format_ident!("{enum_ident}Init{variant_ident}")
    }

    fn make_variants(&self, inner: impl Fn(&Type) -> TokenStream) -> Vec<TokenStream> {
        self.map_variants(
            |variant_ident, variant_type| {
                let inner = inner(variant_type);
                quote! { #variant_ident(#inner) }
            },
            |variant_ident| quote! { #variant_ident },
        )
    }

    fn split_for_declaration(&self, ref_mut: bool) -> (&Generics, Option<&syn::WhereClause>) {
        let the_generics = if ref_mut {
            &self.ref_mut_generics
        } else {
            &self.generics
        };
        (the_generics, the_generics.where_clause.as_ref())
    }

    fn enum_struct(&self) -> TokenStream {
        Paths!(prelude, debug);
        UnsizedEnumContext!(self => enum_ident, generics, vis);

        let wc = &generics.where_clause;
        let phantom_generics_type = phantom_generics_type(generics);

        let phantom_generics: Option<TokenStream> = phantom_generics_type.map(|ty| quote!((#ty)));

        let derives = if phantom_generics.is_some() {
            quote! {
                #[derive(#prelude::DeriveWhere)]
                #[derive_where(Debug)]
            }
        } else {
            quote!(#[derive(#debug)])
        };

        quote! {
            #[repr(C)]
            #derives
            #vis struct #enum_ident #generics #phantom_generics #wc;
        }
    }

    fn discriminant_enum(&self) -> TokenStream {
        Paths!(debug, copy, clone, eq, partial_eq, bytemuck);
        UnsizedEnumContext!(self => vis, discriminant_ident, variant_idents, repr, discriminant_values);

        quote! {
            #[derive(#copy, #clone, #debug, #eq, #partial_eq, Hash, Ord, PartialOrd, #bytemuck::NoUninit)]
            #repr
            #vis enum #discriminant_ident {
                #(#variant_idents #discriminant_values,)*
            }
        }
    }

    fn owned_enum(&self) -> TokenStream {
        Paths!(prelude);
        UnsizedEnumContext!(self => owned_ident, variant_docs, args, generics, integer_repr, discriminant_values, filtered_variant_types);
        let additional_owned = args.owned_attributes.attributes.iter();
        let wc = &generics.where_clause;
        let lt = new_lifetime(generics, None);

        let variants = self.make_variants(|variant_type| {
            quote! {
                <#variant_type as #prelude::UnsizedType>::Owned
            }
        });

        quote! {
            #(#[#additional_owned])*
            #[derive(#prelude::DeriveWhere)]
            #[derive_where(Debug, Copy, Clone, Eq, Hash, Ord, PartialEq, PartialOrd; #(for<#lt> <#filtered_variant_types as #prelude::UnsizedType>::Owned,)*)]
            #[repr(#integer_repr)]
            pub enum #owned_ident #generics #wc {
                #(
                    #(#variant_docs)*
                    #variants #discriminant_values,
                )*
            }
        }
    }

    fn ref_enum(&self) -> TokenStream {
        Paths!(prelude);
        UnsizedEnumContext!(self => ref_ident, variant_docs, top_lt, filtered_variant_types);
        let (generics, wc) = self.split_for_declaration(true);

        let variants = self.make_variants(|variant_type| {
            quote! {
                <#variant_type as #prelude::UnsizedType>::Ref<#top_lt>
            }
        });

        quote! {
            #[derive(#prelude::DeriveWhere)]
            #[derive_where(Debug; #(<#filtered_variant_types as #prelude::UnsizedType>::Ref<#top_lt>,)*)]
            pub enum #ref_ident #generics #wc {
                #(
                    #(#variant_docs)*
                    #variants,
                )*
            }
        }
    }

    fn mut_enum(&self) -> TokenStream {
        Paths!(prelude);
        UnsizedEnumContext!(self => integer_repr, mut_ident, variant_docs, top_lt, filtered_variant_types);
        let (generics, wc) = self.split_for_declaration(true);

        let variants = self.make_variants(|variant_type| {
            quote! {
                <#variant_type as #prelude::UnsizedType>::Mut<#top_lt>
            }
        });

        quote! {
            #[derive(#prelude::DeriveWhere)]
            #[derive_where(Debug; #(<#filtered_variant_types as #prelude::UnsizedType>::Mut<#top_lt>,)*)]
            #[repr(#integer_repr)]
            pub enum #mut_ident #generics #wc {
                #(
                    #(#variant_docs)*
                    #variants,
                )*
            }
        }
    }

    fn as_shared_impl(&self) -> TokenStream {
        Paths!(prelude);
        UnsizedEnumContext!(self => ref_type, top_lt, ref_ident, mut_ident);
        let underscore_gen = combine_gen!(self.generics; <'_>);
        let underscore_ty_gen = underscore_gen.split_for_impl().1;

        let (impl_gen, _, where_clause) = self.generics.split_for_impl();

        let variant_matches = self.make_variants(|_| quote!(inner));

        let variant_returns_ref = self.make_variants(|variant_type| {
            quote! {
                <#variant_type as #prelude::UnsizedType>::ref_as_ref(inner)
            }
        });
        let variant_returns_mut = self.make_variants(|variant_type| {
            quote! {
                <#variant_type as #prelude::UnsizedType>::mut_as_ref(inner)
            }
        });

        quote! {
            #[automatically_derived]
            impl #impl_gen #prelude::AsShared for #ref_ident #underscore_ty_gen #where_clause {
                type Ref<#top_lt> = #ref_type
                    where Self: #top_lt;
                fn as_shared(&self) -> Self::Ref<'_> {
                    match self {
                        #(#ref_ident::#variant_matches => #ref_ident::#variant_returns_ref),*
                    }
                }
            }

            #[automatically_derived]
            impl #impl_gen #prelude::AsShared for #mut_ident #underscore_ty_gen #where_clause {
                type Ref<#top_lt> = #ref_type
                    where Self: #top_lt;
                fn as_shared(&self) -> Self::Ref<'_> {
                    match self {
                        #(#mut_ident::#variant_matches => #ref_ident::#variant_returns_mut),*
                    }
                }
            }
        }
    }

    #[allow(clippy::wrong_self_convention)]
    fn from_owned_impl(&self) -> Option<TokenStream> {
        if self.args.owned_type.is_some() {
            return None;
        }
        Paths!(prelude, result, size_of, bytemuck);
        UnsizedEnumContext!(self => enum_type, owned_ident, filtered_variant_types, variant_idents, integer_repr, discriminant_ident);

        let from_owned_generics =
            combine_gen!(self.generics; where #(#filtered_variant_types: #prelude::FromOwned),*);

        let (impl_gen, _, where_clause) = from_owned_generics.split_for_impl();
        let variant_matches = self.make_variants(|_| quote!(inner));

        let variant_size_returns = self.map_variants(
            |_, ty| {
                quote! {
                    <#ty as #prelude::FromOwned>::byte_size(inner)
                }
            },
            |_| {
                quote! {
                    0
                }
            },
        );

        let from_owned_returns = self.map_variants(
            |_, ty| {
                quote! {
                    <#ty as #prelude::FromOwned>::from_owned(inner, bytes)?
                }
            },
            |_| {
                quote! {
                    0
                }
            },
        );

        Some(quote! {
            #[automatically_derived]
            impl #impl_gen #prelude::FromOwned for #enum_type #where_clause {
                #[inline]
                fn byte_size(owned: &Self::Owned) -> usize {
                    let variant_size = match owned {
                        #(#owned_ident::#variant_matches => #variant_size_returns),*
                    };
                    #size_of::<#discriminant_ident>() + variant_size
                }

                #[inline]
                fn from_owned(owned: Self::Owned, bytes: &mut &mut [u8]) -> #result<usize> {
                    let variant_bytes = #prelude::Advance::try_advance(bytes, #size_of::<#discriminant_ident>())?;
                    let (variant_size, discriminant) = match owned {
                        #(#owned_ident::#variant_matches => (
                            #from_owned_returns,
                            #discriminant_ident::#variant_idents,
                        ),)*
                    };
                    variant_bytes.copy_from_slice(#bytemuck::bytes_of(&(discriminant as #integer_repr)));
                    Ok(#size_of::<#discriminant_ident>() + variant_size)
                }
            }
        })
    }

    fn unsized_type_mut_impl(&self) -> TokenStream {
        Paths!(prelude);
        UnsizedEnumContext!(self => enum_type, mut_type);
        let (impl_gen, _, where_clause) = self.ref_mut_generics.split_for_impl();

        quote! {
            #[automatically_derived]
            unsafe impl #impl_gen #prelude::UnsizedTypeMut for #mut_type #where_clause {
                type UnsizedType = #enum_type;
            }
        }
    }

    fn unsized_type_impl(&self) -> TokenStream {
        Paths!(prelude, result, size_of);
        UnsizedEnumContext!(self => ref_type, top_lt,
            ref_ident, mut_type, mut_ident, enum_type, integer_repr,
            discriminant_ident, variant_idents, owned_ident, filtered_variant_types
        );
        let (impl_gen, _, where_clause) = self.generics.split_for_impl();
        let discriminant_consts = self
            .variant_idents
            .iter()
            .map(|var_ident| format_ident!("{}", var_ident.to_string().to_shouty_snake_case()))
            .collect_vec();

        let variant_owned_from_ref = self.make_variants(|variant_type| {
            quote! {
                <#variant_type as #prelude::UnsizedType>::owned_from_ref(inner)?
            }
        });

        let variant_matches = self.make_variants(|_| quote!(inner));

        let owned_type = self.args.owned_type.as_ref().unwrap_or(&self.owned_type);

        let owned_from_ref = self
            .args
            .owned_from_ref
            .as_ref()
            .map(|path| quote!(#path(r)))
            .unwrap_or(quote! {
                match r {
                        #(
                            #ref_ident::#variant_matches => Ok(#owned_ident::#variant_owned_from_ref),
                        )*
                    }
            });

        let mut_lt = new_lifetime(&self.generics, Some("m"));

        let variant_ref_as_ref = self.make_variants(|variant_type| {
            quote! {
                <#variant_type as #prelude::UnsizedType>::ref_as_ref(inner)
            }
        });

        let variant_mut_as_ref = self.make_variants(|variant_type| {
            quote! {
                <#variant_type as #prelude::UnsizedType>::mut_as_ref(inner)
            }
        });

        let variant_get_ref = self.make_variants(|variant_type| {
            quote! {
                <#variant_type as #prelude::UnsizedType>::get_ref(data)?
            }
        });

        let variant_get_mut = self.make_variants(|variant_type| {
            quote! {
                unsafe { <#variant_type as #prelude::UnsizedType>::get_mut(data)? }
            }
        });

        let variant_resize_notification = self.map_variants(|_, variant_type| {
            quote! {
                unsafe { <#variant_type as #prelude::UnsizedType>::resize_notification(inner, source_ptr, change) }
            }
        }, |_| quote!(Ok(())));

        let variant_data_len = self.map_variants(
            |_, variant_type| {
                quote! {
                    <#variant_type as #prelude::UnsizedType>::data_len(inner)
                }
            },
            |_| quote!(0),
        );

        quote! {
            #[automatically_derived]
            unsafe impl #impl_gen #prelude::UnsizedType for #enum_type #where_clause {
                type Ref<#top_lt> = #ref_type;
                type Mut<#top_lt> = #prelude::StartPointer<#mut_type>;
                type Owned = #owned_type;

                const ZST_STATUS: bool = {
                    true #(&& <#filtered_variant_types as #prelude::UnsizedType>::ZST_STATUS)*
                };

                fn ref_as_ref<#mut_lt>(r: &#mut_lt Self::Ref<'_>) -> Self::Ref<#mut_lt> {
                    match &r {
                        #(
                            #ref_ident::#variant_matches => #ref_ident::#variant_ref_as_ref,
                        )*
                    }
                }

                fn mut_as_ref<#mut_lt>(m: &#mut_lt Self::Mut<'_>) -> Self::Ref<#mut_lt> {
                    match &m.data {
                        #(
                            #mut_ident::#variant_matches => #ref_ident::#variant_mut_as_ref,
                        )*
                    }
                }

                fn get_ref<#top_lt>(data: &mut &#top_lt [u8]) -> #result<Self::Ref<#top_lt>> {
                    #(const #discriminant_consts: #integer_repr = #discriminant_ident::#variant_idents as #integer_repr;)*
                    let maybe_repr_bytes_ptr = #prelude::AdvanceArray::try_advance_array(data);
                    let repr_bytes_ptr = #prelude::ErrorInfo::with_ctx(maybe_repr_bytes_ptr, || format!("Not enough bytes to get enum discriminant of {}", #prelude::type_name::<#enum_type>()))?;
                    let repr: #integer_repr = <#integer_repr>::from_le_bytes(*repr_bytes_ptr);
                    match repr {
                        #(
                            #discriminant_consts =>
                                Ok(#ref_ident::#variant_get_ref),
                        )*
                        _ => #prelude::bail!(#prelude::ProgramError::InvalidAccountData, "Invalid enum discriminant for {} during get_ref", #prelude::type_name::<#enum_type>()),
                    }
                }

                unsafe fn get_mut<#top_lt>(data: &mut *mut [u8]) -> #result<Self::Mut<#top_lt>> {
                    #(const #discriminant_consts: #integer_repr = #discriminant_ident::#variant_idents as #integer_repr;)*
                    let start_ptr = data.cast::<()>();
                    let maybe_repr_bytes = #prelude::RawSliceAdvance::try_advance(data, #size_of::<#integer_repr>());
                    let repr_ptr = #prelude::ErrorInfo::with_ctx(maybe_repr_bytes, || format!("Not enough bytes to get enum discriminant of {}", #prelude::type_name::<#enum_type>()))?;
                    let repr_bytes = unsafe { repr_ptr.cast::<[u8; #size_of::<#integer_repr>()]>().read() };
                    let repr: #integer_repr = <#integer_repr>::from_le_bytes(repr_bytes);
                    let res = match repr {
                        #(
                            #discriminant_consts =>
                                #mut_ident::#variant_get_mut,
                        )*
                        _ => #prelude::bail!(#prelude::ProgramError::InvalidAccountData, "Invalid enum discriminant for {} during get_mut", #prelude::type_name::<#enum_type>()),
                    };
                    Ok(unsafe { #prelude::StartPointer::new(res, start_ptr) })
                }

                fn start_ptr(m: &Self::Mut<'_>) -> *mut () {
                    #prelude::StartPointer::start_ptr(m)
                }

                fn data_len(m: &Self::Mut<'_>) -> usize {
                    #size_of::<#integer_repr>() + match &m.data {
                        #(
                            #mut_ident::#variant_matches => #variant_data_len,
                        )*
                    }
                }

                fn owned_from_ref(r: &Self::Ref<'_>) -> #result<Self::Owned> {
                    #owned_from_ref
                }

                unsafe fn resize_notification(self_mut: &mut Self::Mut<'_>, source_ptr: *const (), change: isize) -> #result<()> {
                    unsafe {
                        Self::Mut::handle_resize_notification(self_mut, source_ptr, change);
                    }
                    match &mut self_mut.data {
                        #(
                            #mut_ident::#variant_matches => {
                                #variant_resize_notification
                            },
                        )*
                    }
                }
            }
        }
    }

    fn unsized_init_default_impl(&self) -> TokenStream {
        Paths!(prelude, result, size_of, bytemuck);
        UnsizedEnumContext!(self => enum_type, discriminant_ident, integer_repr);
        let mut owned_enum = self.item_enum.clone();
        let mut default_inits = strip_inner_attributes(&mut owned_enum, "default_init");
        let Some(default_init) = default_inits.next() else {
            return quote! {};
        };
        if let Some(extra_init) = default_inits.next() {
            abort!(
                extra_init.attribute,
                "Unsized enums may only have one `#[default_init]` attribute"
            );
        }
        let unsized_init = quote!(#prelude::UnsizedInit<#prelude::DefaultInit>);

        let variant_type = self.variant_types[default_init.index].as_ref();

        let variant_size = variant_type
            .map(|ty| quote!(<#ty as #unsized_init>::INIT_BYTES))
            .unwrap_or(quote!(0));

        let variant_init = variant_type
            .map(|ty| quote!(unsafe { <#ty as #unsized_init>::init(bytes, arg) }))
            .unwrap_or(quote!(Ok(())));

        let variant_ident = &self.variant_idents[default_init.index];
        let default_init_generics = if let Some(variant_type) = variant_type {
            combine_gen!(self.generics; where #variant_type: #unsized_init)
        } else {
            self.generics.clone()
        };

        let (default_init_impl, _, default_init_where) = default_init_generics.split_for_impl();
        quote! {
            #[allow(trivial_bounds)]
            #[automatically_derived]
            impl #default_init_impl #unsized_init for #enum_type #default_init_where {
                const INIT_BYTES: usize = #variant_size + #size_of::<#discriminant_ident>();

                fn init(
                    bytes: &mut &mut [u8],
                    arg: #prelude::DefaultInit,
                ) -> #result<()> {
                    #prelude::Advance::try_advance(bytes, #size_of::<#discriminant_ident>())?
                        .copy_from_slice(#bytemuck::bytes_of(&(#discriminant_ident::#variant_ident as #integer_repr)));
                    #variant_init
                }
            }
        }
    }

    fn unsized_init_struct_impl(&self) -> TokenStream {
        Paths!(prelude, result, size_of, bytemuck, copy, clone, debug, default);
        UnsizedEnumContext!(self => enum_type, discriminant_ident, integer_repr, vis, variant_idents);
        let init_generic = new_generic(&self.generics, Some("Init"));

        let init_generic_trait = quote!(#prelude::UnsizedInit<#init_generic>);
        let all_generics = self
            .variant_types
            .iter()
            .map(|variant_ty| {
                if let Some(variant_ty) = variant_ty {
                    combine_gen!(self.generics; <#init_generic> where #variant_ty: #init_generic_trait)
                } else {
                    self.generics.clone()
                }
            })
            .collect_vec();

        let (impl_gens, where_clauses): (Vec<_>, Vec<_>) = all_generics
            .iter()
            .map(|gen| {
                let (impl_gen, _, wc) = gen.split_for_impl();
                (impl_gen, wc)
            })
            .unzip();

        let variant_sizes = self.map_variants(
            |_, variant_type| {
                quote! {
                    <#variant_type as #init_generic_trait>::INIT_BYTES
                }
            },
            |_| quote!(0),
        );

        let variant_inits = self.map_variants(
            |_, variant_type| {
                quote! {
                    unsafe { <#variant_type as #init_generic_trait>::init(bytes, arg.0) }
                }
            },
            |_| quote!(Ok(())),
        );

        let (init_ident_structs, init_arg): (Vec<_>, Vec<_>) = self
            .variant_types
            .iter()
            .zip(variant_idents.iter())
            .map(|(variant_type, variant_ident)| {
                let init_ident = self.init_ident(variant_ident);
                let (struct_body, init_arg) = if variant_type.is_some() {
                    (
                        quote! {
                            <#init_generic>(#vis #init_generic)
                        },
                        quote!(#init_ident<#init_generic>),
                    )
                } else {
                    (quote!(), quote!(#init_ident))
                };

                (
                    quote! {
                        #[derive(#copy, #clone, #debug, #default)]
                        #vis struct #init_ident #struct_body;
                    },
                    init_arg,
                )
            })
            .unzip();

        quote! {
            #(
                #init_ident_structs

                #[allow(trivial_bounds)]
                #[automatically_derived]
                impl #impl_gens #prelude::UnsizedInit<#init_arg> for #enum_type #where_clauses {
                    const INIT_BYTES: usize = #variant_sizes + #size_of::<#discriminant_ident>();

                    fn init(
                        bytes: &mut &mut [u8],
                        arg: #init_arg,
                    ) -> #result<()> {
                        #prelude::Advance::try_advance(bytes, #size_of::<#discriminant_ident>())?
                            .copy_from_slice(#bytemuck::bytes_of(&(#discriminant_ident::#variant_idents as #integer_repr)));
                        #variant_inits
                    }
                }
            )*
        }
    }

    fn extension_impl(&self) -> TokenStream {
        Paths!(prelude, ptr, result, sized);
        UnsizedEnumContext!(self => vis, enum_ident, mut_ident, enum_type, top_lt, mut_type, filtered_variant_types);

        // Create new lifetimes and generics for the extension trait
        let parent_lt = new_lifetime(&self.generics, Some("parent"));
        let child_lt = new_lifetime(&self.generics, Some("child"));
        let p = new_generic(&self.generics, Some("P"));
        let init = new_generic(&self.generics, Some("Init"));

        let extension_ident = format_ident!("{enum_ident}ExclusiveExt");

        let ext_trait_generics = combine_gen!(self.generics;
            <#parent_lt, #top_lt, #p>
        );

        let (impl_gen, ty_gen, wc) = ext_trait_generics.split_for_impl();
        let exclusive_ident = format_ident!("{enum_ident}Exclusive");
        let exclusive_variants = self.make_variants(|variant_ty| {
            quote!(#prelude::ExclusiveWrapper<#parent_lt, #top_lt, <#variant_ty as #prelude::UnsizedType>::Mut<#top_lt>, #p>)
        });

        let exclusive_enum = {
            quote! {
                #[derive(#prelude::DeriveWhere)]
                #[derive_where(Debug; #(#prelude::ExclusiveWrapper<#parent_lt, #top_lt, <#filtered_variant_types as #prelude::UnsizedType>::Mut<#top_lt>, #p>,)*)]
                #vis enum #exclusive_ident #ext_trait_generics #wc {
                    #(#exclusive_variants,)*
                }
            }
        };

        let enum_as_mut = quote!(<#enum_type as #prelude::UnsizedType>::Mut<#top_lt>);

        let get_return_gen = combine_gen!(self.generics; <#child_lt, #top_lt>);
        let get_return_gen_tt = get_return_gen.split_for_impl().1.to_token_stream();
        let mut get_return_gen_args: AngleBracketedGenericArguments =
            parse2(get_return_gen_tt).unwrap_or_abort();
        get_return_gen_args.args.push(parse_quote!(Self));

        let ext_impl_trait_generics = combine_gen!(ext_trait_generics;
            where
                Self: #prelude::ExclusiveRecurse + #sized,
                #enum_type: #prelude::UnsizedType<Mut<'top> = #prelude::StartPointer<#mut_type>>
        );
        let impl_wc = ext_impl_trait_generics.split_for_impl().2;

        let variant_matches = self.make_variants(|_| quote!(variant));

        let getter_bodies = self.map_variants(
            |variant_ident, variant_type| {
                quote! {
                    let variant_addr = #ptr::from_ref(variant).addr();
                    #exclusive_ident::#variant_ident(unsafe {
                        #prelude::ExclusiveWrapper::map_mut::<#variant_type>(self, |inner| {
                            inner.with_addr(variant_addr).cast::<<#variant_type as #prelude::UnsizedType>::Mut<#top_lt>>()
                        })
                    })
                }
            },
            |variant_ident| quote!(#exclusive_ident::#variant_ident),
        );

        let make_setter_ident = |variant_ident: &Ident| {
            format_ident!("set_{}", variant_ident.to_string().to_snake_case())
        };

        let setter_definitions = self.map_variants(
            |variant_ident, variant_type| {
                let setter_method = make_setter_ident(variant_ident);
                let init_ident = self.init_ident(variant_ident);
                quote! {
                    fn #setter_method<#child_lt, #init>(&#child_lt mut self, init: #init) -> #result<#prelude::ExclusiveWrapper<#child_lt, #top_lt, <#variant_type as #prelude::UnsizedType>::Mut<#top_lt>, Self>>
                    where
                        #enum_type: #prelude::UnsizedInit<#init_ident<#init>>
                }
            },
            |variant_ident| {
                let setter_method = make_setter_ident(variant_ident);
                quote! {
                    fn #setter_method(&mut self) -> #result<()>
                }
            },
        );

        let setter_bodies = self.map_variants(
            |variant_ident, variant_type| {
                let init_ident = self.init_ident(variant_ident);

                quote! {
                    unsafe {
                        Self::set_from_init(
                            self,
                            #init_ident(init)
                        )?;
                    }
                    let #mut_ident::#variant_ident(variant) = &***self else {
                        ::core::unreachable!();
                    };
                    let variant_addr = #ptr::from_ref(variant).addr();
                    Ok(unsafe {
                        #prelude::ExclusiveWrapper::map_mut::<#variant_type>(self, |inner| {
                            inner.with_addr(variant_addr).cast::<<#variant_type as #prelude::UnsizedType>::Mut<#top_lt>>()
                        })
                    })
                }
            },
            |variant_ident| {
                let init_ident = self.init_ident(variant_ident);
                quote! {
                    Self::set_from_init(
                        self,
                        #init_ident
                    )
                }
            },
        );

        let extension_trait = quote! {
            #vis trait #extension_ident #impl_gen #impl_wc
            {
                fn get<#child_lt>(&#child_lt mut self) -> #exclusive_ident #get_return_gen_args;

                #(#setter_definitions;)*
            }

            #[automatically_derived]
            impl #impl_gen #extension_ident #ty_gen for #prelude::ExclusiveWrapper<#parent_lt, #top_lt, #enum_as_mut, #p> #impl_wc
            {
                fn get<#child_lt>(&#child_lt mut self) -> #exclusive_ident #get_return_gen_args {
                    match &***self {
                        #(
                            #mut_ident::#variant_matches => {
                                #getter_bodies
                            }
                        )*
                    }
                }

                #(
                    #setter_definitions {
                        #setter_bodies
                    }
                )*
            }
        };

        quote! {
            #exclusive_enum
            #extension_trait
        }
    }

    fn idl_impl(&self) -> TokenStream {
        account_impl(&self.item_enum.clone().into(), &self.args)
    }
}
