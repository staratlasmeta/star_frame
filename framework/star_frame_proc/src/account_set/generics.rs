use crate::util::{new_generic, new_lifetime};
use proc_macro2::{Ident, Span};
use std::collections::HashMap;
use syn::{GenericParam, Generics, Lifetime, LifetimeParam};

#[derive(Debug)]
pub struct AccountSetGenerics {
    pub main_generics: Generics,
    pub other_generics: Generics,
    pub decode_generics: Generics,
    pub info_lifetime: Lifetime,
    pub decode_lifetime: Lifetime,
    pub function_lifetime: Lifetime,
    pub function_generic_type: Ident,
}

pub fn account_set_generics(generics: Generics) -> AccountSetGenerics {
    let mut lifetimes = generics
        .lifetimes()
        .map(|l| (l.lifetime.ident.to_string(), l.clone()))
        .collect::<HashMap<_, _>>();
    let mut add_info = false;
    let info_lifetime = lifetimes
        .entry("info".to_string())
        .or_insert_with(|| {
            add_info = true;
            LifetimeParam::new(Lifetime::new("'info", Span::call_site()))
        })
        .clone();
    let function_lifetime = new_lifetime(&generics);
    let function_generic_type = new_generic(&generics);
    let mut decode_lifetimes = lifetimes.clone();
    let mut add_decode = false;
    let decode_lifetime = decode_lifetimes
        .entry("a".to_string())
        .or_insert_with(|| {
            add_decode = true;
            LifetimeParam::new(Lifetime::new("'a", Span::call_site()))
        })
        .clone();

    let mut other_generics = generics.clone();
    let mut decode_generics = generics.clone();
    if add_info {
        other_generics
            .params
            .push(GenericParam::Lifetime(info_lifetime.clone()));
        decode_generics
            .params
            .push(GenericParam::Lifetime(info_lifetime.clone()));
    }
    if add_decode {
        decode_generics
            .params
            .push(GenericParam::Lifetime(decode_lifetime.clone()));
    }

    AccountSetGenerics {
        main_generics: generics,
        other_generics,
        decode_generics,
        info_lifetime: info_lifetime.lifetime,
        decode_lifetime: decode_lifetime.lifetime,
        function_lifetime,
        function_generic_type,
    }
}
