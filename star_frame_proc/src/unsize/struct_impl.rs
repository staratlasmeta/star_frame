use crate::unsize::{account, UnsizedTypeArgs};
use crate::util::{
    combine_gen, generate_fields_are_trait, get_doc_attributes, get_field_idents, get_field_types,
    get_field_vis, new_generic, new_ident, new_lifetime, phantom_generics_ident,
    phantom_generics_type, pretty_path, reject_attributes, restrict_attributes,
    strip_inner_attributes, BetterGenerics, CombineGenerics, Paths,
};
use heck::ToUpperCamelCase;
use itertools::Itertools;
use proc_macro2::Ident;
use proc_macro2::TokenStream;
use proc_macro_error2::abort;
use quote::{format_ident, quote};
use syn::{parse_quote, Attribute, Field, Generics, ItemStruct, Lifetime, Type, Visibility};

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

    let main_struct = context.main_struct();
    // println!("After inner_struct!");
    let ref_struct = context.ref_struct();
    // println!("After ref_struct!");
    let mut_struct = context.mut_struct();
    // println!("After mut_struct!");
    let owned_struct = context
        .args
        .owned_type
        .is_none()
        .then(|| context.owned_struct());
    // println!("After owned_struct!");
    let sized_struct = context.sized_struct();
    // println!("After sized_struct!");
    let sized_additional_derives = context.sized_additional_derives();
    // println!("After sized_additional_derives!");
    let sized_bytemuck_derives = context.sized_bytemuck_derives();
    // println!("After sized_bytemuck_derives!");
    let ref_mut_derefs = context.ref_mut_derefs();
    // println!("After ref_mut_derefs!");
    let as_shared_impl = context.as_shared_impl();
    // println!("After as_shared_impl!");
    let from_owned_impl = context.from_owned_impl();
    // println!("After from_owned_impl!");
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
        #sized_additional_derives
        #sized_bytemuck_derives
        #ref_mut_derefs
        #as_shared_impl
        #from_owned_impl
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
    top_lt: Lifetime,
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
    unstripped_sized_fields: Vec<Field>,
    unsized_fields: Vec<Field>,
    sized_field_idents: Vec<Ident>,
    sized_field_types: Vec<Type>,
    unsized_field_idents: Vec<Ident>,
    _unsized_field_types: Vec<Type>,
    with_sized_docs: Vec<Vec<Attribute>>,
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
        let unstripped_sized_fields = account_item_struct
            .fields
            .iter()
            .take(first_unsized)
            .cloned()
            .collect_vec();
        let unsized_fields = unsized_fields.to_vec();
        let mut phantom_generic_ident = None;
        let mut phantom_generic_type = None;
        if !args.skip_phantom_generics {
            if let Some(generic_ty) = phantom_generics_type(&item_struct) {
                phantom_generic_ident.replace(phantom_generics_ident());
                phantom_generic_type.replace(generic_ty);
            }
        }

        let top_lt = new_lifetime(&item_struct.generics, Some("top"));
        let ref_mut_generics = combine_gen!(item_struct.generics; <#top_lt>);

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
            .map(|_| new_ident("_sized", get_field_idents(&all_fields), true));

        let sized_field_idents = get_field_idents(&sized_fields).cloned().collect_vec();
        let sized_field_types = get_field_types(&sized_fields).cloned().collect_vec();
        let unsized_field_idents = get_field_idents(&unsized_fields).cloned().collect_vec();
        let unsized_field_types = get_field_types(&unsized_fields).cloned().collect_vec();
        let unsized_field_vis = get_field_vis(&unsized_fields).cloned().collect_vec();

        let with_sized_docs = sized_field_ident
            .as_ref()
            .map(|_ident| {
                vec![parse_quote! {
                    #[doc = "Sized portion of the Unsized Type"]
                }]
            })
            .into_iter()
            .chain(
                unsized_fields
                    .iter()
                    .map(|field| get_doc_attributes(&field.attrs)),
            )
            .collect_vec();

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

        let generics = item_struct.generics.clone();

        Self {
            item_struct,
            generics,
            vis,
            struct_ident,
            top_lt,
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
            unstripped_sized_fields,
            sized_fields,
            unsized_fields,
            sized_field_idents,
            sized_field_types,
            unsized_field_idents,
            _unsized_field_types: unsized_field_types,
            with_sized_docs,
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

    fn main_struct(&self) -> TokenStream {
        Paths!(prelude, debug);
        UnsizedStructContext!(self => vis, struct_ident);

        let (generics, wc) = self.split_for_declaration(false);
        let phantom_ty = phantom_generics_type(generics).map(|ty| quote!((#ty)));
        let docs = get_doc_attributes(&self.item_struct.attrs);
        let derives = if phantom_ty.is_some() {
            quote! {
                #[derive(#prelude::DeriveWhere)]
                #[derive_where(Debug)]
            }
        } else {
            quote!(#[derive(#debug)])
        };
        quote! {
            #(#docs)*
            #derives
            #vis struct #struct_ident #generics #phantom_ty #wc;
        }
    }

    fn ref_struct(&self) -> TokenStream {
        Paths!(prelude);
        UnsizedStructContext!(self => vis, ref_ident, with_sized_vis, with_sized_idents, with_sized_types, top_lt, with_sized_docs);
        let (generics, wc) = self.split_for_declaration(true);
        let transparent = (with_sized_idents.len() == 1).then(|| quote!(#[repr(transparent)]));

        let doc = format!("Ref type for [`{}`]", self.struct_ident);
        quote! {
            #[doc = #doc]
            #[derive(#prelude::DeriveWhere)]
            #[derive_where(Debug; #(<#with_sized_types as #prelude::UnsizedType>::Ref<#top_lt>,)*)]
            #transparent
            #vis struct #ref_ident #generics #wc {
                #(
                    #(#with_sized_docs)*
                    #with_sized_vis #with_sized_idents: <#with_sized_types as #prelude::UnsizedType>::Ref<#top_lt>,
                )*
            }
        }
    }

    fn mut_struct(&self) -> TokenStream {
        Paths!(prelude);
        UnsizedStructContext!(self => vis, mut_ident, top_lt, with_sized_vis, with_sized_idents, with_sized_types, with_sized_docs);
        let (generics, wc) = self.split_for_declaration(true);
        let transparent = (with_sized_idents.len() == 1).then(|| quote!(#[repr(transparent)]));

        let doc = format!("Mut type for [`{}`]", self.struct_ident);
        quote! {
            #[doc = #doc]
            #[derive(#prelude::DeriveWhere)]
            #[derive_where(Debug; #(<#with_sized_types as #prelude::UnsizedType>::Mut<#top_lt>,)*)]
            #transparent
            #vis struct #mut_ident #generics #wc {
                #(
                    #(#with_sized_docs)*
                    #with_sized_vis #with_sized_idents: <#with_sized_types as #prelude::UnsizedType>::Mut<#top_lt>,
                )*
            }
        }
    }

    fn owned_struct(&self) -> TokenStream {
        Paths!(prelude, deref, deref_mut);
        UnsizedStructContext!(self => vis, owned_ident, with_sized_idents, with_sized_vis_pub, with_sized_types, sized_field_ident, with_sized_docs);
        let additional_attributes = self.args.owned_attributes.attributes.iter();

        let (gen, where_clause) = self.split_for_declaration(false);

        let owned_types: Vec<Type> = with_sized_types
            .iter()
            .map(|ty| parse_quote!(<#ty as #prelude::UnsizedType>::Owned))
            .collect_vec();

        let lt = new_lifetime(&self.generics, None);

        let deref_impl = self
            .sized_ident
            .as_ref()
            .map(|sized_ident| {
                let (impl_gen, ty_gen, wc) = gen.split_for_impl();
                quote! {
                    impl #impl_gen #deref for #owned_ident #ty_gen #wc {
                        type Target = #sized_ident #ty_gen;
                        fn deref(&self) -> &Self::Target {
                            &self.#sized_field_ident
                        }
                    }

                    impl #impl_gen #deref_mut for #owned_ident #ty_gen #wc {
                        fn deref_mut(&mut self) -> &mut Self::Target {
                            &mut self.#sized_field_ident
                        }
                    }
                }
            })
            .unwrap_or_default();

        // Owned type should have all pub fields for easier client use
        let all_pub_vis = with_sized_vis_pub.iter().map(|_| quote!(pub)).collect_vec();

        let doc = format!("Owned type for [`{}`]", self.struct_ident);
        quote! {
            #(#[#additional_attributes])*
            #[doc = #doc]
            #[derive(#prelude::DeriveWhere)]
            #[derive_where(Debug, Copy, Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd; #(for<#lt> #owned_types,)*)]
            #vis struct #owned_ident #gen #where_clause {
                #(
                    #(#with_sized_docs)*
                    #all_pub_vis #with_sized_idents: #owned_types,
                )*
            }

            #deref_impl
        }
    }

    fn sized_struct(&self) -> Option<TokenStream> {
        Paths!(prelude, bytemuck, type_to_idl_args_ident);
        UnsizedStructContext!(self => vis, sized_ident, unstripped_sized_fields, phantom_generic_ident, phantom_generic_type);
        some_or_return!(sized_ident);
        let additional_attributes = self.args.sized_attributes.attributes.iter();

        let sized_bytemuck_derives = self.generics.params.is_empty().then(
            || quote!(#bytemuck::CheckedBitPattern, #bytemuck::NoUninit, #bytemuck::Zeroable),
        );
        let (impl_gen, where_clause) = self.split_for_declaration(false);
        let phantom_field = phantom_generic_ident.as_ref().map(|ident| {
            quote!(
                #[allow(clippy::pub_underscore_fields)]
                #vis #ident: #phantom_generic_type,
            )
        });
        let sized_attributes = (!self.args.skip_idl).then(|| {
            let program_account = self.args.program.as_ref().map(|program| {
                quote! {
                    #[#type_to_idl_args_ident(program = #program)]
                }
            });
            quote!(
                #[derive(#prelude::TypeToIdl)]
                #program_account
            )
        });
        let doc = format!("Sized portion of [`{}`]", self.struct_ident);
        let sized_struct = quote! {
            #(#[#additional_attributes])*
            #[doc = #doc]
            #[derive(#prelude::Align1, #sized_bytemuck_derives)]
            #sized_attributes
            #[repr(C, packed)]
            #vis struct #sized_ident #impl_gen #where_clause {
                #(#unstripped_sized_fields,)*
                #phantom_field
            }
        };
        Some(sized_struct)
    }

    // TODO: remove once derive_where adds support for packed structs
    fn sized_additional_derives(&self) -> Option<TokenStream> {
        Paths!(debug, copy, clone, partial_eq, eq);
        UnsizedStructContext!(self => sized_ident, sized_field_idents, sized_field_types);
        some_or_return!(sized_ident);
        let (impl_generics, type_generics, _) = self.generics.split_for_impl();

        let lt = new_lifetime(&self.generics, None);

        let copy_gen = combine_gen!(self.generics; where #(for<#lt> #sized_field_types: #copy,)*);
        let copy_wc = copy_gen.split_for_impl().2;

        let debug_gen = combine_gen!(self.generics; where #(for<#lt> #sized_field_types: #debug,)*);
        let debug_wc: Option<&syn::WhereClause> = debug_gen.split_for_impl().2;

        let partial_eq_gen =
            combine_gen!(self.generics; where #(for<#lt> #sized_field_types: #partial_eq,)*);
        let partial_eq_wc = partial_eq_gen.split_for_impl().2;

        Some(quote! {
            impl #impl_generics #copy for #sized_ident #type_generics #copy_wc {}
            impl #impl_generics #clone for #sized_ident #type_generics #copy_wc {
                fn clone(&self) -> Self {
                    *self
                }
            }
            impl #impl_generics #debug for #sized_ident #type_generics #debug_wc {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    f.debug_struct(stringify!(#sized_ident))
                        #(.field(stringify!(#sized_field_idents), &{ self.#sized_field_idents }))*
                        .finish()
                }
            }

            impl #impl_generics #partial_eq for #sized_ident #type_generics #partial_eq_wc {
                fn eq(&self, other: &Self) -> bool {
                    #({self.#sized_field_idents}.eq(&{other.#sized_field_idents}) &&)*
                    true
                }
            }

            impl #impl_generics #eq for #sized_ident #type_generics #partial_eq_wc { }

        })
    }

    fn sized_bytemuck_derives(&self) -> Option<TokenStream> {
        Paths!(bytemuck, debug, copy, clone);
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

        let bytemuck_print = pretty_path(&bytemuck);
        let zeroable_bit_safety = format!("# Safety\nThis is safe because all fields are [`{bytemuck_print}::CheckedBitPattern::Bits`], which requires [`{bytemuck_print}::AnyBitPattern`], which requires [`{bytemuck_print}::Zeroable`]");
        let any_bit_pattern_safety = format!("# Safety\nThis is safe because all fields are [`{bytemuck_print}::CheckedBitPattern::Bits`], which requires [`{bytemuck_print}::AnyBitPattern`]");
        let no_uninit_safety = format!("# Safety\nThis is safe because the struct is `#[repr(C, packed)` (no padding bytes) and all fields are [`{bytemuck_print}::NoUninit`]");
        let zeroable_safety =
            format!("# Safety\nThis is safe because all fields are [`{bytemuck_print}::Zeroable`]");
        let checked_safety = format!(
            "# Safety\nThis is safe because all fields in [`Self::Bits`] are [`{bytemuck_print}::CheckedBitPattern::Bits`] and share the same repr. The checks are correctly (hopefully) and automatically generated by the macro."
        );

        let copy_bit_gen = combine_gen!(self.generics; where #(#bit_field_types: #copy),*);
        let copy_bit_wc = copy_bit_gen.split_for_impl().2;

        let debug_bit_gen = combine_gen!(self.generics; where #(#bit_field_types: #debug),*);
        let debug_bit_wc: Option<&syn::WhereClause> = debug_bit_gen.split_for_impl().2;

        Some(quote! {
            #validate_fields_are_trait

            #[doc = #zeroable_safety]
            #[automatically_derived]
            unsafe impl #impl_generics #bytemuck::Zeroable for #sized_ident #type_generics #where_clause {}

            #[doc = #no_uninit_safety]
            #[automatically_derived]
            unsafe impl #impl_generics #bytemuck::NoUninit for #sized_ident #type_generics #where_clause {}

            #[repr(C, packed)]
            #[allow(clippy::pub_underscore_fields)]
            #[doc = "`bytemuck`-generated struct for internal purposes only."]
            #[allow(missing_docs)]
            #vis struct #bit_ident #struct_generics #where_clause {
                #(#vis #sized_field_idents: #bit_field_types),*
            }

            impl #impl_generics #copy for #bit_ident #type_generics #copy_bit_wc {}
            impl #impl_generics #clone for #bit_ident #type_generics #copy_bit_wc {
                fn clone(&self) -> Self {
                    *self
                }
            }
            impl #impl_generics #debug for #bit_ident #type_generics #debug_bit_wc {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    f.debug_struct(stringify!(#bit_ident))
                        #(.field(stringify!(#sized_field_idents), &{ self.#sized_field_idents }))*
                        .finish()
                }
            }


            #[doc = #zeroable_bit_safety]
            #[automatically_derived]
            unsafe impl #impl_generics #bytemuck::Zeroable for #bit_ident #type_generics #where_clause {}

            #[doc = #any_bit_pattern_safety]
            #[automatically_derived]
            unsafe impl #impl_generics #bytemuck::AnyBitPattern for #bit_ident #type_generics #where_clause {}

            #[doc = #checked_safety]
            #[automatically_derived]
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
            #[automatically_derived]
            impl #impl_gen #deref for #ref_type #wc {
                type Target = #sized_type;

                fn deref(&self) -> &Self::Target {
                    &self.#sized_field_ident
                }
            }

            #[automatically_derived]
            impl #impl_gen #deref for #mut_type #wc {
                type Target = #sized_type;

                fn deref(&self) -> &Self::Target {
                    &self.#sized_field_ident
                }
            }

            #[automatically_derived]
            impl #impl_gen #deref_mut for #mut_type #wc {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.#sized_field_ident
                }
            }
        })
    }

    fn as_shared_impl(&self) -> TokenStream {
        Paths!(prelude);
        UnsizedStructContext!(self => with_sized_idents, with_sized_types, mut_ident, top_lt, ref_type, ref_ident);

        let underscore_gen = combine_gen!(self.generics; <'_>);
        let underscore_ty_gen = underscore_gen.split_for_impl().1;

        let (impl_gen, _, where_clause) = self.generics.split_for_impl();

        quote! {
            #[automatically_derived]
            impl #impl_gen #prelude::AsShared for #ref_ident #underscore_ty_gen #where_clause {
                type Ref<#top_lt> = #ref_type
                    where Self: #top_lt;
                fn as_shared(&self) -> Self::Ref<'_> {
                    #ref_ident {
                        #(#with_sized_idents: <#with_sized_types as #prelude::UnsizedType>::ref_as_ref(&self.#with_sized_idents),)*
                    }
                }
            }

            #[automatically_derived]
            impl #impl_gen #prelude::AsShared for #mut_ident #underscore_ty_gen #where_clause {
                type Ref<#top_lt> = #ref_type
                    where Self: #top_lt;
                fn as_shared(&self) -> Self::Ref<'_> {
                    #ref_ident {
                        #(#with_sized_idents: <#with_sized_types as #prelude::UnsizedType>::mut_as_ref(&self.#with_sized_idents),)*
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
        Paths!(prelude);
        UnsizedStructContext!(self => struct_type, with_sized_idents, with_sized_types);

        let from_owned_generics =
            combine_gen!(self.generics; where #(#with_sized_types: #prelude::FromOwned),*);

        let (impl_gen, _, where_clause) = from_owned_generics.split_for_impl();

        Some(quote! {
            #[automatically_derived]
            unsafe impl #impl_gen #prelude::FromOwned for #struct_type #where_clause {
                #[inline]
                fn byte_size(owned: &Self::Owned) -> usize {
                    #(<#with_sized_types as #prelude::FromOwned>::byte_size(&owned.#with_sized_idents) +)* 0
                }

                #[inline]
                fn from_owned(owned: Self::Owned, bytes: &mut &mut [u8]) -> Result<usize> {
                    let size = {
                        #(<#with_sized_types as #prelude::FromOwned>::from_owned(owned.#with_sized_idents, bytes)? +)* 0
                    };
                    Ok(size)
                }
            }
        })
    }

    fn unsized_type_impl(&self) -> TokenStream {
        Paths!(prelude, result);
        UnsizedStructContext!(self => ref_type, struct_type, top_lt, with_sized_types, ref_ident, with_sized_idents,
            mut_type, mut_ident, struct_ident
        );
        let (impl_gen, _, where_clause) = self.generics.split_for_impl();

        let (last_ty, all_but_last_ty) = with_sized_types
            .split_last()
            .expect("self should have fields");
        let (_, all_but_last_idents) = with_sized_idents
            .split_last()
            .expect("self should have fields");
        let zst_messages = all_but_last_idents.iter().map(|ident| {
            format!("Zero-sized types are not allowed in the middle of UnsizedType structs.\n     Found ZST at `{struct_ident}.{ident}`")
        });

        let owned_type = self.args.owned_type.as_ref().unwrap_or(&self.owned_type);
        let owned_from_ref = self.args.owned_from_ref.as_ref().map(|path| quote!(#path(r))).unwrap_or_else(|| {
            quote! {
                Ok(Self::Owned {
                    #(#with_sized_idents: <#with_sized_types as #prelude::UnsizedType>::owned_from_ref(&r.#with_sized_idents)?,)*
                })
            }
        });

        let mut_lt = new_lifetime(&self.generics, Some("m"));

        quote! {
            #[automatically_derived]
            unsafe impl #impl_gen #prelude::UnsizedType for #struct_type #where_clause {
                type Ref<#top_lt> = #ref_type;
                type Mut<#top_lt> = #mut_type;
                type Owned = #owned_type;

                const ZST_STATUS: bool = {
                    #(if !<#all_but_last_ty as #prelude::UnsizedType>::ZST_STATUS {
                        panic!(#zst_messages);
                    })*
                    <#last_ty as UnsizedType>::ZST_STATUS
                };

                fn ref_as_ref<#mut_lt>(r: &#mut_lt Self::Ref<'_>) -> Self::Ref<#mut_lt> {
                    #ref_ident{
                        #(#with_sized_idents: <#with_sized_types as #prelude::UnsizedType>::ref_as_ref(&r.#with_sized_idents),)*
                    }
                }

                fn mut_as_ref<#mut_lt>(m: &#mut_lt Self::Mut<'_>) -> Self::Ref<#mut_lt> {
                    #ref_ident{
                        #(#with_sized_idents: <#with_sized_types as #prelude::UnsizedType>::mut_as_ref(&m.#with_sized_idents),)*
                    }
                }

                fn get_ref<#top_lt>(data: &mut &#top_lt [u8]) -> #result<Self::Ref<#top_lt>> {
                    Ok(#ref_ident {
                        #(#with_sized_idents: <#with_sized_types as #prelude::UnsizedType>::get_ref(data)?,)*
                    })
                }

                unsafe fn get_mut<#top_lt>(data: &mut *mut [u8]) -> #result<Self::Mut<#top_lt>> {
                    Ok(#mut_ident {
                        #(#with_sized_idents: unsafe {<#with_sized_types as #prelude::UnsizedType>::get_mut(data)? },)*
                    })
                }

                fn owned_from_ref(r: &Self::Ref<'_>) -> #result<Self::Owned> {
                    #owned_from_ref
                }

                unsafe fn resize_notification(self_mut: &mut Self::Mut<'_>, source_ptr: *const (), change: isize) -> #result<()> {
                    #(unsafe {<#with_sized_types as #prelude::UnsizedType>::resize_notification(&mut self_mut.#with_sized_idents, source_ptr, change)}?;)*
                    Ok(())
                }
            }
        }
    }

    fn unsized_init_default_impl(&self) -> TokenStream {
        Paths!(prelude, result);
        UnsizedStructContext!(self => struct_type, with_sized_types);
        let default_init_generics = combine_gen!(self.generics; where #(#with_sized_types: #prelude::UnsizedInit<#prelude::DefaultInit>),*);
        let (default_init_impl, _, default_init_where) = default_init_generics.split_for_impl();
        let unsized_init = quote!(#prelude::UnsizedInit<#prelude::DefaultInit>);
        quote! {
            #[allow(trivial_bounds)]
            #[automatically_derived]
            unsafe impl #default_init_impl #unsized_init for #struct_type #default_init_where {
                const INIT_BYTES: usize = 0 #(+ <#with_sized_types as #unsized_init>::INIT_BYTES)*;
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
        UnsizedStructContext!(self => vis, with_sized_types, with_sized_vis_pub, struct_type, struct_ident);

        let init_struct_ident = format_ident!("{struct_ident}Init");

        let sized_field_ident = self
            .sized_ident
            .as_ref()
            .map(|_| new_ident("sized", get_field_idents(&self.unsized_fields), false));

        let sized_with_unsized_idents = sized_field_ident
            .iter()
            .chain(self.unsized_field_idents.iter())
            .cloned()
            .collect_vec();

        let init_generic_idents = self
            .with_sized_idents
            .iter()
            .map(|ident| {
                format_ident!(
                    "{struct_ident}Init{}",
                    ident.to_string().to_upper_camel_case()
                )
            })
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
                #(#with_sized_vis_pub #sized_with_unsized_idents: #init_generic_idents,)*
            }

            #[allow(trivial_bounds)]
            #[automatically_derived]
            unsafe impl #init_impl_impl_generics #prelude::UnsizedInit<#init_struct_type> for #struct_type #init_impl_where_clause {
                const INIT_BYTES: usize = 0 #(+ <#with_sized_types as #prelude::UnsizedInit<#init_generic_idents>>::INIT_BYTES)*;

                unsafe fn init(
                    bytes: &mut &mut [u8],
                    arg: #init_struct_type,
                ) -> #result<()> {
                    #(
                        unsafe { <#with_sized_types as #prelude::UnsizedInit<#init_generic_idents>>::init(bytes, arg.#sized_with_unsized_idents) }?;
                    )*
                    Ok(())
                }
            }
        }
    }

    fn extension_impl(&self) -> TokenStream {
        Paths!(prelude, sized);
        UnsizedStructContext!(self => struct_ident, unsized_fields, struct_type, top_lt);
        let parent_lt = new_lifetime(&self.generics, Some("parent"));
        let child = new_lifetime(&self.generics, Some("child"));
        let p = new_generic(&self.generics, Some("P"));

        let ext_trait_generics = combine_gen!(self.generics; <#parent_lt, #top_lt, #p> where
            Self: #prelude::ExclusiveRecurse + #sized,
        );

        let (impl_gen, ty_gen, wc) = ext_trait_generics.split_for_impl();
        let pub_extension_ident = format_ident!("{struct_ident}ExclusiveExt");
        let priv_extension_ident = format_ident!("{struct_ident}ExclusiveExtPrivate");

        let (pub_unsized_fields, priv_unsized_fields): (Vec<_>, Vec<_>) =
            unsized_fields.iter().partition(|field| match field.vis {
                Visibility::Public(_) => true,
                Visibility::Inherited => false,
                Visibility::Restricted(_) => {
                    abort!(field.vis, "Unsized fields must be `pub` or private")
                }
            });

        let make_ext_trait = |vis: &Visibility, fields: Vec<&Field>, extension_ident: &Ident| {
            let field_idents = get_field_idents(&fields).collect_vec();
            let field_types = get_field_types(&fields).collect_vec();
            quote! {
                #vis trait #extension_ident #impl_gen #wc
                {
                    #(
                        fn #field_idents<#child>(&#child mut self) -> #prelude::ExclusiveWrapper<#child, #top_lt, <#field_types as #prelude::UnsizedType>::Mut<#top_lt>, Self>;
                    )*
                }

                #[automatically_derived]
                impl #impl_gen #extension_ident #ty_gen for #prelude::ExclusiveWrapper<#parent_lt, #top_lt, <#struct_type as #prelude::UnsizedType>::Mut<#top_lt>, #p> #wc {
                    #(
                        fn #field_idents<#child>(&#child mut self) -> #prelude::ExclusiveWrapper<#child, #top_lt, <#field_types as #prelude::UnsizedType>::Mut<#top_lt>, Self> {
                            unsafe { #prelude::ExclusiveWrapper::map_mut::<#field_types>(self, |x| &raw mut (*x).#field_idents) }
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
