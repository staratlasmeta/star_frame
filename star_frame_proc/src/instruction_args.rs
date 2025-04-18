use derive_more::Debug;
use easy_proc::{find_attr, ArgumentList};
use proc_macro2::TokenStream;
use proc_macro_error2::abort;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    DeriveInput, Expr, Ident, Lifetime, Token, Type,
};

use crate::{
    idl::derive_instruction_to_idl,
    util::{ensure_data_struct, new_lifetime, reject_generics, Paths},
};

#[derive(Debug)]
enum InstructionArgType {
    Decode,
    Validate,
    Run,
    Cleanup,
}

impl InstructionArgType {
    fn ident_str(&self) -> &'static str {
        match self {
            InstructionArgType::Decode => "decode",
            InstructionArgType::Validate => "validate",
            InstructionArgType::Run => "run",
            InstructionArgType::Cleanup => "cleanup",
        }
    }
}
impl Parse for InstructionArgType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        Ok(match ident.to_string().as_str() {
            "decode" => InstructionArgType::Decode,
            "validate" => InstructionArgType::Validate,
            "run" => InstructionArgType::Run,
            "cleanup" => InstructionArgType::Cleanup,
            _ => {
                return Err(input.error(
                    "Invalid instruction arg type. Must be one of: decode, validate, run, cleanup",
                ))
            }
        })
    }
}

enum RefKind {
    Ref,
    RefMut,
    Owned,
}

impl Parse for RefKind {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.parse::<Option<Token![&]>>()?.is_none() {
            Ok(RefKind::Owned)
        } else if input.parse::<Option<Token![mut]>>()?.is_some() {
            Ok(RefKind::RefMut)
        } else {
            Ok(RefKind::Ref)
        }
    }
}

struct InstructionArg {
    reference: RefKind,
    arg_type: InstructionArgType,
}

impl Parse for InstructionArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(InstructionArg {
            reference: input.parse()?,
            arg_type: input.parse()?,
        })
    }
}

impl InstructionArg {
    fn info(&self, attribute_type: AttributeType, lt: &Lifetime) -> ArgInfo {
        match attribute_type {
            AttributeType::Struct(ident) => match self.reference {
                RefKind::RefMut => (parse_quote! { &#lt mut #ident }, parse_quote! { r }),
                RefKind::Ref => (parse_quote! { &#lt #ident }, parse_quote! { &*r }),
                RefKind::Owned => (parse_quote! { #ident }, parse_quote! { *r }),
            },
            AttributeType::Field(ident, ty) => match self.reference {
                RefKind::RefMut => (
                    parse_quote! { &#lt mut #ty },
                    parse_quote! { &mut r.#ident },
                ),
                RefKind::Ref => (parse_quote! { &#lt #ty }, parse_quote! { &r.#ident }),
                RefKind::Owned => (parse_quote! { #ty }, parse_quote! { r.#ident }),
            },
        }
    }
}

#[derive(Copy, Clone)]
enum AttributeType<'a> {
    Struct(&'a Ident),
    Field(&'a Ident, &'a Type),
}

type ArgInfo = (Type, Expr);

#[derive(ArgumentList, Default)]
struct InstructionArgsArgs {
    #[argument(presence)]
    skip_idl: bool,
}

fn idl_impl(input: &DeriveInput) -> Option<TokenStream> {
    Paths!(instruction_args_ident);
    let args = find_attr(&input.attrs, &instruction_args_ident)
        .map(InstructionArgsArgs::parse_arguments)
        .unwrap_or_default();

    (!args.skip_idl).then(|| derive_instruction_to_idl(input))
}

pub fn derive_instruction_args_impl(input: DeriveInput) -> TokenStream {
    Paths!(ix_args_ident, prelude);
    let ident = &input.ident;

    reject_generics(
        &input,
        Some("Generics are not supported for InstructionArgs"),
    );

    let data_struct = ensure_data_struct(
        &input,
        Some("InstructionArgs can only be derived for structs"),
    );

    let lt = new_lifetime(&input.generics, None);

    let default_type: ArgInfo = (parse_quote! {()}, parse_quote! {()});
    let mut decode: Option<ArgInfo> = None;
    let mut validate: Option<ArgInfo> = None;
    let mut run: Option<ArgInfo> = None;
    let mut cleanup: Option<ArgInfo> = None;

    let mut handle_attrs = |attrs: &[syn::Attribute],
                            attribute_type: AttributeType,
                            lt: &Lifetime| {
        let attr = find_attr(attrs, &ix_args_ident);
        if let Some(args) = attr
            .map(|attr| {
                attr.parse_args_with(Punctuated::<InstructionArg, Token![,]>::parse_terminated).unwrap_or_else(|_| {
                    abort!(attr, "Attribute must be of the form `#[ix_args(decode, validate, run, cleanup)]`, optionaly with `&` or `&mut` to the argument. Any of the args can be provided.")
                })
            }) {
                for arg in args {
                    let info = arg.info(attribute_type, lt);
                    let arg_to_replace = match arg.arg_type {
                        InstructionArgType::Decode => &mut decode,
                        InstructionArgType::Validate => &mut validate,
                        InstructionArgType::Run => &mut run,
                        InstructionArgType::Cleanup => &mut cleanup,
                    };
                    let old = arg_to_replace.replace(info);
                    if old.is_some() {
                        abort!(attr, "Duplicate instruction arg type: {}", arg.arg_type.ident_str());
                    }
                }
            }
    };

    handle_attrs(&input.attrs, AttributeType::Struct(&input.ident), &lt);

    for (i, field) in data_struct.fields.iter().enumerate() {
        let ident = field.ident.clone().unwrap_or_else(|| format_ident!("{i}"));
        handle_attrs(&field.attrs, AttributeType::Field(&ident, &field.ty), &lt);
    }

    let (decode_ty, decode_expr) = decode.unwrap_or_else(|| default_type.clone());
    let (validate_ty, validate_expr) = validate.unwrap_or_else(|| default_type.clone());
    let (run_ty, run_expr) = run.unwrap_or_else(|| default_type.clone());
    let (cleanup_ty, cleanup_expr) = cleanup.unwrap_or_else(|| default_type.clone());

    let idl_impl = idl_impl(&input);

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote! {
        #idl_impl

        impl #impl_generics #prelude::InstructionArgs for #ident #ty_generics #where_clause {
            type DecodeArg<#lt> = #decode_ty;
            type ValidateArg<#lt> = #validate_ty;
            type RunArg<#lt> = #run_ty;
            type CleanupArg<#lt> = #cleanup_ty;

            fn split_to_args(r: &mut Self) -> #prelude::IxArgs<Self> {
                #prelude::IxArgs {
                    decode: #decode_expr,
                    validate: #validate_expr,
                    run: #run_expr,
                    cleanup: #cleanup_expr,
                }
            }
        }
    }
}
