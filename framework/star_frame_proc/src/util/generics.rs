use crate::util::Paths;
use derive_more::{Deref, DerefMut};
use itertools::Itertools;
use proc_macro2::Span;
use proc_macro_error2::abort;
use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{
    bracketed, parse_quote, token, Attribute, ConstParam, DeriveInput, GenericParam, Generics,
    Ident, ItemEnum, ItemStruct, Lifetime, LifetimeParam, Token, Type, TypeParam, WhereClause,
};

#[derive(Debug, Deref, DerefMut, Clone, Default)]
pub struct BetterGenerics {
    _bracket: token::Bracket,
    #[deref]
    #[deref_mut]
    pub generics: Generics,
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

#[allow(dead_code)]
pub fn new_lifetime<G: GetGenerics>(generics: &G) -> Lifetime {
    let mut lifetime = "l".to_string();
    while generics
        .get_generics()
        .lifetimes()
        .map(|l| l.lifetime.ident.to_string())
        .any(|l| l == lifetime)
    {
        lifetime.push('_');
    }
    Lifetime::new(&format!("'{lifetime}"), Span::call_site())
}

fn new_generic_inner<G: GetGenerics>(generics: &G, extra_idents: &[Ident]) -> Ident {
    let generics = generics.get_generics();
    let type_idents = generics
        .type_params()
        .map(|t| t.ident.clone())
        .collect::<Vec<_>>();
    let const_idents = generics
        .const_params()
        .map(|c| c.ident.clone())
        .collect::<Vec<_>>();
    let mut new_generic = "A".to_string();
    while type_idents
        .iter()
        .chain(const_idents.iter())
        .chain(extra_idents.iter())
        .map(ToString::to_string)
        .any(|g| g == new_generic)
    {
        new_generic.push('_');
    }
    format_ident!("{new_generic}")
}

pub fn new_generic<G: GetGenerics>(generics: &G) -> Ident {
    new_generic_inner(generics, &[])
}

pub fn new_generics<G: GetGenerics, const N: usize>(generics: &G) -> [Ident; N] {
    let mut idents = Vec::with_capacity(N);
    for _ in 0..N {
        idents.push(new_generic_inner(generics, &idents));
    }
    idents
        .try_into()
        .expect("idents should be of the same length")
}

pub fn type_generic_idents<G: GetGenerics>(generics: &G) -> Vec<Ident> {
    generics
        .get_generics()
        .type_params()
        .map(|p| p.ident.clone())
        .collect()
}

pub fn reject_generics(item: &impl GetGenerics, error: Option<&str>) {
    let generics = item.get_generics();
    if !generics.params.is_empty() {
        abort!(generics, error.unwrap_or("Generics are not supported"));
    }
}

pub fn phantom_generics_ident() -> Ident {
    format_ident!("__generics")
}

pub fn phantom_generics_type(item: &impl GetGenerics) -> Option<Type> {
    let phantom_data = Paths::default().phantom_data;
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
        #phantom_data<fn() -> (#(#tys),*)>
    })
}
