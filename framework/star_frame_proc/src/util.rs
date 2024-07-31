use crate::get_crate_name;
use derive_more::{Deref, DerefMut};
use itertools::Itertools;
use proc_macro2::TokenStream;
use proc_macro_error::{abort, abort_call_site};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use std::fmt::Debug;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    bracketed, parse_quote, token, Attribute, ConstParam, Data, DataStruct, DataUnion, DeriveInput,
    Expr, ExprLit, Field, Fields, GenericParam, Generics, Ident, ItemEnum, ItemStruct, Lifetime,
    LifetimeParam, Lit, Meta, MetaNameValue, Path, Token, Type, TypeParam, WhereClause,
};

#[derive(Debug, Clone)]
pub struct Paths {
    pub crate_name: TokenStream,
    pub macro_prelude: TokenStream,

    // static_assertions
    pub static_assertions: TokenStream,

    pub prelude: TokenStream,
    // std
    pub box_ty: TokenStream,
    pub clone: TokenStream,
    pub copy: TokenStream,
    pub debug: TokenStream,
    pub default: TokenStream,
    pub deref: TokenStream,
    pub deref_mut: TokenStream,
    pub eq: TokenStream,
    pub non_null: TokenStream,
    pub panic: TokenStream,
    pub partial_eq: TokenStream,
    pub phantom_data: TokenStream,
    pub ptr: TokenStream,

    pub size_of: TokenStream,

    // derivative
    pub derivative: TokenStream,
    // account set
    pub account_set: TokenStream,
    pub account_set_decode: TokenStream,
    pub account_set_validate: TokenStream,
    pub account_set_cleanup: TokenStream,
    pub get_seeds: TokenStream,
    pub program_account: TokenStream,
    // syscalls
    pub syscalls: TokenStream,
    pub syscall_invoke: TokenStream,

    pub solana_runtime: TokenStream,

    pub result: TokenStream,
    // idl
    #[cfg(feature = "idl")]
    pub account_to_idl: TokenStream,
    #[cfg(feature = "idl")]
    pub account_set_to_idl: TokenStream,
    #[cfg(feature = "idl")]
    pub instruction_to_idl: TokenStream,
    #[cfg(feature = "idl")]
    pub instruction_set_to_idl: TokenStream,
    #[cfg(feature = "idl")]
    pub type_to_idl: TokenStream,

    #[cfg(feature = "idl")]
    pub program_to_idl: TokenStream,
    // star frame idl
    pub semver: TokenStream,
    pub idl_definition: TokenStream,
    pub idl_definition_ref: TokenStream,
    pub idl_type: TokenStream,
    pub idl_type_def: TokenStream,
    pub idl_type_id: TokenStream,
    pub idl_field: TokenStream,
    pub idl_account: TokenStream,
    pub idl_account_set_def: TokenStream,
    pub idl_account_set: TokenStream,
    pub idl_account_set_struct_field: TokenStream,
    pub idl_instruction_def: TokenStream,
    pub idl_instruction: TokenStream,
    pub idl_seeds: TokenStream,
    pub account_id: TokenStream,

    pub account_set_id: TokenStream,
    // instruction
    pub star_frame_instruction: TokenStream,
    pub instruction_set: TokenStream,

    pub instruction: TokenStream,
    // program
    pub system_program: TokenStream,
    pub star_frame_program: TokenStream,

    pub declared_program_type: Type,
    // idents
    pub account_ident: Ident,
    pub account_set_ident: Ident,
    pub decode_ident: Ident,
    pub validate_ident: Ident,
    pub cleanup_ident: Ident,
    pub idl_ident: Ident,
    pub star_frame_program_ident: Ident,
    pub program_id_ident: Ident,

    pub idl_ty_program_ident: Ident,
    pub align1: TokenStream,
    pub packed_value_checked: TokenStream,
    pub advance: TokenStream,

    pub advance_array: TokenStream,

    // bytemuck
    pub checked: TokenStream,
    pub bytemuck: TokenStream,
    pub checked_bit_pattern: TokenStream,
    pub pod: TokenStream,
    // solana
    pub account_info: TokenStream,
    pub program_error: TokenStream,
    pub program_result: TokenStream,
    pub sol_memset: TokenStream,

    pub pubkey: TokenStream,

    pub msg: TokenStream,
    // anyhow
    pub anyhow_macro: TokenStream,
}

impl Default for Paths {
    fn default() -> Self {
        let crate_name = get_crate_name();
        Self {
            crate_name: crate_name.clone(),

            macro_prelude: quote! { #crate_name::__private::macro_prelude },
            prelude: quote! { #crate_name::prelude },

            // static_assertions
            static_assertions: quote! { #crate_name::static_assertions },

            // std
            box_ty: quote! { ::std::boxed::Box },
            clone: quote! { ::std::clone::Clone },
            copy: quote! { ::std::marker::Copy },
            debug: quote! { ::std::fmt::Debug },
            default: quote! { ::std::default::Default },
            deref: quote! { ::std::ops::Deref },
            deref_mut: quote! { ::std::ops::DerefMut },
            eq: quote! { ::std::cmp::Eq },
            non_null: quote! { ::std::ptr::NonNull },
            panic: quote! { ::std::panic },
            partial_eq: quote! { ::std::cmp::PartialEq },
            phantom_data: quote! { ::std::marker::PhantomData },
            ptr: quote! { ::std::ptr },
            size_of: quote! { ::std::mem::size_of },

            // derivative
            derivative: quote! { #crate_name::derivative::Derivative },

            // account set
            account_set: quote! { #crate_name::account_set::AccountSet },
            account_set_decode: quote! { #crate_name::account_set::AccountSetDecode },
            account_set_validate: quote! { #crate_name::account_set::AccountSetValidate },
            account_set_cleanup: quote! { #crate_name::account_set::AccountSetCleanup },
            get_seeds: quote! { #crate_name::account_set::GetSeeds },
            program_account: quote! { #crate_name::account_set::ProgramAccount },

            // syscalls
            syscalls: quote! { #crate_name::syscalls::Syscalls },
            syscall_invoke: quote! { #crate_name::syscalls::SyscallInvoke },
            solana_runtime: quote! { #crate_name::syscalls::solana_runtime::SolanaRuntime },

            result: quote! { #crate_name::Result },

            // idl
            #[cfg(feature = "idl")]
            account_to_idl: quote! { #crate_name::idl::AccountToIdl },
            #[cfg(feature = "idl")]
            account_set_to_idl: quote! { #crate_name::idl::AccountSetToIdl },
            #[cfg(feature = "idl")]
            instruction_to_idl: quote! { #crate_name::idl::InstructionToIdl },
            #[cfg(feature = "idl")]
            instruction_set_to_idl: quote! { #crate_name::idl::InstructionSetToIdl },
            #[cfg(feature = "idl")]
            type_to_idl: quote! { #crate_name::idl::TypeToIdl },
            #[cfg(feature = "idl")]
            program_to_idl: quote! { #crate_name::idl::ProgramToIdl },

            // star frame idl
            semver: quote! { #crate_name::star_frame_idl::SemVer },
            idl_definition: quote! { #crate_name::star_frame_idl::IdlDefinition },
            idl_definition_ref: quote! { #crate_name::star_frame_idl::IdlDefinitionReference },
            idl_type: quote! { #crate_name::star_frame_idl::ty::IdlType },
            idl_type_def: quote! { #crate_name::star_frame_idl::ty::IdlTypeDef },
            idl_type_id: quote! { #crate_name::star_frame_idl::ty::TypeId },
            idl_field: quote! { #crate_name::star_frame_idl::ty::IdlField },
            idl_account: quote! { #crate_name::star_frame_idl::account::IdlAccount },
            idl_account_set_def: quote! { #crate_name::star_frame_idl::account_set::IdlAccountSetDef },
            idl_account_set: quote! { #crate_name::star_frame_idl::account_set::IdlAccountSet },
            idl_account_set_struct_field: quote! { #crate_name::star_frame_idl::account_set::IdlAccountSetStructField },
            idl_instruction_def: quote! { #crate_name::star_frame_idl::instruction::IdlInstructionDef },
            idl_instruction: quote! { #crate_name::star_frame_idl::instruction::IdlInstruction },
            idl_seeds: quote! { #crate_name::star_frame_idl::seeds::IdlSeeds },
            account_id: quote! { #crate_name::star_frame_idl::account::AccountId },
            account_set_id: quote! { #crate_name::star_frame_idl::account_set::AccountSetId },

            // instruction
            star_frame_instruction: quote! { #crate_name::instruction::StarFrameInstruction },
            instruction_set: quote! { #crate_name::instruction::InstructionSet },
            instruction: quote! { #crate_name::instruction::Instruction },

            // program
            system_program: quote! { #crate_name::program::system_program::SystemProgram },
            star_frame_program: quote! { #crate_name::program::StarFrameProgram },
            declared_program_type: parse_quote! { crate::StarFrameDeclaredProgram },

            // idents
            account_ident: format_ident!("account"),
            account_set_ident: format_ident!("account_set"),
            decode_ident: format_ident!("decode"),
            validate_ident: format_ident!("validate"),
            cleanup_ident: format_ident!("cleanup"),
            idl_ident: format_ident!("idl"),
            idl_ty_program_ident: format_ident!("program"),
            star_frame_program_ident: format_ident!("program"),
            program_id_ident: format_ident!("program_id"),

            align1: quote! { #crate_name::align1::Align1 },
            packed_value_checked: quote! { #crate_name::data_types::PackedValueChecked },
            advance_array: quote! { #crate_name::advance::AdvanceArray },
            advance: quote! { #crate_name::advance::Advance},

            // bytemuck
            bytemuck: quote! { #crate_name::bytemuck },
            checked: quote! { #crate_name::bytemuck::checked },
            checked_bit_pattern: quote! { #crate_name::bytemuck::checked::CheckedBitPattern },
            pod: quote! { #crate_name::bytemuck::Pod },

            // solana
            account_info: quote! { #crate_name::solana_program::account_info::AccountInfo },
            program_error: quote! { #crate_name::solana_program::program_error::ProgramError },
            program_result: quote! { #crate_name::solana_program::entrypoint::ProgramResult },
            sol_memset: quote! { #crate_name::solana_program::program_memory::sol_memset },
            pubkey: quote! { #crate_name::solana_program::pubkey::Pubkey },
            msg: quote! { #crate_name::solana_program::msg },

            // anyhow
            anyhow_macro: quote! { #crate_name::anyhow::anyhow },
        }
    }
}

#[derive(Debug, Deref, DerefMut, Clone, Default)]
pub struct BetterGenerics {
    _bracket: token::Bracket,
    #[deref]
    #[deref_mut]
    generics: Generics,
}
impl BetterGenerics {
    pub fn into_inner(self) -> Generics {
        self.generics
    }
}
impl Parse for BetterGenerics {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _bracket = bracketed!(content in input);
        let mut generics = if !content.peek(Token![<]) {
            Generics::default()
        } else {
            let lt_token: Token![<] = content.parse()?;

            let mut params = Punctuated::new();
            loop {
                if content.peek(Token![>]) {
                    break;
                }

                let attrs = content.call(Attribute::parse_outer)?;
                let lookahead = content.lookahead1();
                if lookahead.peek(Lifetime) {
                    params.push_value(GenericParam::Lifetime(LifetimeParam {
                        attrs,
                        ..content.parse()?
                    }));
                } else if lookahead.peek(Ident) {
                    params.push_value(GenericParam::Type(TypeParam {
                        attrs,
                        ..content.parse()?
                    }));
                } else if lookahead.peek(Token![const]) {
                    params.push_value(GenericParam::Const(ConstParam {
                        attrs,
                        ..content.parse()?
                    }));
                } else {
                    return Err(lookahead.error());
                }

                if content.peek(Token![>]) {
                    break;
                }
                let punct = content.parse()?;
                params.push_punct(punct);
            }

            let gt_token: Token![>] = content.parse()?;

            Generics {
                lt_token: Some(lt_token),
                params,
                gt_token: Some(gt_token),
                where_clause: None,
            }
        };
        generics.where_clause = content.parse()?;
        Ok(Self { _bracket, generics })
    }
}

pub trait CombineGenerics {
    fn combine(&self, other: Self) -> Self;
}

impl CombineGenerics for Generics {
    fn combine(&self, other: Self) -> Self {
        let generics_a = self.clone();

        let params = generics_a.params.into_iter().chain(other.params).collect();

        let where_clause: Option<WhereClause> =
            if generics_a.where_clause.is_some() || other.where_clause.is_some() {
                let predicates = other
                    .where_clause
                    .into_iter()
                    .chain(generics_a.where_clause)
                    .flat_map(|w| w.predicates)
                    .collect();
                Some(WhereClause {
                    where_token: Default::default(),
                    predicates,
                })
            } else {
                None
            };

        Generics {
            params,
            where_clause,
            ..Default::default()
        }
    }
}

#[allow(dead_code)]
pub fn get_docs<'a>(attrs: impl IntoIterator<Item = &'a Attribute>) -> String {
    attrs
        .into_iter()
        .filter(|a| a.path().is_ident("doc"))
        .map(|a: &'a Attribute| {
            if let Meta::NameValue(MetaNameValue {
                value:
                    Expr::Lit(ExprLit {
                        lit: Lit::Str(str), ..
                    }),
                ..
            }) = &a.meta
            {
                str.value()
            } else {
                abort!(a, "Expected doc attribute to be a name value pair")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn verify_repr(
    attrs: &[Attribute],
    repr_required: impl IntoIterator<Item = Ident>,
    allow_others: bool,
    require_present: bool,
) -> Punctuated<Ident, Token![,]> {
    let repr = attrs.iter().find(|attr| attr.path().is_ident("repr"));
    if let Some(repr) = repr {
        let repr_ty = repr
            .parse_args_with(|p: ParseStream| p.parse_terminated(Ident::parse, Token![,]))
            .unwrap_or_else(|e| abort!(repr, "Could not parse repr type: {}", e));
        let mut repr_required = repr_required
            .into_iter()
            .map(|r| (r, false))
            .collect::<Vec<_>>();
        for repr_ty in repr_ty.iter() {
            if let Some((_, found)) = repr_required.iter_mut().find(|(r, _)| r == repr_ty) {
                *found = true;
            } else if !allow_others {
                abort!(repr_ty, "Unexpected repr type: {}", quote! { #repr_ty });
            }
        }
        for (r, found) in repr_required {
            if !found {
                abort_call_site!("Missing #[repr({:?})] attribute", r);
            }
        }
        repr_ty
    } else if require_present {
        abort_call_site!(
            "Missing #[repr({:?})] attribute",
            repr_required.into_iter().collect::<Vec<_>>()
        );
    } else {
        Punctuated::new()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StrippedAttribute {
    pub index: usize,
    pub attribute: Attribute,
}

pub trait EnumerableAttributes {
    fn enumerate_attributes(&mut self) -> impl Iterator<Item = (usize, &mut Vec<Attribute>)>;
}

impl EnumerableAttributes for syn::ItemStruct {
    fn enumerate_attributes(&mut self) -> impl Iterator<Item = (usize, &mut Vec<Attribute>)> {
        self.fields
            .iter_mut()
            .enumerate()
            .map(|(index, f)| (index, &mut f.attrs))
    }
}

impl EnumerableAttributes for syn::ItemEnum {
    fn enumerate_attributes(&mut self) -> impl Iterator<Item = (usize, &mut Vec<Attribute>)> {
        self.variants
            .iter_mut()
            .enumerate()
            .map(|(index, v)| (index, &mut v.attrs))
    }
}

impl EnumerableAttributes for Vec<Attribute> {
    fn enumerate_attributes(&mut self) -> impl Iterator<Item = (usize, &mut Vec<Attribute>)> {
        std::iter::once((0, self))
    }
}

/// Strips all matching attributes from each attribute group (e.g., struct fields, enum variants) and returns them in order with
/// their group index.
pub fn strip_inner_attributes<'a>(
    item: &'a mut impl EnumerableAttributes,
    attribute_name: &'a str,
) -> impl Iterator<Item = StrippedAttribute> + 'a {
    item.enumerate_attributes().flat_map(|(index, attrs)| {
        let mut removed = vec![];
        attrs.retain(|attr| {
            attr.path()
                .is_ident(attribute_name)
                .then(|| {
                    removed.push(StrippedAttribute {
                        index,
                        attribute: attr.clone(),
                    });
                    false
                })
                .unwrap_or(true)
        });
        removed
    })
}

pub fn make_derivative_attribute(
    traits: Punctuated<Path, Token![,]>,
    types: &[impl ToTokens],
) -> Attribute {
    let bounds = traits
        .iter()
        .map(|t| {
            let derivitive_bounds = types.iter().map(|ty| quote!(#ty: #t)).collect::<Vec<_>>();
            let derivative_bounds = quote!(#(#derivitive_bounds),*).to_string();
            quote!(#t(bound = #derivative_bounds))
        })
        .collect_vec();
    parse_quote!(#[derivative(#(#bounds),*)])
}

pub fn add_derivative_attributes(
    struct_item: &mut ItemStruct,
    traits: Punctuated<Path, Token![,]>,
) {
    let attributes = make_derivative_attribute(traits, &get_field_types(struct_item).collect_vec());
    struct_item.attrs.push(attributes);
}

pub fn get_field_types(fields: &impl FieldIter) -> impl Iterator<Item = &Type> {
    fields.field_iter().map(|field| &field.ty)
}

/// Check that all fields implement a given trait
///
/// Adapted from the bytemuck derive crate
pub fn generate_fields_are_trait<T: GetGenerics + FieldIter + Spanned>(
    input: &T,
    trait_: Punctuated<syn::Path, Token![+]>,
) -> TokenStream {
    let generics = input.get_generics();
    let (impl_generics, _ty_generics, where_clause) = generics.split_for_impl();
    let span = input.span();
    let field_types = get_field_types(input);
    quote_spanned! {span => const _: fn() = || {
        #[allow(clippy::missing_const_for_fn)]
        #[doc(hidden)]
        fn check #impl_generics () #where_clause {
          fn assert_impl<T: #trait_>() {}
          #(assert_impl::<#field_types>();)*
        }
      };
    }
}

pub trait GetGenerics {
    fn get_generics(&self) -> &Generics;
}

impl GetGenerics for Generics {
    fn get_generics(&self) -> &Generics {
        self
    }
}

macro_rules! get_generics {
    ($($item:ty),*) => {
        $(
            impl GetGenerics for $item {
                fn get_generics(&self) -> &Generics {
                    &self.generics
                }
            }
        )*
    };
}

get_generics!(DeriveInput, ItemStruct, ItemEnum);

pub trait FieldIter {
    fn field_iter(&self) -> impl Iterator<Item = &Field>;
}

impl FieldIter for Fields {
    fn field_iter(&self) -> impl Iterator<Item = &Field> {
        self.iter()
    }
}

impl FieldIter for Vec<Field> {
    fn field_iter(&self) -> impl Iterator<Item = &Field> {
        self.iter()
    }
}

macro_rules! field_iter {
    ($($item:ty),*) => {
        $(
            impl FieldIter for $item {
                fn field_iter(&self) -> impl Iterator<Item = &Field> {
                    self.fields.iter()
                }
            }
        )*
    };
}

field_iter!(DataStruct, ItemStruct);

impl FieldIter for DeriveInput {
    fn field_iter(&self) -> impl Iterator<Item = &Field> {
        match &self.data {
            Data::Struct(DataStruct { fields, .. }) => fields.iter(),
            Data::Union(DataUnion { fields, .. }) => fields.named.iter(),
            Data::Enum(_) => abort!(self, "cannot get fields on an enum"),
        }
    }
}

pub fn type_generic_idents<G: GetGenerics>(generics: &G) -> Vec<Ident> {
    generics
        .get_generics()
        .type_params()
        .map(|p| p.ident.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_strip_attributes_struct() {
        let mut struct_item: syn::ItemStruct = syn::parse_quote! {
            struct MyStruct {
                #[my_attr]
                field1: u8,
                field2: u8,
                #[my_attr(hello)]
                #[my_attr]
                field3: u8,
            }
        };
        let stripped_attributes: Vec<_> =
            strip_inner_attributes(&mut struct_item, "my_attr").collect();
        let expected_stripped_attributes = vec![
            StrippedAttribute {
                index: 0,
                attribute: syn::parse_quote! { #[my_attr] },
            },
            StrippedAttribute {
                index: 2,
                attribute: syn::parse_quote! { #[my_attr(hello)] },
            },
            StrippedAttribute {
                index: 2,
                attribute: syn::parse_quote! { #[my_attr] },
            },
        ];
        assert_eq!(stripped_attributes, expected_stripped_attributes);
        assert_eq!(
            struct_item,
            syn::parse_quote! {
                struct MyStruct {
                    field1: u8,
                    field2: u8,
                    field3: u8,
                }
            }
        );
    }

    #[test]
    fn test_strip_attributes_enum() {
        let mut enum_item: syn::ItemEnum = syn::parse_quote! {
            enum MyEnum {
                #[my_attr]
                Variant1,
                Variant2,
                #[my_attr(hello)]
                #[my_attr(hello2)]
                Variant3,
            }
        };
        let stripped_attributes: Vec<_> =
            strip_inner_attributes(&mut enum_item, "my_attr").collect();
        let expected_stripped_attributes = vec![
            StrippedAttribute {
                index: 0,
                attribute: syn::parse_quote! { #[my_attr] },
            },
            StrippedAttribute {
                index: 2,
                attribute: syn::parse_quote! { #[my_attr(hello)] },
            },
            StrippedAttribute {
                index: 2,
                attribute: syn::parse_quote! { #[my_attr(hello2)] },
            },
        ];
        assert_eq!(stripped_attributes, expected_stripped_attributes);
        assert_eq!(
            enum_item,
            syn::parse_quote! {
                enum MyEnum {
                    Variant1,
                    Variant2,
                    Variant3,
                }
            }
        );
    }
}
