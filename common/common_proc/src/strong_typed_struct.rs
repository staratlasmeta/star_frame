use crate::get_crate_name;
use proc_macro2::TokenStream;
use proc_macro_error::*;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::*;

#[derive(Default)]
struct StrongTypedArgs {
    strong_typed_struct_name: Option<Ident>,
}
impl Parse for StrongTypedArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(StrongTypedArgs {
            strong_typed_struct_name: input.parse()?,
        })
    }
}

#[derive(Debug)]
struct FixedPointUnit {
    _comma: Token![,],
    unit_type: Type,
}
impl Parse for FixedPointUnit {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(FixedPointUnit {
            _comma: input.parse()?,
            unit_type: input.parse()?,
        })
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
struct FixedPointFieldArgs {
    div: LitInt,
    unit: Option<FixedPointUnit>,
}
impl Parse for FixedPointFieldArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(FixedPointFieldArgs {
            div: input.parse()?,
            unit: if input.peek(Token![,]) {
                Some(input.parse()?)
            } else {
                None
            },
        })
    }
}

custom_keyword!(skip_pubkey_check);

struct KeyForArgs {
    ty: Type,
    skip_pubkey_check: Option<skip_pubkey_check>,
}
impl Parse for KeyForArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            ty: input.parse()?,
            skip_pubkey_check: input.parse()?,
        })
    }
}

enum FieldArgs {
    FixedPoint(FixedPointFieldArgs),
    KeyFor(KeyForArgs),
    OptionalKeyFor(KeyForArgs),
    EnumWrapper(Type),
    SubStruct,
    Bool,
}

pub fn strong_typed_struct_impl(derive_input: DeriveInput) -> TokenStream {
    let crate_name = get_crate_name();
    let input = match derive_input.data {
        Data::Struct(strct) => strct,
        Data::Enum(_) | Data::Union(_) => {
            abort_call_site!("StrongTypedStruct can only be derived for structs")
        }
    };

    let args = derive_input
        .attrs
        .iter()
        .find(|attr| attr.path.is_ident("strong_type"))
        .map(|attr| attr.parse_args::<StrongTypedArgs>().unwrap())
        .unwrap_or_default();
    let ident = derive_input.ident;
    let vis = &derive_input.vis;
    let struct_name = args.strong_typed_struct_name.unwrap_or_else(|| {
        let mut name = ident.to_string();
        name.push_str("StrongTyped");
        Ident::new(&name, ident.span())
    });

    let path_to_fixed_point = |fixed_point_unit: Type, fixed_point_div: LitInt, path: &TypePath| {
        if path.path.segments.len() != 1 {
            abort!(path, "Only integer types are supported");
        }
        match path.path.segments[0].ident.to_string().as_str() {
            "u8" => {
                quote! { #crate_name::FixedPointU8<#fixed_point_unit, #fixed_point_div> }
            }
            "u16" => {
                quote! { #crate_name::FixedPointU16<#fixed_point_unit, #fixed_point_div> }
            }
            "u32" => {
                quote! { #crate_name::FixedPointU32<#fixed_point_unit, #fixed_point_div> }
            }
            "u64" => {
                quote! { #crate_name::FixedPointU64<#fixed_point_unit, #fixed_point_div> }
            }
            "u128" => {
                quote! { #crate_name::FixedPointU128<#fixed_point_unit, #fixed_point_div> }
            }
            "i8" => {
                quote! { #crate_name::FixedPointI8<#fixed_point_unit, #fixed_point_div> }
            }
            "i16" => {
                quote! { #crate_name::FixedPointI16<#fixed_point_unit, #fixed_point_div> }
            }
            "i32" => {
                quote! { #crate_name::FixedPointI32<#fixed_point_unit, #fixed_point_div> }
            }
            "i64" => {
                quote! { #crate_name::FixedPointI64<#fixed_point_unit, #fixed_point_div> }
            }
            "i128" => {
                quote! { #crate_name::FixedPointI128<#fixed_point_unit, #fixed_point_div> }
            }
            _ => abort!(path, "Only integer types are supported"),
        }
    };

    let mut extra_checks = Vec::new();

    let convert_field = |field: &Field| {
        let mut field_arg = None;
        for attr in &field.attrs {
            if attr.path.is_ident("fixed_point")
                && field_arg
                    .replace(FieldArgs::FixedPoint(attr.parse_args()?))
                    .is_some()
            {
                abort!(attr, "Only one strong type attribute allowed per field");
            }
            if attr.path.is_ident("key_for")
                && field_arg
                    .replace(FieldArgs::KeyFor(attr.parse_args()?))
                    .is_some()
            {
                abort!(attr, "Only one strong type attribute allowed per field");
            }
            if attr.path.is_ident("optional_key_for")
                && field_arg
                    .replace(FieldArgs::OptionalKeyFor(attr.parse_args()?))
                    .is_some()
            {
                abort!(attr, "Only one strong type attribute allowed per field");
            }
            if attr.path.is_ident("enum_wrapper")
                && field_arg
                    .replace(FieldArgs::EnumWrapper(attr.parse_args()?))
                    .is_some()
            {
                abort!(attr, "Only one strong type attribute allowed per field");
            }
            if attr.path.is_ident("strong_sub_struct")
                && field_arg.replace(FieldArgs::SubStruct).is_some()
            {
                abort!(attr, "Only one strong type attribute allowed per field");
            }
            if attr.path.is_ident("bool_wrapper") && field_arg.replace(FieldArgs::Bool).is_some() {
                abort!(attr, "Only one strong type attribute allowed per field");
            }
        }
        match field_arg {
            None => Ok(quote! { #field }),
            Some(field_args) => {
                let mut field = field.clone();
                let type_tokens = match field_args {
                    FieldArgs::FixedPoint(fixed_point) => {
                        let fixed_point_unit = fixed_point.unit.map_or_else(
                            || parse2(quote! { #crate_name::Unitless }).unwrap(),
                            |unit| unit.unit_type,
                        );
                        match &field.ty {
                            Type::Path(path) => {
                                path_to_fixed_point(fixed_point_unit, fixed_point.div, path)
                            }
                            Type::Array(TypeArray { elem, len, .. }) => match &**elem {
                                Type::Path(path) => {
                                    let new_type = path_to_fixed_point(
                                        fixed_point_unit,
                                        fixed_point.div,
                                        path,
                                    );
                                    quote! { [#new_type; #len] }
                                }
                                _ => abort!(field.ty, "Only integer types are supported"),
                            },
                            _ => abort!(field.ty, "Only integer types are supported"),
                        }
                    }
                    FieldArgs::KeyFor(KeyForArgs {
                        ty,
                        skip_pubkey_check,
                    }) => {
                        let (elem, out) = match &field.ty {
                            Type::Array(TypeArray { elem, len, .. }) => {
                                (&**elem, quote! { [#crate_name::KeyFor<#ty>; #len] })
                            }
                            elem => (elem, quote! { #crate_name::KeyFor<#ty> }),
                        };
                        if skip_pubkey_check.is_none() {
                            match elem {
                                Type::Path(path)
                                    if path
                                        .path
                                        .segments
                                        .last()
                                        .map_or(false, |seg| seg.ident == "Pubkey") => {}
                                ty => abort!(ty, "Type must be `Pubkey`"),
                            }
                        }
                        out
                    }
                    FieldArgs::OptionalKeyFor(KeyForArgs {
                        ty,
                        skip_pubkey_check,
                    }) => {
                        let (elem, out) = match &field.ty {
                            Type::Array(TypeArray { elem, len, .. }) => {
                                (&**elem, quote! { [#crate_name::OptionalKeyFor<#ty>; #len] })
                            }
                            elem => (elem, quote! { #crate_name::OptionalKeyFor<#ty> }),
                        };
                        if skip_pubkey_check.is_none() {
                            match elem {
                                Type::Path(path)
                                    if path.path.segments.last().map_or(false, |seg| {
                                        seg.ident == "Pubkey"
                                            || seg.ident == "OptionalNonSystemPubkey"
                                    }) => {}
                                ty => {
                                    abort!(ty, "Type must be `Pubkey` or `OptionalNonSystemPubkey`")
                                }
                            }
                        }
                        out
                    }
                    FieldArgs::EnumWrapper(ty) => match &field.ty {
                        Type::Array(TypeArray { elem, len, .. }) => {
                            extra_checks.push(quote! {
                                #crate_name::static_assertions::assert_impl_all!(#ty: #crate_name::UnitEnumFromRepr<Repr = #elem>);
                            });
                            quote! { [#crate_name::UnitEnumWrapper<#ty>; #len] }
                        }
                        elem => {
                            extra_checks.push(quote! {
                                #crate_name::static_assertions::assert_impl_all!(#ty: #crate_name::UnitEnumFromRepr<Repr = #elem>);
                            });
                            quote! { #crate_name::UnitEnumWrapper<#ty> }
                        }
                    },
                    FieldArgs::SubStruct => match &field.ty {
                        Type::Array(TypeArray { elem, len, .. }) => {
                            quote! { [<#elem as #crate_name::StrongTypedStruct>::StrongTyped; #len] }
                        }
                        field_ty => {
                            quote! { <#field_ty as #crate_name::StrongTypedStruct>::StrongTyped }
                        }
                    },
                    FieldArgs::Bool => {
                        let field_ty = &field.ty;
                        {
                            extra_checks.push(quote! {
                             #crate_name::static_assertions::assert_impl_all!(#field_ty: #crate_name::Boolable);
                            });
                            quote! { #crate_name::BoolWrapper}
                        }
                    }
                };
                field.ty = parse2(type_tokens)?;
                field.attrs.retain(|attr| attr.path.is_ident("doc"));
                Ok(quote! { #field })
            }
        }
    };

    let fixed_point_struct = match &input.fields {
        Fields::Unnamed(fields) => {
            let fields_mapped = fields
                .unnamed
                .iter()
                .map(convert_field)
                .collect::<Result<Vec<_>>>()
                .unwrap_or_else(|e| abort!(e.span(), "{}", e));

            quote! {
                #vis struct #struct_name (
                    #(#fields_mapped,)*
                )
            }
        }
        Fields::Named(fields) => {
            let fields_mapped = fields
                .named
                .iter()
                .map(convert_field)
                .collect::<Result<Vec<_>>>()
                .unwrap_or_else(|e| abort!(e.span(), "{}", e));

            quote! {
                #vis struct #struct_name {
                    #(#fields_mapped,)*
                }
            }
        }
        Fields::Unit => quote! {
            #vis struct #struct_name;
        },
    };

    let doc_string =
        String::from("Strongly typed representation of [`") + &ident.to_string() + "`]";

    quote! {
        #[doc = #doc_string]
        #[derive(Copy, Clone, Debug, #crate_name::bytemuck::Pod, #crate_name::bytemuck::Zeroable)]
        #[repr(C, packed)]
        #fixed_point_struct
        #[automatically_derived]
        impl #crate_name::DataSize for #struct_name {
            const MIN_DATA_SIZE: usize = <#ident as #crate_name::DataSize>::MIN_DATA_SIZE;
        }
        #[automatically_derived]
        unsafe impl #crate_name::SafeZeroCopy for #struct_name {}

        #crate_name::static_assertions::const_assert_eq!(
            ::std::mem::size_of::<#ident>(),
            ::std::mem::size_of::<#struct_name>(),
        );

        #(#extra_checks)*

        #[automatically_derived]
        unsafe impl #crate_name::StrongTypedStruct for #ident {
            type StrongTyped = #struct_name;

            fn as_strong_typed(&self) -> &Self::StrongTyped {
                #crate_name::bytemuck::cast_ref(self)
            }
            fn as_strong_typed_mut(&mut self) -> &mut Self::StrongTyped {
                #crate_name::bytemuck::cast_mut(self)
            }
        }
    }
}
