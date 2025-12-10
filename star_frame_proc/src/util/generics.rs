use std::ops::{Deref, DerefMut};

use crate::util::Paths;
use itertools::Itertools;
use proc_macro2::Span;
use proc_macro_error2::abort;
use quote::{format_ident, quote, ToTokens};
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    token, Attribute, ConstParam, DeriveInput, GenericParam, Generics, Ident, ItemEnum, ItemFn,
    ItemStruct, Lifetime, LifetimeParam, Token, Type, TypeParam, WhereClause,
};

#[derive(Debug, Clone, Default)]
pub struct BetterGenerics {
    _bracket: token::Bracket,
    pub generics: Generics,
}

impl Deref for BetterGenerics {
    type Target = Generics;
    fn deref(&self) -> &Self::Target {
        &self.generics
    }
}

impl DerefMut for BetterGenerics {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.generics
    }
}

impl ToTokens for BetterGenerics {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.generics.to_tokens(tokens)
    }
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

pub trait GetGenerics {
    fn get_generics(&self) -> &Generics;
}

impl GetGenerics for Generics {
    fn get_generics(&self) -> &Generics {
        self
    }
}

impl GetGenerics for ItemFn {
    fn get_generics(&self) -> &Generics {
        &self.sig.generics
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

get_generics!(DeriveInput, ItemStruct, ItemEnum, BetterGenerics);

pub trait CombineGenerics {
    fn combine<G: GetGenerics>(&self, other: &G) -> Self;
}

macro_rules! combine_gen {
    ($generic:expr; $($other:tt)*) => {
        $crate::util::CombineGenerics::combine::<$crate::util::BetterGenerics>(&$generic, &parse_quote!([$($other)*]))
    };
}
pub(crate) use combine_gen;

impl CombineGenerics for Generics {
    fn combine<G: GetGenerics>(&self, other: &G) -> Self {
        let other = other.get_generics().clone();
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

pub fn new_ident<'s, 'i>(
    ident_start: &'s str,
    existing: impl Iterator<Item = &'i Ident>,
    prepend: bool,
) -> Ident {
    let mut new_ident = ident_start.to_string();
    let existing = existing.map(|i| i.to_string()).collect_vec();
    while existing.iter().any(|g| g == &new_ident) {
        if prepend {
            new_ident.insert(0, '_');
        } else {
            new_ident.push('_');
        }
    }
    Ident::new(&new_ident, Span::call_site())
}

pub fn new_lifetime<G: GetGenerics>(generics: &G, lifetime_str: Option<&str>) -> Lifetime {
    let existing = generics
        .get_generics()
        .lifetimes()
        .map(|l| &l.lifetime.ident);
    let new_lifetime = new_ident(lifetime_str.unwrap_or("a"), existing, false);
    Lifetime::new(&format!("'{new_lifetime}"), Span::call_site())
}

pub fn new_generic<G: GetGenerics>(generics: &G, generic_str: Option<&str>) -> Ident {
    let generic_str = generic_str.unwrap_or("A");
    let generics = generics.get_generics();
    let type_idents = generics.type_params().map(|t| &t.ident);
    let const_idents = generics.const_params().map(|c| &c.ident);
    new_ident(generic_str, type_idents.chain(const_idents), false)
}

pub fn reject_generics(item: &impl GetGenerics, error: Option<&str>) {
    let generics = item.get_generics();
    if !generics.params.is_empty() {
        abort!(generics, error.unwrap_or("Generics are not supported"));
    }
}

pub fn phantom_generics_ident() -> Ident {
    format_ident!("_generics")
}

pub fn phantom_generics_type(item: &impl GetGenerics) -> Option<Type> {
    Paths!(phantom_data, box_ty);
    let generics = item.get_generics();
    if generics.params.is_empty() {
        return None;
    }
    let type_params = generics.type_params().map(|p| p.ident.to_token_stream());
    let lifetime_tys = generics.lifetimes().map(|p| {
        let lifetime = &p.lifetime;
        quote! {&#lifetime ()}
    });
    let tys = type_params.chain(lifetime_tys).collect_vec();
    Some(parse_quote! {
        #phantom_data<fn() -> (#(#box_ty<#tys>),*)>
    })
}
