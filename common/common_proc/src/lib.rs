//! Proc macros for the common utils library.
#![warn(clippy::pedantic, missing_docs)]
#![allow(
    clippy::wildcard_imports,
    clippy::module_name_repetitions,
    clippy::too_many_lines
)]

mod enum_refs;
mod strong_typed_struct;
mod unit_enum_from_repr;
mod unpackable;
mod zero_copy_checks;

use crate::enum_refs::enum_refs_impl;
use crate::strong_typed_struct::strong_typed_struct_impl;
use crate::unit_enum_from_repr::unit_enum_from_repr_impl;
use crate::unpackable::unpackable_impl;
use crate::zero_copy_checks::{zero_copy_checks, ZeroCopyType};
use proc_macro2::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use proc_macro_error::{abort, proc_macro_error};
use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::token::Token;
use syn::*;

fn get_crate_name() -> TokenStream {
    let generator_crate = crate_name("common_utils").expect("Could not find `common_utils`");
    match generator_crate {
        FoundCrate::Itself => quote! { common_utils },
        FoundCrate::Name(name) => {
            let ident = format_ident!("{}", name);
            quote! { ::#ident }
        }
    }
}

/// Similar to strum's `FromRepr` derive but includes a trait for generic implementations and does not support non-unit enums.
#[proc_macro_error]
#[proc_macro_derive(UnitEnumFromRepr)]
pub fn unit_enum_from_repr(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let out = unit_enum_from_repr_impl(parse_macro_input!(input as DeriveInput));
    // println!("{}", out);
    out.into()
}

/// Adds functions to get the strongly typed version of a `SafeZeroCopy` struct.
/// Currently supports the following features:
/// - `strong_sub_struct`: Sub type that is also `StrongTypedStruct`.
/// - `fixed_point`: Fixed point integer type with unit library.
/// - `key_for`: `Pubkey` wrapper that type checks to be a certain account type.
/// - `optional_key_for`: `Option<Pubkey>` (or `OptionalNonSystemPubkey`) wrapper that type
/// checks to be a certain account type.
/// - `enum_wrapper`: Wrapper for an enum that is stored using its repr.
///
/// All type conversions are zero runtime cost due to being pointer casts.
///
/// ```
/// # use anchor_lang::{Discriminator, ZeroCopy};
/// # use bytemuck::{Pod, Zeroable};
/// use common_utils::prelude::*;
/// # declare_id!("22222222222222222222222222222222222222222222");
///
/// define_unit! {
///     /// Defines a custom unit, in this case length in meters.
///     Meter;
///     /// Another custom unit, in this case time in seconds.
///     Second;
/// }
///
/// // Account traits omitted due to anchor issues.
/// /// A standard Anchor account.
/// # #[derive(Debug, Zeroable, Pod, Copy, Clone)]
/// # #[repr(C)]
/// struct TestAccount;
/// # impl DataSize for TestAccount{
/// #     const MIN_DATA_SIZE: usize = 0;
/// # }
/// # unsafe impl SafeZeroCopy for TestAccount{}
/// # impl Owner for TestAccount {
/// #     fn owner() -> Pubkey {
/// #         id()
/// #     }
/// # }
/// # impl Discriminator for TestAccount {
/// #     fn discriminator() -> [u8; 8] {
/// #         [1; 8]
/// #     }
/// # }
/// # impl ZeroCopy for TestAccount{}
///
///
/// /// An enum that can be converted to its repr using [`UnitEnumFromRepr`].
/// #[derive(UnitEnumFromRepr, Copy, Clone, Debug, PartialEq)]
/// #[repr(u16)]
/// enum TestEnum { A, B, C }
///
/// #[safe_zero_copy]
/// #[zero_copy]
/// #[derive(StrongTypedStruct)]
/// struct SubStruct{
///     /// Defines that this value is in 100ths of meters.
///     #[fixed_point(100, Meter)]
///     sub_val: u64,
///     /// Defines that this value represents the enum [`TestEnum`].
///     /// The type is [`u16`] because the enum is `#[repr(u16)]`.
///     #[enum_wrapper(TestEnum)]
///     sub_enum_val: u16
/// }
///
/// #[safe_zero_copy]
/// #[zero_copy]
/// #[derive(StrongTypedStruct)]
/// struct TestStruct {
///     /// Defines that this value is in 100ths of meters per second.
///     /// We use [`DivUnit`] here to combine 2 units, [`MulUnit`] is also available.
///     #[fixed_point(100, DivUnit<Meter, Second>)]
///     test_val: u32,
///     /// This field is treated as another struct that implements [`StrongTypedStruct`].
///     #[strong_sub_struct]
///     sub_struct: SubStruct,
///     /// Defines that this value is a [`Pubkey`] for a [`TestAccount`].
///     #[key_for(TestAccount)]
///     account_key: Pubkey,
///     /// Same as `account_key` but can is optional with `None` being the system program ie `[0; 32]`.
///     #[optional_key_for(TestAccount)]
///     optional_account_key: Pubkey,
///     #[bool_wrapper]
///     test_bool: u8,
/// }
///
/// let test_struct = TestStruct {
///     test_val: 100,
///     sub_struct: SubStruct {
///         sub_val: 200,
///         sub_enum_val: 1,
///     },
///     account_key: Pubkey::new_unique(),
///     optional_account_key: System::id(),
///     test_bool:0,
/// };
/// // The [`StrongTypedStruct::as_strong_typed`] function converts a reference of the original
/// // struct to a reference of the strongly typed struct.
/// let strong_typed_test_struct: &<TestStruct as StrongTypedStruct>::StrongTyped = test_struct.as_strong_typed();
///
/// // Fixed point values are stored with their unit and divisor.
/// let test_val: FixedPointU32<DivUnit<Meter, Second>, 100> = strong_typed_test_struct.test_val;
/// // Fixed point values can be turned into their float representation.
/// let test_val_float: FloatWithUnit<DivUnit<Meter, Second>> = test_val.to_float();
///
/// let sub_val: FixedPointU64<Meter, 100> = strong_typed_test_struct.sub_struct.sub_val;
/// let sub_val_float: FloatWithUnit<Meter> = sub_val.to_float();
/// // Units can be type checked by using the proper functions (many available).
/// let seconds_float: FloatWithUnit<Second> = sub_val_float.div_and_cancel(test_val_float);
/// // Can be converted back to fixed.
/// let seconds: FixedPointU64<Second, 1000> = seconds_float.to_fixed_u64();
/// assert_eq!(seconds.to_raw(), 2000);
///
/// // Keys type-check that they only compare with the correct account type.
/// // This is achieved through implementations of [`PartialEq`] for [`KeyFor`] and [`OptionalKeyFor`]
/// // against anchor account types.
/// // They can also be set via standard anchor account types.
/// let account_key: KeyFor<TestAccount> = strong_typed_test_struct.account_key;
/// let optional_account_key: OptionalKeyFor<TestAccount> = strong_typed_test_struct.optional_account_key;
///
/// // Account example omitted due to anchor issues.
///
/// assert_eq!(*account_key.pubkey(), test_struct.account_key);
/// assert_eq!(optional_account_key.pubkey(), None);
///
/// // Enums are wrapped
/// let sub_enum_val_wrapped: UnitEnumWrapper<TestEnum> = strong_typed_test_struct.sub_struct.sub_enum_val;
/// // This wrapper allows access to the enum value.
/// // This will error if the value is invalid.
/// let sub_enum_val: TestEnum = sub_enum_val_wrapped.enum_value().unwrap();
/// assert_eq!(sub_enum_val, TestEnum::B);
#[proc_macro_error]
#[proc_macro_derive(
    StrongTypedStruct,
    attributes(
        strong_sub_struct,
        fixed_point,
        key_for,
        optional_key_for,
        enum_wrapper,
        bool_wrapper
    )
)]
pub fn strong_typed_struct(derive_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let out = strong_typed_struct_impl(parse_macro_input!(derive_input as DeriveInput));
    // println!("{}", out);
    out.into()
}

/// Provides extra checks and impls for anchor's `zero_copy` attribute.
#[proc_macro_error]
#[proc_macro_attribute]
pub fn safe_zero_copy(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    zero_copy_checks(args, input.into(), ZeroCopyType::Struct)
}

/// Provides extra checks and impls for anchor's `account(zero_copy)` attribute.
#[proc_macro_error]
#[proc_macro_attribute]
pub fn safe_zero_copy_account(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    zero_copy_checks(args, input.into(), ZeroCopyType::Account)
}

/// Creates two new enums that are the same as the attributed one but with [`Ref`](std::cell::Ref)
/// and [`RefMut`](std::cell::RefMut) internals rather than owned.
#[proc_macro_error]
#[proc_macro_attribute]
pub fn enum_refs(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    enum_refs_impl(args, input)
}

/// Creates a new struct that is not packed and implements `AnchorSerialize` and `AnchorDeserialize`.
#[proc_macro_error]
#[proc_macro_derive(Unpackable, attributes(unpackable, packed_sub_struct))]
pub fn unpackable(derive_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let out = unpackable_impl(parse_macro_input!(derive_input as DeriveInput));
    // println!("{}", out);
    out.into()
}

struct IdentWithArgs<A> {
    ident: Ident,
    args: Option<IdentArg<A>>,
}
impl<A> Parse for IdentWithArgs<A>
where
    A: Parse + Token,
{
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            ident: input.parse()?,
            args: if input.peek(token::Paren) {
                Some(input.parse()?)
            } else {
                None
            },
        })
    }
}
impl<A> ToTokens for IdentWithArgs<A>
where
    A: ToTokens,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ident.to_tokens(tokens);
        self.args.to_tokens(tokens);
    }
}

struct IdentArg<A> {
    paren: token::Paren,
    arg: Option<A>,
}
impl<A> Parse for IdentArg<A>
where
    A: Parse + Token,
{
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;

        Ok(Self {
            paren: parenthesized!(content in input),
            arg: content.parse()?,
        })
    }
}
impl<A> ToTokens for IdentArg<A>
where
    A: ToTokens,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.paren.surround(tokens, |tokens| {
            self.arg.to_tokens(tokens);
        });
    }
}

/// Derives `Align1` for a valid type.
#[proc_macro_error]
#[proc_macro_derive(Align1)]
pub fn derive_align1(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let crate_name = get_crate_name();
    let derive_input = parse_macro_input!(item as DeriveInput);
    match derive_input.data.clone() {
        Data::Struct(DataStruct { fields, .. }) => {
            derive_align1_for_struct(fields, derive_input, &crate_name)
        }
        Data::Union(DataUnion { fields, .. }) => {
            derive_align1_for_struct(Fields::Named(fields), derive_input, &crate_name)
        }
        Data::Enum(e) => {
            // TODO: Derive for repr u8 and unit enums
            for variant in e.variants {
                if variant.fields != Fields::Unit {
                    abort!(variant.fields, "Align1 only supports unit enums");
                }
            }

            abort!(e.enum_token, "Align1 cannot be derived for enums");
        }
    }
}

fn derive_align1_for_struct(
    fields: Fields,
    derive_input: DeriveInput,
    crate_name: &TokenStream,
) -> proc_macro::TokenStream {
    let packed = derive_input.attrs.into_iter().any(|attr| {
        attr.path.is_ident("repr") && {
            let Ok(args) = attr.parse_args_with(|p: ParseStream| {
                p.parse_terminated::<_, Token![,]>(IdentWithArgs::<LitInt>::parse)
            }) else { abort!(attr, "Repr invalid args") };
            // args.iter().any(|arg|arg.ident.to_string() == "packed" && {
            //     if let Some(num) = arg.args {
            //
            //     }
            // });
            for arg in args {
                let ident = arg.ident.to_string();
                let arg = arg.args.as_ref().and_then(|a| a.arg.as_ref());
                if &ident == "align" && arg.map_or(false, |align| &align.to_string() != "1") {
                    abort!(arg, "`align` argument must be 1 to implement `Align1`");
                }
                if &ident == "packed" {
                    if arg.map_or(false, |align| &align.to_string() != "1") {
                        abort!(
                            arg,
                            "`packed` argument must be 1 or not present to implement `Align1`"
                        );
                    } else {
                        return true;
                    }
                }
            }
            false
        }
    });

    let ident = derive_input.ident;

    let mut gen = derive_input.generics;
    let wc = gen.make_where_clause();
    if !packed {
        for field in fields {
            let ty = field.ty;
            wc.predicates
                .push(parse_quote!(#ty: #crate_name::align1::Align1));
        }
    }
    let (impl_gen, type_gen, where_clause) = gen.split_for_impl();

    (quote! {
        unsafe impl #impl_gen #crate_name::align1::Align1 for #ident #type_gen #where_clause {}
    })
    .into()
}
