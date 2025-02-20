use crate::unsize::account::account_impl;
use crate::unsize::UnsizedTypeArgs;
use crate::util::{
    get_doc_attributes, get_repr, make_derivative_attribute, new_generic, new_generics,
    phantom_generics_type, restrict_attributes, strip_inner_attributes, BetterGenerics,
    CombineGenerics, Paths,
};
use heck::ToSnakeCase;
use itertools::{izip, Itertools};
use proc_macro2::{Ident, TokenStream};
use proc_macro_error2::abort;
use quote::{format_ident, quote};
use syn::{
    parse_quote, Attribute, Fields, GenericParam, Generics, ImplGenerics, ItemEnum, ItemStruct,
    Type, TypeGenerics, TypeParam, Variant,
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
    let owned_enum = context.owned_enum();
    let meta_enum = context.meta_enum();
    let ref_wrapper_enum = context.ref_wrapper_enum();
    let variant_structs = context.variant_structs();
    let init_structs = context.init_structs();
    let zeroed_init_impl = context.default_init_impl();
    let enum_trait = context.unsized_enum_impl();
    let unsized_type_impl = context.unsized_type_impl();
    let ext_trait_impl = context.ext_trait_impl();
    let idl_impl = context.idl_impl();

    quote! {
        #enum_struct
        #discriminant_enum
        #owned_enum
        #meta_enum
        #ref_wrapper_enum

        #enum_trait
        #unsized_type_impl
        #variant_structs
        #init_structs
        #zeroed_init_impl
        #ext_trait_impl
        #idl_impl
    }
}

pub struct UnsizedEnumContext {
    item_enum: ItemEnum,
    enum_ident: Ident,
    enum_type: Type,
    discriminant_ident: Ident,
    meta_ident: Ident,
    meta_type: Type,
    owned_ident: Ident,
    owned_type: Type,
    ref_wrapper_ident: Ident,
    variant_idents: Vec<Ident>,
    variant_docs: Vec<Vec<Attribute>>,
    variant_types: Vec<Type>,
    variant_struct_idents: Vec<Ident>,
    variant_struct_types: Vec<Type>,
    init_idents: Vec<Ident>,
    args: UnsizedTypeArgs,
}

impl UnsizedEnumContext {
    fn parse(item_enum: ItemEnum, args: UnsizedTypeArgs) -> Self {
        restrict_attributes(&item_enum, &["default_init", "doc"]);
        let (_, ty_generics, _) = item_enum.generics.split_for_impl();
        let enum_ident = item_enum.ident.clone();
        let enum_type = parse_quote!(#enum_ident #ty_generics);
        let discriminant_ident = format_ident!("{enum_ident}Discriminant");
        let ref_wrapper_ident = format_ident!("{enum_ident}RefWrapper");
        let meta_ident = format_ident!("{enum_ident}Meta");
        let meta_type = parse_quote!(#meta_ident #ty_generics);
        let owned_ident = format_ident!("{enum_ident}Owned");
        let owned_type = parse_quote!(#owned_ident #ty_generics);

        if !args.sized_attributes.attributes.is_empty() {
            abort!(
                args.sized_attributes,
                "Unsized enums may not have `sized_attriubtes`"
            );
        }

        let variant_types = item_enum
            .variants
            .iter()
            .map::<Type, _>(|variant| {
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
                        field.ty.clone()
                    }
                    Fields::Unit => {
                        parse_quote!(())
                    }
                }
            })
            .collect_vec();
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
        let variant_struct_idents = variant_idents
            .iter()
            .map(|var_ident| format_ident!("{enum_ident}Variant{var_ident}"))
            .collect_vec();
        let variant_struct_types = variant_struct_idents
            .iter()
            .map(|var_ident| parse_quote!(#var_ident #ty_generics))
            .collect_vec();
        let init_idents = variant_idents
            .iter()
            .map(|var_ident| format_ident!("{enum_ident}Init{var_ident}"))
            .collect_vec();

        Self {
            item_enum,
            enum_ident,
            enum_type,
            discriminant_ident,
            ref_wrapper_ident,
            meta_ident,
            meta_type,
            owned_ident,
            owned_type,
            variant_idents,
            variant_docs,
            variant_types,
            init_idents,
            variant_struct_idents,
            variant_struct_types,
            args,
        }
    }

    fn generics(&self) -> Generics {
        self.item_enum.generics.clone()
    }

    fn split_for_impl(&self) -> (ImplGenerics, TypeGenerics, Option<&syn::WhereClause>) {
        self.item_enum.generics.split_for_impl()
    }

    fn enum_struct(&self) -> ItemStruct {
        Paths!(prelude, derivative, debug, default, clone, copy);
        UnsizedEnumContext!(self => enum_ident, item_enum);
        let (impl_gen, _, wc) = self.split_for_impl();
        let phantom_generics_type = phantom_generics_type(item_enum);

        let phantom_generics: Option<TokenStream> = phantom_generics_type.map(|ty| quote!((#ty)));

        let derivative_attr =
            make_derivative_attribute::<bool>(parse_quote!(#debug, #default, #clone, #copy), &[]);

        parse_quote! {
            #[repr(C)]
            #[derive(#prelude::Align1, #derivative)]
            #derivative_attr
            pub struct #enum_ident #impl_gen #phantom_generics #wc;
        }
    }

    fn discriminant_enum(&self) -> ItemEnum {
        Paths! {
            debug,
            copy,
            clone,
            eq,
            partial_eq,
            bytemuck,
        }
        UnsizedEnumContext!(self => discriminant_ident, item_enum);
        let discriminant_values = item_enum.variants.iter().map::<Variant, _>(|variant| {
            let ident = &variant.ident;
            let discriminant = &variant
                .discriminant
                .as_ref()
                .map(|(eq, expr)| quote! { #eq #expr });
            parse_quote! {
                #ident #discriminant
            }
        });

        let repr = get_repr(&item_enum.attrs);

        // todo: add common traits to paths
        parse_quote! {
            #[derive(#copy, #clone, #debug, #eq, #partial_eq, Hash, Ord, PartialOrd, #bytemuck::CheckedBitPattern, #bytemuck::NoUninit)]
            #repr
            pub enum #discriminant_ident {
                #(#discriminant_values),*
            }
        }
    }

    fn owned_enum(&self) -> ItemEnum {
        Paths!(prelude, debug);
        UnsizedEnumContext!(self => owned_ident, variant_idents, variant_types, variant_docs, args);
        let additional_owned = args.owned_attributes.attributes.iter();
        let (impl_gen, _, wc) = self.split_for_impl();

        parse_quote! {
            #[derive(#debug)]
            #(#[#additional_owned])*
            pub enum #owned_ident #impl_gen #wc {
                #(
                    #(#variant_docs)*
                    #variant_idents(<#variant_types as #prelude::UnsizedType>::Owned)
                ),*
            }
        }
    }

    fn meta_enum(&self) -> ItemEnum {
        Paths! {prelude, debug, copy, clone}
        UnsizedEnumContext! {self => meta_ident, variant_idents, variant_types, variant_docs}
        let (impl_gen, _, wc) = self.split_for_impl();
        parse_quote! {
            #[derive(#debug, #copy, #clone)]
            pub enum #meta_ident #impl_gen #wc {
                #(
                    #(#variant_docs)*
                    #variant_idents(<#variant_types as #prelude::UnsizedType>::RefMeta)
                ),*
            }
        }
    }

    fn ref_wrapper_enum(&self) -> ItemEnum {
        Paths!(prelude, debug, copy, clone);
        UnsizedEnumContext!(self => ref_wrapper_ident, variant_idents, variant_struct_types, variant_docs);
        let mut generics = self.generics();
        let new_generic = new_generic(&generics);
        generics.params.insert(0, parse_quote!(#new_generic));

        let (impl_gen, _, where_clause) = generics.split_for_impl();
        parse_quote! {
            #[derive(#debug, #copy, #clone)]
            pub enum #ref_wrapper_ident #impl_gen #where_clause {
                #(
                    #(#variant_docs)*
                    #variant_idents(#prelude::UnsizedEnumVariantRef<#new_generic, #variant_struct_types>)
                ),*
            }
        }
    }

    fn variant_structs(&self) -> TokenStream {
        Paths!(derivative, prelude, debug, default, clone, copy);
        UnsizedEnumContext!(self => variant_types, meta_ident, variant_idents, item_enum, enum_type, variant_struct_idents, variant_struct_types, discriminant_ident);

        let derivative_attr =
            make_derivative_attribute::<bool>(parse_quote!(#debug, #default, #clone, #copy), &[]);
        let (impl_gen, _, where_clause) = self.split_for_impl();
        let phantom_generics_type = phantom_generics_type(item_enum);

        quote! {
            #(
                #[derive(#derivative)]
                #derivative_attr
                pub struct #variant_struct_idents #impl_gen (#phantom_generics_type);

                #[automatically_derived]
                unsafe impl #impl_gen #prelude::UnsizedEnumVariant for #variant_struct_types #where_clause {
                    type UnsizedEnum = #enum_type;
                    type InnerType = #variant_types;
                    const DISCRIMINANT: <Self::UnsizedEnum as #prelude::UnsizedEnum>::Discriminant = #discriminant_ident::#variant_idents;
                    fn new_meta(
                        meta: <Self::InnerType as #prelude::UnsizedType>::RefMeta,
                    ) -> <Self::UnsizedEnum as #prelude::UnsizedType>::RefMeta {
                        #meta_ident::#variant_idents(meta)
                    }
                }
            )*
        }
    }

    fn default_init_impl(&self) -> TokenStream {
        Paths!(prelude, size_of, result);
        UnsizedEnumContext!(self => enum_type, item_enum, variant_struct_types, discriminant_ident);
        let mut owned_enum = item_enum.clone();
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
        let init_type = quote!(#prelude::DefaultInit);
        let default_variant_struct = &variant_struct_types[default_init.index];
        let inner_type =
            quote!(<#default_variant_struct as #prelude::UnsizedEnumVariant>::InnerType);

        let mut generics = self.generics();
        generics
            .make_where_clause()
            .predicates
            .push(parse_quote!(#inner_type: #prelude::UnsizedInit<#init_type>));
        let (impl_gen, _, where_clause) = self.split_for_impl();

        let s_gen = new_generic(&generics);
        quote! {
            #[automatically_derived]
            impl #impl_gen #prelude::UnsizedInit<#init_type> for #enum_type #where_clause {
                const INIT_BYTES: usize = #size_of::<#discriminant_ident>() + <#inner_type as #prelude::UnsizedInit<#init_type>>::INIT_BYTES;

                unsafe fn init<#s_gen: #prelude::AsMutBytes>(
                    super_ref: #s_gen,
                    arg: #init_type,
                ) -> #result<#prelude::UnsizedInitReturn<#s_gen, Self>> {
                    <#default_variant_struct as #prelude::UnsizedEnumVariant>::init(super_ref, arg, |init| init)
                }
            }
        }
    }

    fn init_structs(&self) -> TokenStream {
        Paths!(copy, clone, debug, default, prelude, size_of, result);
        UnsizedEnumContext!(self => init_idents, variant_types, enum_type, variant_struct_types, discriminant_ident);

        let init_struct_generic = format_ident!("InitStruct");

        let mut generics = self.generics();
        let init_gen = new_generic(&generics);
        generics.params.push(parse_quote!(#init_gen));
        let init_where_clauses = variant_types
            .iter()
            .map(|ty| {
                let mut init_gens = generics.clone();
                init_gens
                    .make_where_clause()
                    .predicates
                    .push(parse_quote!(#ty: #prelude::UnsizedInit<#init_gen>));
                init_gens.where_clause
            })
            .collect_vec();

        let s_gen = new_generic(&generics);

        let (impl_gen, ..) = generics.split_for_impl();

        quote! {
            #(
                #[derive(#copy, #clone, #debug, #default)]
                pub struct #init_idents<#init_struct_generic>(pub #init_struct_generic);

                #[automatically_derived]
                impl #impl_gen #prelude::UnsizedInit<#init_idents<#init_gen>> for #enum_type #init_where_clauses {
                    const INIT_BYTES: usize = #size_of::<#discriminant_ident>() + <#variant_types as #prelude::UnsizedInit<#init_gen>>::INIT_BYTES;

                    unsafe fn init<#s_gen: #prelude::AsMutBytes>(
                        super_ref: #s_gen,
                        arg: #init_idents<#init_gen>,
                    ) -> #result<#prelude::UnsizedInitReturn<#s_gen, Self>> {
                        <#variant_struct_types as #prelude::UnsizedEnumVariant>::init(super_ref, arg, |init_type| init_type.0)
                    }
                }
            )*
        }
    }

    fn unsized_enum_impl(&self) -> TokenStream {
        Paths!(prelude);
        UnsizedEnumContext!(self => enum_type, variant_idents, meta_ident, discriminant_ident);
        let (impl_gen, _, where_clause) = self.split_for_impl();
        let s = new_generic(&self.item_enum);
        quote! {
            #[allow(clippy::ignored_unit_patterns)]
            #[automatically_derived]
            impl #impl_gen #prelude::UnsizedEnum for #enum_type #where_clause {
                type Discriminant = #discriminant_ident;

                fn discriminant<#s: #prelude::AsBytes>(
                    r: &impl #prelude::RefWrapperTypes<Super = #s, Ref = Self::RefData>,
                ) -> Self::Discriminant {
                    match r.r() {
                        #(
                            #meta_ident::#variant_idents(_) => Self::Discriminant::#variant_idents,
                        )*
                    }
                }
            }
        }
    }

    fn unsized_type_impl(&self) -> TokenStream {
        Paths!(prelude, crate_name, result);
        UnsizedEnumContext! {self =>
            enum_type, meta_type, owned_type, variant_idents, variant_types, meta_ident, discriminant_ident,
            variant_struct_types, ref_wrapper_ident, owned_ident
        }
        let (impl_gen, _, where_clause) = self.split_for_impl();
        let s = new_generic(&self.item_enum);

        quote! {
            #[automatically_derived]
            unsafe impl #impl_gen #prelude::UnsizedType for #enum_type #where_clause {
                type RefMeta = #meta_type;
                type RefData = #meta_type;
                type Owned = #owned_type;
                type IsUnsized = #crate_name::typenum::True;

                fn from_bytes<#s: #prelude::AsBytes>(
                    super_ref: #s,
                ) -> #result<#prelude::FromBytesReturn<#s, Self::RefData, Self::RefMeta>> {
                    match Self::discriminant_from_bytes(&super_ref)? {
                        #(
                            #discriminant_ident::#variant_idents =>
                                unsafe { <#variant_struct_types as #prelude::UnsizedEnumVariant>::from_bytes(super_ref) },
                        )*
                    }
                }

                unsafe fn from_bytes_and_meta<#s: #prelude::AsBytes>(
                    super_ref: #s,
                    meta: Self::RefMeta,
                ) -> #result<#prelude::FromBytesReturn<#s, Self::RefData, Self::RefMeta>> {
                    match meta {
                        #(
                            #meta_ident::#variant_idents(m) => unsafe {
                                <#variant_struct_types as #prelude::UnsizedEnumVariant>::from_bytes_and_meta(super_ref, meta, m)
                            },
                        )*
                    }
                }

                fn owned<#s: #prelude::AsBytes>(r: #prelude::RefWrapper<#s, Self::RefData>) -> #result<Self::Owned> {
                    match r.get()? {
                        #(
                           #ref_wrapper_ident::#variant_idents(r) =>
                                <#variant_types as #prelude::UnsizedType>::owned(r)
                                .map(#owned_ident::#variant_idents),
                        )*
                    }
                }
            }
        }
    }

    fn ext_trait_impl(&self) -> TokenStream {
        Paths!(prelude, result);
        UnsizedEnumContext! {self =>
            ref_wrapper_ident, enum_ident, enum_type, variant_idents, meta_ident,
            variant_types, variant_struct_types, discriminant_ident
        }
        let ext_trait_ident = format_ident!("{enum_ident}Ext");
        let setter_methods = variant_idents
            .iter()
            .map(|var_ident| format_ident!("set_{}", var_ident.to_string().to_snake_case()))
            .collect_vec();

        let mut ref_wrapper_gen = self.generics();
        ref_wrapper_gen.params.insert(
            0,
            GenericParam::Type(TypeParam {
                ident: format_ident!("Self"),
                attrs: vec![],
                colon_token: None,
                bounds: Default::default(),
                eq_token: None,
                default: None,
            }),
        );
        let ref_wrapper_ty_gen = ref_wrapper_gen.split_for_impl().1;

        let mut generics = self.generics();
        generics = generics.combine::<BetterGenerics>(&parse_quote!([where
            Self: Sized + #prelude::RefWrapperTypes<Ref = <#enum_type as #prelude::UnsizedType>::RefData>,
            Self::Super: #prelude::AsBytes
        ]));

        let (impl_gen, ty_gen, where_clause) = generics.split_for_impl();
        let [init_gen, self_gen] = new_generics(&generics);

        let impl_trait_gen = generics.combine::<BetterGenerics>(&parse_quote!([<#self_gen>]));
        let impl_trait_impl_gen = impl_trait_gen.split_for_impl().0;

        let setter_definitions = izip!(setter_methods, variant_struct_types, variant_types)
            .map(|(method, struct_type, variant_type)| quote! {
            fn #method<#init_gen>(self, init: #init_gen) -> #result<#prelude::UnsizedEnumVariantRef<Self, #struct_type>>
            where
                Self: #prelude::RefWrapperMutExt,
                Self::Super: #prelude::Resize<<#enum_type as #prelude::UnsizedType>::RefMeta>,
                #variant_type: #prelude::UnsizedInit<#init_gen>
        }).collect_vec();

        quote! {
            pub trait #ext_trait_ident #impl_gen
            #where_clause
            {
                fn get(self) -> #result<#ref_wrapper_ident #ref_wrapper_ty_gen>;

                #[inline]
                fn discriminant(&self) -> #discriminant_ident {
                    <#enum_type as #prelude::UnsizedEnum>::discriminant(self)
                }

                #(#setter_definitions;)*
            }

            impl #impl_trait_impl_gen #ext_trait_ident #ty_gen for #self_gen #where_clause {
                fn get(self) -> #result<#ref_wrapper_ident #ref_wrapper_ty_gen> {
                    match *self.r() {
                        #(
                            #meta_ident::#variant_idents(m) => Ok(
                                #ref_wrapper_ident::#variant_idents(unsafe {
                                    <#variant_struct_types as #prelude::UnsizedEnumVariant>::get(self, m)?
                                })
                            ),
                        )*
                    }
                }

                #(
                    #setter_definitions {
                        <#variant_struct_types as #prelude::UnsizedEnumVariant>::set(self, init)
                    }
                )*
            }


        }
    }

    fn idl_impl(&self) -> TokenStream {
        account_impl(&self.item_enum.clone().into(), &self.args)
    }
}
