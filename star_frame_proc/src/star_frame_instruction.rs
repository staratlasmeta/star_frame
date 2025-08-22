use proc_macro2::TokenStream;
use proc_macro_error2::abort;
use quote::{format_ident, quote};
use syn::{parse_quote, FnArg, ItemFn, ReturnType, Type, TypeReference};

use crate::util::{reject_generics, Paths};

pub fn star_frame_instruction_impl(mut input: ItemFn) -> TokenStream {
    Paths!(prelude);
    reject_generics(
        &input,
        Some("Generics are not supported for star_frame_instruction"),
    );

    let ident = input.sig.ident.clone();

    // TODO: Do we make the ident allowed to be snake case like normal functions?
    // let ident = format_ident!(
    // "{}",
    // input.sig.ident.to_string().to_pascal_case(),
    // span = input.sig.ident.span()
    // );
    input.sig.ident = format_ident!("run_instruction");

    let ReturnType::Type(_arrow, return_type) = &input.sig.output else {
        abort!(input.sig, "Expected a return type of `Result<T, E>`");
    };

    if input.sig.inputs.len() > 3 {
        abort!(
            input.sig,
            "Expected at most three arguments: account_set, run_arg, ctx"
        );
    }

    let mut input_iter = input.sig.inputs.clone().into_iter();

    let Some(FnArg::Typed(account_set)) = input_iter.next() else {
        abort!(input.sig, "Expected account_set argument");
    };

    let run_arg = input_iter
        .next()
        .unwrap_or_else(|| parse_quote!(_run_arg: Self::RunArg<'_>));
    let ctx = input_iter
        .next()
        .unwrap_or_else(|| parse_quote!(_ctx: &mut Context));

    let Type::Reference(TypeReference {
        mutability: Some(_),
        elem: account_set_type,
        ..
    }) = (*account_set.ty).clone()
    else {
        abort!(
            account_set,
            "Expected account_set to be of type `&mut MyAccountSet`"
        );
    };

    input.sig.inputs = parse_quote!(#account_set, #run_arg, #ctx);

    quote! {
        impl #prelude::StarFrameInstruction for #ident {
            type ReturnType = <#return_type as #prelude::IxReturnType>::ReturnType;
            type Accounts<'b, 'c> = #account_set_type;

            #input
        }
    }
}
