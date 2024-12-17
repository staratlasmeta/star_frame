use crate::unsize::UnsizedTypeArgs;
use crate::util::{get_repr, make_derivative_attribute, new_generic, phantom_generics_type, Paths};
use itertools::Itertools;
use proc_macro2::{Ident, TokenStream};
use proc_macro_error::abort;
use quote::{format_ident, quote};
use syn::{
    parse_quote, Expr, Fields, ImplGenerics, ItemEnum, ItemStruct, Type, TypeGenerics, Variant,
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
    let context = UnsizedEnumContext::parse(item_enum);
    let enum_struct = context.enum_struct();
    let discriminant_enum = context.discriminant_enum();
    let owned_enum = context.owned_enum(unsized_args);
    let meta_enum = context.meta_enum();
    let ref_wrapper_enum = context.ref_wrapper_enum();
    let variant_structs = context.variant_structs();
    let enum_trait = context.unsized_enum_impl();
    let unsized_type_impl = context.unsized_type_impl();

    quote! {
        #enum_struct
        #discriminant_enum
        #owned_enum
        #meta_enum
        #ref_wrapper_enum

        #enum_trait
        #unsized_type_impl
        #variant_structs
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
    variant_types: Vec<Type>,
    variant_struct_idents: Vec<Ident>,
    variant_struct_types: Vec<Type>,
    init_idents: Vec<Ident>,
}

impl UnsizedEnumContext {
    fn parse(item_enum: ItemEnum) -> Self {
        let (_, ty_generics, _) = item_enum.generics.split_for_impl();
        let enum_ident = item_enum.ident.clone();
        let enum_type = parse_quote!(#enum_ident #ty_generics);
        let discriminant_ident = format_ident!("{enum_ident}Discriminant");
        let ref_wrapper_ident = format_ident!("{enum_ident}RefWrapper");
        let meta_ident = format_ident!("{enum_ident}Meta");
        let meta_type = parse_quote!(#meta_ident #ty_generics);
        let owned_ident = format_ident!("{enum_ident}Owned");
        let owned_type = parse_quote!(#owned_ident #ty_generics);
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
            variant_types,
            init_idents,
            variant_struct_idents,
            variant_struct_types,
        }
    }

    fn split_for_impl(&self) -> (ImplGenerics, TypeGenerics, Option<&syn::WhereClause>) {
        self.item_enum.generics.split_for_impl()
    }

    fn enum_struct(&self) -> ItemStruct {
        Paths!(debug);
        UnsizedEnumContext!(self => enum_ident, item_enum);
        let (impl_gen, _, wc) = self.split_for_impl();
        let phantom_generics_type = phantom_generics_type(item_enum);

        let phantom_generics: Option<TokenStream> =
            phantom_generics_type.map(|ty| quote!(__phantom_generics: #ty,));

        parse_quote! {
            #[repr(transparent)]
            #[derive(#debug)]
            pub struct #enum_ident #impl_gen #wc {
                #phantom_generics
                data: [u8]
            }
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

    fn owned_enum(&self, args: UnsizedTypeArgs) -> ItemEnum {
        Paths!(prelude, debug);
        UnsizedEnumContext!(self => owned_ident, variant_idents, variant_types);
        let additional_owned = args.owned_attributes.attributes.iter();
        let (impl_gen, _, wc) = self.split_for_impl();

        parse_quote! {
            #[derive(#debug)]
            #(#[#additional_owned])*
            pub enum #owned_ident #impl_gen #wc {
                #(#variant_idents(<#variant_types as #prelude::UnsizedType>::Owned)),*
            }
        }
    }

    fn meta_enum(&self) -> ItemEnum {
        Paths! {prelude, debug, copy, clone}
        UnsizedEnumContext! {self => meta_ident, variant_idents, variant_types }
        let (impl_gen, _, wc) = self.split_for_impl();
        parse_quote! {
            #[derive(#debug, #copy, #clone)]
            pub enum #meta_ident #impl_gen #wc {
                #(#variant_idents(<#variant_types as #prelude::UnsizedType>::RefMeta)),*
            }
        }
    }

    fn ref_wrapper_enum(&self) -> ItemEnum {
        Paths!(prelude, debug, copy, clone);
        UnsizedEnumContext!(self => ref_wrapper_ident, variant_idents, variant_struct_types, item_enum);
        let mut generics = item_enum.generics.clone();
        let new_generic = new_generic(&generics);
        generics.params.insert(0, parse_quote!(#new_generic));

        let (impl_gen, _, where_clause) = generics.split_for_impl();
        parse_quote! {
            #[derive(#debug, #copy, #clone)]
            pub enum #ref_wrapper_ident #impl_gen #where_clause {
                #(
                    #variant_idents(#prelude::UnsizedEnumVariantRef<#new_generic, #variant_struct_types>)
                ),*
            }
        }
    }

    fn variant_structs(&self) -> TokenStream {
        Paths!(derivative, prelude);
        UnsizedEnumContext!(self => variant_types, meta_ident, variant_idents, item_enum, enum_type, variant_struct_idents, variant_struct_types, discriminant_ident);

        let derivative_attr =
            make_derivative_attribute::<bool>(parse_quote!(Debug, Default, Clone, Copy), &[]);
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

    fn init_structs(&self) -> TokenStream {
        todo!("Finish this")
        // Paths!(copy, clone, debug, default, prelude);
        // UnsizedEnumContext!(self => init_idents, variant_types, meta_ident, variant_idents, item_enum, enum_type, variant_struct_idents, variant_struct_types, discriminant_ident);
        //
        // let init_generic = format_ident!("InitStruct");
        //
        // let mut generics = item_enum.generics.clone();
        // let new_generic = new_generic(&generics);
        // generics.params.insert(0, parse_quote!(#new_generic));
        // // generics.make_where_clause().predicates.push()
        //
        // let derivative_attr =
        //     make_derivative_attribute::<bool>(parse_quote!(Debug, Default, Clone, Copy), &[]);
        // let (impl_gen, _, where_clause) = self.split_for_impl();
        // let phantom_generics_type = phantom_generics_type(item_enum);
        //
        // quote! {
        //     #(
        //         #[derive(#copy, #clone, #debug, #default)]
        //         pub struct #init_idents<#init_generic>(pub #init_generic);
        //
        //         #[automatically_derived]
        //         impl #impl_gen #prelude::UnsizedEnumVariant for #variant_struct_types #where_clause {
        //             type UnsizedEnum = #enum_type;
        //             type InnerType = #variant_types;
        //             const DISCRIMINANT: <Self::UnsizedEnum as #prelude::UnsizedEnum>::Discriminant = #discriminant_ident::#variant_idents;
        //             fn new_meta(
        //                 meta: <Self::InnerType as #prelude::UnsizedType>::RefMeta,
        //             ) -> <Self::UnsizedEnum as #prelude::UnsizedType>::RefMeta {
        //                 #meta_ident::#variant_idents(meta)
        //             }
        //         }
        //     )*
        // }
    }

    fn unsized_enum_impl(&self) -> TokenStream {
        Paths!(prelude);
        UnsizedEnumContext!(self => enum_type, variant_idents, meta_ident, discriminant_ident);
        let (impl_gen, _, where_clause) = self.split_for_impl();
        let s = new_generic(&self.item_enum);
        quote! {
            #[automatically_derived]
            #[allow(clippy::ignored_unit_patterns)]
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
            enum_type, meta_type, owned_type, variant_idents, variant_types, meta_ident, discriminant_ident, variant_struct_idents, ref_wrapper_ident, owned_ident
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
                            #discriminant_ident::#variant_idents => unsafe { #variant_struct_idents::from_bytes(super_ref) },
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
                                #variant_struct_idents::from_bytes_and_meta(super_ref, meta, m)
                            }
                        ),*
                    }
                }

                fn owned<#s: #prelude::AsBytes>(r: #prelude::RefWrapper<#s, Self::RefData>) -> #result<Self::Owned> {
                    match r.get()? {
                        #(
                           #ref_wrapper_ident::#variant_idents(r) =>
                                <#variant_types as #prelude::UnsizedType>::owned(r)
                                .map(#owned_ident::#variant_idents),
                        ),*
                    }
                }
            }
        }
    }
}
