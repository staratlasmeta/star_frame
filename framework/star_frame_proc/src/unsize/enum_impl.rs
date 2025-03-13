use crate::unsize::account::account_impl;
use crate::unsize::UnsizedTypeArgs;
use crate::util::{
    get_doc_attributes, get_repr, new_generic, new_lifetime, phantom_generics_type,
    restrict_attributes, strip_inner_attributes, BetterGenerics, CombineGenerics, IntegerRepr,
    Paths, Representation,
};
use heck::{ToShoutySnakeCase, ToSnakeCase};
use itertools::Itertools;
use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error2::abort;
use quote::{format_ident, quote};
use syn::{
    parse_quote, Attribute, Fields, Generics, ItemEnum, ItemStruct, Lifetime, Type, Visibility,
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
    let ref_enum = context.ref_enum();
    let mut_enum = context.mut_enum();
    let as_shared_impl = context.as_shared_impl();
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
    variant_types: Vec<Type>,
    ref_mut_generics: Generics,
    rm_lt: Lifetime,
    init_idents: Vec<Ident>,
    args: UnsizedTypeArgs,
    integer_repr: IntegerRepr,
}

impl UnsizedEnumContext {
    fn parse(item_enum: ItemEnum, args: UnsizedTypeArgs) -> Self {
        restrict_attributes(&item_enum, &["default_init", "doc"]);
        let ref_mut_lifetime = new_lifetime(&item_enum.generics, None);
        let ref_mut_generics = item_enum
            .generics
            .combine::<BetterGenerics>(&parse_quote!([<#ref_mut_lifetime>]));
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

        let init_idents = variant_idents
            .iter()
            .map(|var_ident| format_ident!("{enum_ident}Init{var_ident}"))
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

        Self {
            vis: item_enum.vis.clone(),
            generics: item_enum.generics.clone(),
            ref_mut_generics,
            rm_lt: ref_mut_lifetime,
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
            init_idents,
            args,
        }
    }

    fn split_for_declaration(&self, ref_mut: bool) -> (&Generics, Option<&syn::WhereClause>) {
        let the_generics = if ref_mut {
            &self.ref_mut_generics
        } else {
            &self.generics
        };
        (the_generics, the_generics.where_clause.as_ref())
    }

    fn enum_struct(&self) -> ItemStruct {
        Paths!(prelude, debug, default, clone, copy);
        UnsizedEnumContext!(self => enum_ident, generics, vis);

        let wc = &generics.where_clause;
        let phantom_generics_type = phantom_generics_type(generics);

        let phantom_generics: Option<TokenStream> = phantom_generics_type.map(|ty| quote!((#ty)));

        parse_quote! {
            #[repr(C)]
            #[derive(#prelude::Align1)]
            #[#prelude::derivative(#debug, #default, #copy, #clone)]
            #vis struct #enum_ident #generics #phantom_generics #wc;
        }
    }

    fn discriminant_enum(&self) -> ItemEnum {
        Paths!(debug, copy, clone, eq, partial_eq, bytemuck);
        UnsizedEnumContext!(self => vis, discriminant_ident, variant_idents, repr, discriminant_values);

        parse_quote! {
            #[derive(#copy, #clone, #debug, #eq, #partial_eq, Hash, Ord, PartialOrd, #bytemuck::NoUninit)]
            #repr
            #vis enum #discriminant_ident {
                #(#variant_idents #discriminant_values,)*
            }
        }
    }

    fn owned_enum(&self) -> ItemEnum {
        Paths!(prelude, debug);
        UnsizedEnumContext!(self => owned_ident, variant_idents, variant_types, variant_docs, repr, args, discriminant_values, generics);
        let additional_owned = args.owned_attributes.attributes.iter();
        let wc = &generics.where_clause;

        parse_quote! {
            #[derive(#debug)]
            #(#[#additional_owned])*
            #repr
            pub enum #owned_ident #generics #wc {
                #(
                    #(#variant_docs)*
                    #variant_idents(<#variant_types as #prelude::UnsizedType>::Owned) #discriminant_values,
                )*
            }
        }
    }

    fn ref_enum(&self) -> ItemEnum {
        Paths!(prelude, debug, copy, clone);
        UnsizedEnumContext!(self => ref_ident, variant_idents, variant_types, variant_docs, rm_lt, discriminant_values, repr);
        let (generics, wc) = self.split_for_declaration(true);
        parse_quote! {
            #[derive(#debug, #copy, #clone)]
            #repr
            pub enum #ref_ident #generics #wc {
                #(
                    #(#variant_docs)*
                    #variant_idents(<#variant_types as #prelude::UnsizedType>::Ref<#rm_lt>) #discriminant_values,
                )*
            }
        }
    }

    fn mut_enum(&self) -> ItemEnum {
        Paths!(prelude, debug);
        UnsizedEnumContext!(self => mut_ident, variant_idents, variant_types, variant_docs, rm_lt, repr, discriminant_values);
        let (generics, wc) = self.split_for_declaration(true);
        parse_quote! {
            #[derive(#debug)]
            #repr
            pub enum #mut_ident #generics #wc {
                #(
                    #(#variant_docs)*
                    #variant_idents(<#variant_types as #prelude::UnsizedType>::Mut<#rm_lt>) #discriminant_values,
                )*
            }
        }
    }

    fn as_shared_impl(&self) -> TokenStream {
        Paths!(prelude);
        UnsizedEnumContext!(self => rm_lt, mut_type, ref_ident, variant_types, mut_ident, variant_idents);
        let as_shared_lt = new_lifetime(&self.generics, Some("as_shared"));
        let shared_lt = new_lifetime(&self.generics, Some("shared"));

        let as_shared_generics =
            self.ref_mut_generics
                .combine::<BetterGenerics>(&parse_quote!([<#as_shared_lt> where
                    #rm_lt: #as_shared_lt,
                    #(<#variant_types as #prelude::UnsizedType>::Mut<#rm_lt>:
                        #prelude::AsShared<#as_shared_lt, Shared<#as_shared_lt> = <#variant_types as #prelude::UnsizedType>::Ref<#as_shared_lt>>
                    ),*
                ]));
        let (impl_gen, _, wc) = as_shared_generics.split_for_impl();

        let ref_gen = self
            .generics
            .combine::<BetterGenerics>(&parse_quote!([<#shared_lt>]));
        let ref_ty_gen = ref_gen.split_for_impl().1;

        quote! {
            #[automatically_derived]
            impl #impl_gen #prelude::AsShared<#as_shared_lt> for #mut_type #wc {
                type Shared<#shared_lt> = #ref_ident #ref_ty_gen where Self: #shared_lt;
                fn as_shared(&#as_shared_lt self) -> Self::Shared<#as_shared_lt> {
                    match self {
                        #(
                            #mut_ident::#variant_idents(inner) => {
                                #ref_ident::#variant_idents(
                                    <<#variant_types as #prelude::UnsizedType>::Mut<#rm_lt> as #prelude::AsShared>::as_shared(inner)
                                )
                            }
                        )*
                    }
                }
            }
        }
    }

    fn unsized_type_impl(&self) -> TokenStream {
        Paths!(prelude, result);
        UnsizedEnumContext!(self => ref_type, rm_lt, owned_type,
            ref_ident, mut_type, mut_ident, enum_type, variant_types, integer_repr,
            discriminant_ident, variant_idents
        );
        let (impl_gen, _, where_clause) = self.generics.split_for_impl();
        let discriminant_consts = self
            .variant_idents
            .iter()
            .map(|var_ident| format_ident!("{}", var_ident.to_string().to_shouty_snake_case()))
            .collect_vec();

        quote! {
            #[automatically_derived]
            unsafe impl #impl_gen #prelude::UnsizedType for #enum_type #where_clause {
                type Ref<#rm_lt> = #ref_type;
                type Mut<#rm_lt> = #prelude::StartPointer<#mut_type>;
                type Owned = #owned_type;

                const ZST_STATUS: bool = {
                    true #(&& <#variant_types as #prelude::UnsizedType>::ZST_STATUS)*
                };

                fn get_ref<#rm_lt>(data: &mut &#rm_lt [u8]) -> #result<Self::Ref<#rm_lt>> {
                    #(const #discriminant_consts: #integer_repr = #discriminant_ident::#variant_idents as #integer_repr;)*
                    let repr: #integer_repr = <#integer_repr>::from_le_bytes(*#prelude::AdvanceArray::try_advance_array(data)?);
                    match repr {
                        #(
                            #discriminant_consts =>
                                Ok(#ref_ident::#variant_idents(<#variant_types as #prelude::UnsizedType>::get_ref(data)?)),
                        )*
                        _ => #prelude::bail!("Invalid enum discriminant"),
                    }
                }

                fn get_mut<#rm_lt>(data: &mut &#rm_lt mut [u8]) -> #result<Self::Mut<#rm_lt>> {
                    #(const #discriminant_consts: #integer_repr = #discriminant_ident::#variant_idents as #integer_repr;)*
                    let start_ptr = data.as_mut_ptr().cast_const().cast::<()>();
                    let repr: #integer_repr = <#integer_repr>::from_le_bytes(*#prelude::AdvanceArray::try_advance_array(data)?);
                    let res = match repr {
                        #(
                            #discriminant_consts =>
                                #mut_ident::#variant_idents(<#variant_types as #prelude::UnsizedType>::get_mut(data)?),
                        )*
                        _ => #prelude::bail!("Invalid enum discriminant"),
                    };
                    Ok(unsafe { #prelude::StartPointer::new(start_ptr, res) })
                }

                fn owned_from_ref(r: Self::Ref<'_>) -> #result<Self::Owned> {
                    match r {
                        #(
                            #ref_ident::#variant_idents(inner) => Ok(#owned_type::#variant_idents(
                                <#variant_types as #prelude::UnsizedType>::owned_from_ref(inner)?,
                            )),
                        )*
                    }
                }

                unsafe fn resize_notification(self_mut: &mut Self::Mut<'_>, source_ptr: *const (), change: isize) -> #result<()> {
                    unsafe { Self::Mut::handle_resize_notification(self_mut, source_ptr, change) };
                    match &mut self_mut.data {
                        #(
                            #mut_ident::#variant_idents(inner) => unsafe {
                                <#variant_types as #prelude::UnsizedType>::resize_notification(inner, source_ptr, change)
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
        let variant_type = &self.variant_types[default_init.index];
        let variant_ident = &self.variant_idents[default_init.index];

        let unsized_init = quote!(#prelude::UnsizedInit<#prelude::DefaultInit>);
        let default_init_generics = self
            .generics
            .combine::<BetterGenerics>(&parse_quote!([where #variant_type: #unsized_init]));
        let (default_init_impl, _, default_init_where) = default_init_generics.split_for_impl();
        quote! {
            #[allow(trivial_bounds)]
            #[automatically_derived]
            impl #default_init_impl #unsized_init for #enum_type #default_init_where {
                const INIT_BYTES: usize = <#variant_type as #unsized_init>::INIT_BYTES + #size_of::<#discriminant_ident>();

                unsafe fn init(
                    bytes: &mut &mut [u8],
                    arg: #prelude::DefaultInit,
                ) -> #result<()> {
                    #prelude::Advance::try_advance(bytes, #size_of::<#discriminant_ident>())?
                        .copy_from_slice(#bytemuck::bytes_of(&(#discriminant_ident::#variant_ident as #integer_repr)));
                    unsafe { <#variant_type as #unsized_init>::init(bytes, arg) }
                }
            }
        }
    }

    fn unsized_init_struct_impl(&self) -> TokenStream {
        Paths!(prelude, result, size_of, bytemuck, copy, clone, debug, default);
        UnsizedEnumContext!(self => enum_type, discriminant_ident, integer_repr, init_idents, vis, variant_types, variant_idents);
        let init_generic = new_generic(&self.generics, Some("Init"));

        let init_generic_trait = quote!(#prelude::UnsizedInit<#init_generic>);
        let all_generics = self
            .variant_types
            .iter()
            .map(|variant_ty| {
                self.generics.combine::<BetterGenerics>(&parse_quote!([
                    <#init_generic> where #variant_ty: #init_generic_trait
                ]))
            })
            .collect_vec();
        let base_generics = &all_generics[0];
        let impl_gen = base_generics.split_for_impl().0;
        let where_clauses = all_generics
            .iter()
            .map(|gen| gen.split_for_impl().2)
            .collect_vec();

        quote! {
            #(
                #[derive(#copy, #clone, #debug, #default)]
                #vis struct #init_idents<#init_generic>(#vis #init_generic);

                #[allow(trivial_bounds)]
                #[automatically_derived]
                impl #impl_gen #prelude::UnsizedInit<#init_idents<#init_generic>> for #enum_type #where_clauses {
                    const INIT_BYTES: usize = <#variant_types as #init_generic_trait>::INIT_BYTES + #size_of::<#discriminant_ident>();

                    unsafe fn init(
                        bytes: &mut &mut [u8],
                        arg: #init_idents<#init_generic>,
                    ) -> #result<()> {
                        #prelude::Advance::try_advance(bytes, #size_of::<#discriminant_ident>())?
                            .copy_from_slice(#bytemuck::bytes_of(&(#discriminant_ident::#variant_idents as #integer_repr)));
                        unsafe { <#variant_types as #init_generic_trait>::init(bytes, arg.0) }
                    }
                }
            )*
        }
    }

    #[allow(non_snake_case)]
    fn extension_impl(&self) -> TokenStream {
        Paths!(prelude, debug, result);
        UnsizedEnumContext!(self => vis, enum_ident, repr, variant_idents, variant_types, rm_lt, mut_ident, init_idents, discriminant_values);

        // Create new lifetimes and generics for the extension trait
        let info = new_lifetime(&self.generics, Some("info"));
        let b = new_lifetime(&self.generics, Some("b"));
        let c = new_lifetime(&self.generics, Some("c"));

        let O = new_generic(&self.generics, Some("O"));
        let Init = new_generic(&self.generics, Some("Init"));
        let A = new_generic(&self.generics, Some("A"));

        let wc =
            quote!(#O: #prelude::UnsizedType + ?Sized, #A: #prelude::UnsizedTypeDataAccess<#info>);
        // Combine generics for the extension trait
        let ext_trait_generics = self.generics.combine::<BetterGenerics>(&parse_quote!([
            <#b, #rm_lt, #info, #O, #A> where #wc
        ]));

        // Combine generics for the exclusive enum
        let ext_enum_return_generics = self.generics.combine::<BetterGenerics>(&parse_quote!([
            <#c, #rm_lt, #info, #O, #A> where #wc
        ]));

        let return_ty_gen = ext_enum_return_generics.split_for_impl().1;

        let (impl_gen, ty_gen, wc) = ext_trait_generics.split_for_impl();

        // Create identifiers for the extension trait and exclusive enum
        let extension_ident = format_ident!("{enum_ident}ExclusiveExt");
        let exclusive_ident = format_ident!("{enum_ident}Exclusive");

        // Type for the ExclusiveWrapperT that will implement the extension trait
        let impl_for = quote!(#prelude::ExclusiveWrapperT<#b, #rm_lt, #info, #enum_ident, #O, #A>);

        let setter_methods = self
            .variant_idents
            .iter()
            .map(|var_ident| format_ident!("set_{}", var_ident.to_string().to_snake_case()))
            .collect_vec();

        // RustRover highlights spans used in unsafe operations (and generics of those unsafe operations),
        // so this is a workaround to avoid that.
        let enum_ident_stripped_span = format_ident!("{enum_ident}", span = Span::call_site());

        // Generate the extension trait
        let extension_trait = quote! {
            #vis trait #extension_ident #impl_gen #wc
            {
                fn get<#c>(&#c mut self) -> #exclusive_ident #return_ty_gen;

                #(
                    fn #setter_methods<#Init>(&mut self, init: #Init) -> #result<()>
                    where
                        #enum_ident: #prelude::UnsizedInit<#init_idents<#Init>>;
                )*
            }
        };

        // Generate the implementation of the extension trait
        let extension_impl = quote! {
            #[automatically_derived]
            impl #impl_gen #extension_ident #ty_gen for #impl_for #wc
            {
                fn get<#c>(&#c mut self) -> #exclusive_ident #return_ty_gen {
                    match &***self {
                        #(
                            #mut_ident::#variant_idents(_) => {
                                #exclusive_ident::#variant_idents(unsafe {
                                    #prelude::ExclusiveWrapper::map_ref(self, |inner| {
                                        match &mut **inner {
                                            #mut_ident::#variant_idents(inner) => inner,
                                            _ => unreachable!(),
                                        }
                                    })
                                })
                            }
                        )*
                    }
                }

                #(
                    fn #setter_methods<Init>(&mut self, init: #Init) -> #result<()>
                    where
                        #enum_ident: #prelude::UnsizedInit<#init_idents<#Init>>,
                    {
                        unsafe {
                            #prelude::ExclusiveWrapper::set_start_pointer_data::<#enum_ident_stripped_span, _>(
                                self,
                                #init_idents(init),
                            )
                        }
                    }
                )*
            }
        };

        // Generate the exclusive enum
        let exclusive_enum = quote! {
            #[derive(#debug)]
            #repr
            #vis enum #exclusive_ident #impl_gen #wc
            {
                #(
                    #variant_idents(#prelude::ExclusiveWrapperT<#b, #rm_lt, #info, #variant_types, #O, #A>) #discriminant_values,
                )*
            }
        };

        quote! {
            #extension_trait
            #extension_impl
            #exclusive_enum
        }
    }

    fn idl_impl(&self) -> TokenStream {
        account_impl(&self.item_enum.clone().into(), &self.args)
    }
}
