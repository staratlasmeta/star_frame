use anyhow::{anyhow, Result};
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use star_frame_idl::verifier::verify_idl_definitions;
use star_frame_idl::IdlDefinition;
use std::env::current_dir;
use std::fs::File;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{bracketed, parse_macro_input, LitStr, Token};

#[proc_macro_error]
#[proc_macro]
pub fn cpi_for_idl(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as CpiForIdl);
    let idls = input
        .files
        .read_files()
        .unwrap_or_else(|e| abort!("Error parsing idls: {}", e));
    println!("idls: {:#?}", idls);

    verify_idl_definitions(&idls).unwrap_or_else(|e| abort!("Error verifying idls: {}", e));

    (quote! {}).into()
}

mod tokens {
    use syn::custom_keyword;
    custom_keyword!(files);
}

// cpi_for_idl!(files = ["path/to/idl.json", "path/to/idl2.json"]);
struct CpiForIdl {
    files: IdlFiles,
}
impl Parse for CpiForIdl {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            files: input.parse()?,
        })
    }
}

// files = ["path/to/idl.json", "path/to/idl2.json"]
struct IdlFiles {
    _keyword: tokens::files,
    _eq: Token![=],
    _bracket: syn::token::Bracket,
    files: Punctuated<LitStr, Token![,]>,
}
impl Parse for IdlFiles {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _keyword = input.parse()?;
        let _eq = input.parse()?;
        let content;
        let _bracket = bracketed!(content in input);
        let files = content.parse_terminated(Parse::parse)?;
        Ok(Self {
            _keyword,
            _eq,
            _bracket,
            files,
        })
    }
}
impl IdlFiles {
    fn read_files(&self) -> Result<Vec<IdlDefinition>> {
        self.files
            .iter()
            .map(|f| {
                let path = current_dir()?.join(f.value());
                let file = File::open(f.value())
                    .map_err(|e| anyhow!("Could not open path {path:?}: {e}"))?;
                Ok(serde_json::from_reader(file)?)
            })
            .collect()
    }
}
