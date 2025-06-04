use proc_macro2::Span;
use std::collections::HashMap;
use syn::{GenericParam, Generics, Lifetime, LifetimeParam};

#[derive(Debug)]
pub struct AccountSetGenerics {
    pub main_generics: Generics,
    pub decode_generics: Generics,
    pub decode_lifetime: Lifetime,
}

pub fn account_set_generics(generics: Generics) -> AccountSetGenerics {
    let lifetimes = generics
        .lifetimes()
        .map(|l| (l.lifetime.ident.to_string(), l.clone()))
        .collect::<HashMap<_, _>>();
    let mut decode_lifetimes = lifetimes.clone();
    let mut add_decode = false;
    let decode_lifetime = decode_lifetimes
        .entry("a".to_string())
        .or_insert_with(|| {
            add_decode = true;
            LifetimeParam::new(Lifetime::new("'a", Span::call_site()))
        })
        .clone();

    let mut decode_generics = generics.clone();

    if add_decode {
        decode_generics
            .params
            .push(GenericParam::Lifetime(decode_lifetime.clone()));
    }

    AccountSetGenerics {
        main_generics: generics,
        decode_generics,
        decode_lifetime: decode_lifetime.lifetime,
    }
}
