use crate::unsize::{account, UnsizedTypeArgs};
use crate::util::{
    generate_fields_are_trait, get_field_idents, get_field_types, get_field_vis, new_generics,
    new_ident, new_lifetime, phantom_generics_ident, phantom_generics_type, reject_attributes,
    restrict_attributes, strip_inner_attributes, BetterGenerics, CombineGenerics, Paths,
};
use heck::ToUpperCamelCase;
use itertools::Itertools;
use proc_macro2::Ident;
use proc_macro2::TokenStream;
use proc_macro_error2::abort;
use quote::{format_ident, quote};
use syn::{parse_quote, Field, Generics, ItemStruct, Lifetime, Type, Visibility};

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
    let context = UnsizedStructContext::parse(item_struct, unsized_args);

    // println!("After context!");
    let main_struct = context.main_struct();
    // println!("After inner_struct!");
    let ref_struct = context.ref_struct();
    // println!("After ref_struct!");
    let mut_struct = context.mut_struct();
    // println!("After mut_struct!");
    let owned_struct = context.owned_struct();
    // println!("After owned_struct!");
    let sized_struct = context.sized_struct();
    // println!("After sized_struct!");
    let sized_bytemuck_derives = context.sized_bytemuck_derives();
    // println!("After sized_bytemuck_derives!");
    let ref_mut_derefs = context.ref_mut_derefs();
    // println!("After ref_mut_derefs!");
    let as_shared_impl = context.as_shared_impl();
    // println!("After as_shared_impl!");
    let unsized_type_impl = context.unsized_type_impl();
    // println!("After unsized_type_impl!");
    let default_init_impl = context.unsized_init_default_impl();
    // // println!("After unsized_init_impl!");
    let init_struct_impl = context.unsized_init_struct_impl();
    // // println!("After init_struct_impl!");
    let extension_impl = context.extension_impl();
    // // println!("After extension_impl!");
    let account_impl = account::account_impl(&context.account_item_struct.into(), &context.args);
    // println!("After account_impl!");

    quote! {
        #main_struct
        #ref_struct
        #mut_struct
        #owned_struct
        #sized_struct
        #sized_bytemuck_derives
        #ref_mut_derefs
        #as_shared_impl
        #unsized_type_impl
        #default_init_impl
        #init_struct_impl
        #extension_impl
        #account_impl
    }
}

#[derive(Clone)]
pub struct UnsizedStructContext {
    vis: Visibility,
    struct_ident: Ident,
    struct_type: Type,
    rm_lt: Lifetime,
    ref_mut_generics: Generics,
    generics: Generics,
    mut_ident: Ident,
    mut_type: Type,
    ref_ident: Ident,
    ref_type: Type,
    owned_ident: Ident,
    owned_type: Type,
    sized_ident: Option<Ident>,
    sized_type: Option<Type>,
    sized_field_ident: Option<Ident>,
    phantom_generic_ident: Option<Ident>,
    phantom_generic_type: Option<Type>,
    account_item_struct: ItemStruct,
    sized_fields: Vec<Field>,
    unsized_fields: Vec<Field>,
    owned_fields: Vec<Field>,
    sized_field_idents: Vec<Ident>,
    sized_field_types: Vec<Type>,
    unsized_field_idents: Vec<Ident>,
    unsized_field_types: Vec<Type>,
    with_sized_idents: Vec<Ident>,
    with_sized_types: Vec<Type>,
    with_sized_vis: Vec<Visibility>,
    with_sized_vis_pub: Vec<Visibility>,
    args: UnsizedTypeArgs,
}

impl UnsizedStructContext {
    fn parse(mut item_struct: ItemStruct, args: UnsizedTypeArgs) -> Self {
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

        if unsized_start.is_empty() {
            abort!(item_struct, "No `unsized_start` attribute found");
        }
        restrict_attributes(&item_struct, &["unsized_start", "type_to_idl", "doc"]);
        if unsized_start.len() > 1 {
            abort!(
                unsized_start[1].attribute,
                "`unsized_start` can only start once!"
            );
        }

        if matches!(item_struct.fields, syn::Fields::Unnamed(_)) {
            abort!(item_struct.fields, "Unnamed fields are not supported")
        }

        let vis = item_struct.vis.clone();
        let first_unsized = unsized_start[0].index;
        let all_fields = item_struct.fields.iter().cloned().collect::<Vec<_>>();
        let (sized_fields, unsized_fields) = all_fields.split_at(first_unsized);
        let sized_fields = sized_fields.to_vec();
        let unsized_fields = unsized_fields.to_vec();
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
        let mut phantom_generic_ident = None;
        let mut phantom_generic_type = None;
        if !args.skip_phantom_generics {
            if let Some(generic_ty) = phantom_generics_type(&item_struct) {
                phantom_generic_ident.replace(phantom_generics_ident());
                phantom_generic_type.replace(generic_ty);
            }
        }

        let ref_mut_lifetime = new_lifetime(&item_struct.generics, None);
        let ref_mut_generics = item_struct
            .generics
            .combine::<BetterGenerics>(&parse_quote!([<#ref_mut_lifetime>]));

        let ref_mut_type_gen = ref_mut_generics.split_for_impl().1;
        let type_generics = &item_struct.generics.split_for_impl().1;
        let struct_ident = item_struct.ident.clone();
        let struct_type = parse_quote!(#struct_ident #type_generics);
        let mut_ident = format_ident!("{struct_ident}Mut");
        let mut_type = parse_quote!(#mut_ident #ref_mut_type_gen);
        let ref_ident = format_ident!("{struct_ident}Ref");
        let ref_type = parse_quote!(#ref_ident #ref_mut_type_gen);
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

        let sized_field_ident = sized_ident
            .as_ref()
            .map(|_| new_ident("sized", get_field_idents(&all_fields)));

        let sized_field_idents = get_field_idents(&sized_fields).cloned().collect_vec();
        let sized_field_types = get_field_types(&sized_fields).cloned().collect_vec();
        let unsized_field_idents = get_field_idents(&unsized_fields).cloned().collect_vec();
        let unsized_field_types = get_field_types(&unsized_fields).cloned().collect_vec();
        let unsized_field_vis = get_field_vis(&unsized_fields).cloned().collect_vec();

        let with_sized_idents = sized_field_ident
            .iter()
            .chain(unsized_field_idents.iter())
            .cloned()
            .collect_vec();
        let with_sized_types = sized_type
            .iter()
            .chain(unsized_field_types.iter())
            .cloned()
            .collect_vec();
        let mut with_sized_vis_pub = unsized_field_vis.clone();
        let mut with_sized_vis = unsized_field_vis.clone();
        sized_field_ident.is_some().then(|| {
            with_sized_vis_pub.insert(0, parse_quote!(pub));
            with_sized_vis.insert(0, Visibility::Inherited);
        });

        Self {
            generics: item_struct.generics,
            vis,
            struct_ident,
            rm_lt: ref_mut_lifetime,
            ref_mut_generics,
            struct_type,
            mut_ident,
            mut_type,
            ref_ident,
            ref_type,
            owned_ident,
            owned_type,
            sized_field_ident,
            sized_ident,
            sized_type,
            phantom_generic_ident,
            phantom_generic_type,
            account_item_struct,
            sized_fields,
            unsized_fields,
            owned_fields,
            sized_field_idents,
            sized_field_types,
            unsized_field_idents,
            unsized_field_types,
            with_sized_idents,
            with_sized_types,
            with_sized_vis,
            with_sized_vis_pub,
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

    fn main_struct(&self) -> ItemStruct {
        Paths!(prelude, debug);
        UnsizedStructContext!(self => vis, struct_ident);

        let (generics, wc) = self.split_for_declaration(false);
        let phantom_ty = phantom_generics_type(generics).map(|ty| quote!((#ty)));
        parse_quote! {
            #[#prelude::derivative(#debug)]
            #[derive(#prelude::Align1)]
            #vis struct #struct_ident #generics #phantom_ty #wc;
        }
    }

    fn ref_struct(&self) -> ItemStruct {
        Paths!(prelude, copy, clone, debug);
        UnsizedStructContext!(self => vis, ref_ident, with_sized_vis, with_sized_idents, with_sized_types, rm_lt);
        let (generics, wc) = self.split_for_declaration(true);
        let transparent = (with_sized_idents.len() == 1).then(|| quote!(#[repr(transparent)]));
        parse_quote! {
            #[#prelude::derivative(#debug, #copy, #clone)]
            #transparent
            #vis struct #ref_ident #generics #wc {
                #(
                    #with_sized_vis #with_sized_idents: <#with_sized_types as #prelude::UnsizedType>::Ref<#rm_lt>,
                )*
            }
        }
    }

    fn mut_struct(&self) -> ItemStruct {
        Paths!(prelude, debug);
        UnsizedStructContext!(self => vis, mut_ident, rm_lt, with_sized_vis, with_sized_idents, with_sized_types);
        let (generics, wc) = self.split_for_declaration(true);
        let transparent = (with_sized_idents.len() == 1).then(|| quote!(#[repr(transparent)]));

        parse_quote! {
            #[#prelude::derivative(#debug)]
            #transparent
            #vis struct #mut_ident #generics #wc {
                #(
                    #with_sized_vis #with_sized_idents: <#with_sized_types as #prelude::UnsizedType>::Mut<#rm_lt>,
                )*
            }
        }
    }

    fn owned_struct(&self) -> ItemStruct {
        Paths!(prelude, debug);
        UnsizedStructContext!(self => vis, owned_ident, owned_fields);
        let additional_attributes = self.args.owned_attributes.attributes.iter();

        let (gen, where_clause) = self.split_for_declaration(false);

        parse_quote! {
            #(#[#additional_attributes])*
            #[#prelude::derivative(#debug)]
            #vis struct #owned_ident #gen #where_clause {
                #(#owned_fields,)*
            }
        }
    }

    fn sized_struct(&self) -> Option<ItemStruct> {
        Paths!(prelude, debug, bytemuck, copy, clone, partial_eq, eq);
        UnsizedStructContext!(self => vis, sized_ident, sized_fields, phantom_generic_ident, phantom_generic_type);
        some_or_return!(sized_ident);
        let additional_attributes = self.args.sized_attributes.attributes.iter();

        let sized_bytemuck_derives = self.generics.params.is_empty().then(
            || quote!(#bytemuck::CheckedBitPattern, #bytemuck::NoUninit, #bytemuck::Zeroable),
        );
        let (impl_gen, where_clause) = self.split_for_declaration(false);
        let phantom_field = phantom_generic_ident
            .as_ref()
            .map(|ident| quote!(#vis #ident: #phantom_generic_type,));
        let sized_struct: ItemStruct = parse_quote! {
            #(#[#additional_attributes])*
            #[#prelude::derivative(#copy, #clone, #debug, #partial_eq, #eq)]
            #[derive(#prelude::Align1, #sized_bytemuck_derives)]
            #[repr(C, packed)]
            #vis struct #sized_ident #impl_gen #where_clause {
                #(#sized_fields,)*
                #phantom_field
            }
        };
        Some(sized_struct)
    }

    fn sized_bytemuck_derives(&self) -> Option<TokenStream> {
        Paths!(prelude, bytemuck, debug, copy, clone);
        UnsizedStructContext!(self => vis, sized_fields, sized_ident, sized_field_idents, sized_field_types, phantom_generic_type, phantom_generic_ident);
        some_or_return!(sized_ident);
        if self.generics.params.is_empty() {
            return None;
        }
        let sized_field_idents = phantom_generic_ident
            .iter()
            .chain(sized_field_idents)
            .collect_vec();
        let sized_field_types = phantom_generic_type
            .iter()
            .chain(sized_field_types)
            .collect_vec();

        let (impl_generics, type_generics, where_clause) = self.generics.split_for_impl();
        let struct_generics = &self.generics;

        let bit_ident = format_ident!("{}Bits", sized_ident);
        let bit_field_types = sized_field_types
            .iter()
            .map::<Type, _>(|ty| parse_quote!(<#ty as #bytemuck::CheckedBitPattern>::Bits))
            .collect_vec();

        let validate_fields_are_trait = generate_fields_are_trait(
            sized_fields,
            &self.generics,
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

            #[#prelude::derivative(#debug, #copy, #clone)]
            #[repr(C, packed)]
            #vis struct #bit_ident #struct_generics #where_clause {
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

    fn ref_mut_derefs(&self) -> Option<TokenStream> {
        Paths!(deref, deref_mut);
        UnsizedStructContext!(self => sized_type, sized_field_ident, ref_type, mut_type);
        some_or_return!(sized_type);

        let (impl_gen, _, wc) = self.ref_mut_generics.split_for_impl();

        Some(quote! {
            impl #impl_gen #deref for #ref_type #wc {
                type Target = #sized_type;

                fn deref(&self) -> &Self::Target {
                    &self.#sized_field_ident
                }
            }

            impl #impl_gen #deref for #mut_type #wc {
                type Target = #sized_type;

                fn deref(&self) -> &Self::Target {
                    &self.#sized_field_ident
                }
            }

            impl #impl_gen #deref_mut for #mut_type #wc {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.#sized_field_ident
                }
            }
        })
    }

    fn as_shared_impl(&self) -> TokenStream {
        Paths!(prelude);
        UnsizedStructContext!(self => rm_lt, with_sized_types, with_sized_idents, mut_type, ref_ident, unsized_field_types);
        let as_shared_lt = new_lifetime(&self.generics, Some("as_shared"));
        let shared_lt = new_lifetime(&self.generics, Some("shared"));

        let as_shared_generics = self.ref_mut_generics.combine::<BetterGenerics>(
            &parse_quote!([<#as_shared_lt> where
                #rm_lt: #as_shared_lt,
                #(<#unsized_field_types as #prelude::UnsizedType>::Mut<#rm_lt>:
                    #prelude::AsShared<#as_shared_lt, Shared<#as_shared_lt> = <#unsized_field_types as #prelude::UnsizedType>::Ref<#as_shared_lt>>
                ),*
            ]),
        );
        let (impl_gen, _, wc) = as_shared_generics.split_for_impl();

        let ref_gen = self
            .generics
            .combine::<BetterGenerics>(&parse_quote!([<#shared_lt>]));
        let ref_ty_gen = ref_gen.split_for_impl().1;

        quote! {
            impl #impl_gen #prelude::AsShared<#as_shared_lt> for #mut_type #wc {
                type Shared<#shared_lt> = #ref_ident #ref_ty_gen where Self: #shared_lt;
                fn as_shared(&#as_shared_lt self) -> Self::Shared<#as_shared_lt> {
                    #ref_ident {
                        #(
                            #with_sized_idents: <<#with_sized_types as #prelude::UnsizedType>::Mut<#rm_lt> as
                                #prelude::AsShared>::as_shared(&self.#with_sized_idents),
                        )*
                    }
                }
            }
        }
    }

    fn unsized_type_impl(&self) -> TokenStream {
        Paths!(prelude, result);
        UnsizedStructContext!(self => ref_type, sized_field_idents, struct_type, rm_lt, owned_type,
            with_sized_types, ref_ident, with_sized_idents, sized_type, mut_type, mut_ident,
            unsized_field_idents, unsized_field_types
        );
        let (impl_gen, _, where_clause) = self.generics.split_for_impl();

        let sized_resize = sized_type.as_ref().map(|sized_type| {
            quote! {
                unsafe { <#sized_type as #prelude::UnsizedType>::resize_notification(r, operation) }?;
            }
        });

        quote! {
            unsafe impl #impl_gen #prelude::UnsizedType for #struct_type #where_clause {
                type Ref<#rm_lt> = #ref_type;
                type Mut<#rm_lt> = #mut_type;
                type Owned = #owned_type;

                fn get_ref<#rm_lt>(data: &mut &#rm_lt [u8]) -> #result<Self::Ref<#rm_lt>> {
                    Ok(#ref_ident {
                        #(#with_sized_idents: <#with_sized_types as #prelude::UnsizedType>::get_ref(data)?,)*
                    })
                }

                fn get_mut<#rm_lt>(data: &mut &#rm_lt mut [u8]) -> #result<Self::Mut<#rm_lt>> {
                    Ok(#mut_ident {
                        #(#with_sized_idents: <#with_sized_types as #prelude::UnsizedType>::get_mut(data)?,)*
                    })
                }

                fn owned_from_ref(r: Self::Ref<'_>) -> #result<Self::Owned> {
                    Ok(Self::Owned {
                        #(#sized_field_idents: r.#sized_field_idents,)*
                        #(#unsized_field_idents: <#unsized_field_types as #prelude::UnsizedType>::owned_from_ref(r.#unsized_field_idents)?,)*
                    })
                }

                unsafe fn resize_notification(r: &mut &mut [u8], operation: #prelude::ResizeOperation) -> #result<()> {
                    #sized_resize
                    #prelude::__resize_notification_checked! {
                        r, operation -> #(#unsized_field_types),*
                    }
                }
            }
        }
    }

    fn unsized_init_default_impl(&self) -> TokenStream {
        Paths!(prelude, result);
        UnsizedStructContext!(self => struct_type, with_sized_types);
        let default_init_generics = self.generics.combine::<BetterGenerics>(
            &parse_quote!([where #(#with_sized_types: #prelude::UnsizedInit<#prelude::DefaultInit>),*]),
        );
        let (default_init_impl, _, default_init_where) = default_init_generics.split_for_impl();
        let unsized_init = quote!(#prelude::UnsizedInit<#prelude::DefaultInit>);
        quote! {
            #[allow(trivial_bounds)]
            #[automatically_derived]
            impl #default_init_impl #unsized_init for #struct_type #default_init_where {
                const INIT_BYTES: usize = #(<#with_sized_types as #unsized_init>::INIT_BYTES)+*;

                unsafe fn init(
                    bytes: &mut &mut [u8],
                    arg: #prelude::DefaultInit,
                ) -> #result<()> {
                    #(
                        unsafe { <#with_sized_types as #unsized_init>::init(bytes, arg) }?;
                    )*
                    Ok(())
                }
            }
        }
    }

    fn unsized_init_struct_impl(&self) -> TokenStream {
        Paths!(prelude, result, copy, clone, debug);
        UnsizedStructContext!(self => vis, with_sized_idents, with_sized_types, with_sized_vis_pub
            struct_type, struct_ident);

        let init_struct_ident = format_ident!("{struct_ident}Init");

        let init_generic_idents = self
            .with_sized_idents
            .iter()
            .map(|ident| format_ident!("{}Init", ident.to_string().to_upper_camel_case()))
            .collect_vec();

        let init_generics: BetterGenerics = parse_quote!([
            <#(#init_generic_idents),*> where
                #(#with_sized_types: #prelude::UnsizedInit<#init_generic_idents>),*
        ]);

        let combined_init_generics = self.generics.combine(&init_generics);

        let (init_impl_impl_generics, _, init_impl_where_clause) =
            combined_init_generics.split_for_impl();

        let struct_type_generics = init_generics.split_for_impl().1;

        let init_struct_type = quote!(#init_struct_ident #struct_type_generics);

        quote! {
            #[derive(#copy, #clone, #debug)]
            #vis struct #init_struct_ident #init_generics {
                #(#with_sized_vis_pub #with_sized_idents: #init_generic_idents,)*
            }

            #[allow(trivial_bounds)]
            #[automatically_derived]
            impl #init_impl_impl_generics #prelude::UnsizedInit<#init_struct_type> for #struct_type #init_impl_where_clause {
                const INIT_BYTES: usize = #(<#with_sized_types as #prelude::UnsizedInit<#init_generic_idents>>::INIT_BYTES)+*;

                unsafe fn init(
                    bytes: &mut &mut [u8],
                    arg: #init_struct_type,
                ) -> #result<()> {
                    #(
                        unsafe { <#with_sized_types as #prelude::UnsizedInit<#init_generic_idents>>::init(bytes, arg.#with_sized_idents) }?;
                    )*
                    Ok(())
                }
            }
        }
    }

    fn extension_impl(&self) -> TokenStream {
        Paths!(prelude);
        UnsizedStructContext!(self => struct_ident, unsized_fields, rm_lt, struct_type);
        let info = new_lifetime(&self.generics, Some("info"));
        let [o, a] = new_generics(&self.generics);
        let ext_trait_generics = self
            .ref_mut_generics
            .combine::<BetterGenerics>(&parse_quote!([
                <#info, #o, #a> where
                    #o: #prelude::UnsizedType,
                    #a: #prelude::UnsizedTypeDataAccess<#info>
            ]));

        let (impl_gen, ty_gen, wc) = ext_trait_generics.split_for_impl();
        let pub_extension_ident = format_ident!("{}ExclusivePub", struct_ident);
        let priv_extension_ident = format_ident!("{}Exclusive", struct_ident);

        let (pub_unsized_fields, priv_unsized_fields): (Vec<_>, Vec<_>) =
            unsized_fields.iter().partition(|field| match field.vis {
                Visibility::Public(_) => true,
                Visibility::Inherited => false,
                Visibility::Restricted(_) => {
                    abort!(field.vis, "Unsized fields must be `pub` or private")
                }
            });

        let impl_for = quote!(#prelude::ExclusiveWrapper<#rm_lt, #info, <#struct_type as #prelude::UnsizedType>::Mut<#rm_lt>, #o, #a>);

        let make_ext_trait = |vis: &Visibility, fields: Vec<&Field>, extension_ident: &Ident| {
            let field_idents = get_field_idents(&fields).collect_vec();
            let field_types = get_field_types(&fields).collect_vec();
            let field_fn_idents = field_idents
                .iter()
                .map(|ident| new_ident(&format!("{ident}_exclusive"), field_idents.iter().copied()))
                .collect_vec();
            quote! {
                #vis trait #extension_ident #impl_gen #wc
                {
                    #(
                        fn #field_fn_idents(self) -> #prelude::ExclusiveWrapper<#rm_lt, #info, <#field_types as #prelude::UnsizedType>::Mut<#rm_lt>, #o, #a>;
                    )*
                }

                impl #impl_gen #extension_ident #ty_gen for #impl_for #wc {
                    #(
                        fn #field_fn_idents(self) -> #prelude::ExclusiveWrapper<#rm_lt, #info, <#field_types as #prelude::UnsizedType>::Mut<#rm_lt>, #o, #a> {
                            unsafe { #prelude::ExclusiveWrapper::map(self, |x| x.#field_idents) }
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
            #pub_trait
            #priv_trait
        }
    }
}
