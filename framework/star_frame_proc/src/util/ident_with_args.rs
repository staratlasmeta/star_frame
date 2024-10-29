use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::token::Token;
use syn::{parenthesized, token};

pub struct IdentWithArgs<A> {
    pub ident: Ident,
    pub args: Option<IdentArg<A>>,
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
pub struct IdentArg<A> {
    pub paren: token::Paren,
    pub arg: Option<A>,
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
