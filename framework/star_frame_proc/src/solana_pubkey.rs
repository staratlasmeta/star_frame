use crate::get_crate_name;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Expr, LitByte, LitStr};

// Almost all code from here on is copied from solana-sdk-macro, with ::solana_program replaced with
// #crate_name to allow using this from star_frame without depending on solana_program directly

pub fn pubkey_impl(input: TokenStream) -> TokenStream {
    let id = parse_macro_input!(input as ProgramSdkPubkey);
    TokenStream::from(quote! {#id})
}

struct ProgramSdkPubkey(proc_macro2::TokenStream);

impl Parse for ProgramSdkPubkey {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let crate_name = get_crate_name();
        parse_id(
            input,
            quote! { #crate_name::solana_program::pubkey::Pubkey },
        )
        .map(Self)
    }
}

impl ToTokens for ProgramSdkPubkey {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let id = &self.0;
        tokens.extend(quote! {#id})
    }
}

fn parse_id(
    input: ParseStream,
    pubkey_type: proc_macro2::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
    let id = if input.peek(syn::LitStr) {
        let id_literal: LitStr = input.parse()?;
        parse_pubkey(&id_literal, &pubkey_type)?
    } else {
        let expr: Expr = input.parse()?;
        quote! { #expr }
    };

    if !input.is_empty() {
        let stream: proc_macro2::TokenStream = input.parse()?;
        return Err(syn::Error::new_spanned(stream, "unexpected token"));
    }
    Ok(id)
}

fn parse_pubkey(
    id_literal: &LitStr,
    pubkey_type: &proc_macro2::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
    let id_vec = bs58::decode(id_literal.value())
        .into_vec()
        .map_err(|_| syn::Error::new_spanned(id_literal, "failed to decode base58 string"))?;
    let id_array = <[u8; 32]>::try_from(<&[u8]>::clone(&&id_vec[..])).map_err(|_| {
        syn::Error::new_spanned(
            id_literal,
            format!("pubkey array is not 32 bytes long: len={}", id_vec.len()),
        )
    })?;
    let bytes = id_array.iter().map(|b| LitByte::new(*b, Span::call_site()));
    Ok(quote! {
        #pubkey_type::new_from_array(
            [#(#bytes,)*]
        )
    })
}
