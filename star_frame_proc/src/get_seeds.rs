use crate::util::{get_docs, ignore_cfg_module, Paths};
use easy_proc::{find_attr, ArgumentList};
use proc_macro2::TokenStream;
use proc_macro_error2::abort;
use quote::{format_ident, quote};
use syn::{parse_quote, Data, DeriveInput, Expr};

#[derive(Debug, ArgumentList, Default)]
pub struct GetSeedsArgs {
    // todo: rename with `r#const` once ArgumentList supports it.
    pub seed_const: Option<Expr>,
    #[argument(presence)]
    pub skip_idl: bool,
}

pub fn derive_get_seeds_impl(input: DeriveInput) -> TokenStream {
    let data_struct = match input.data {
        Data::Struct(s) => s,
        Data::Enum(e) => abort!(e.enum_token, "GetSeeds cannot be derived for enums"),
        Data::Union(u) => abort!(u.union_token, "GetSeeds cannot be derived for unions"),
    };

    let Paths {
        get_seeds_ident,
        result,
        prelude,
        ..
    } = Paths::default();

    let GetSeedsArgs {
        skip_idl,
        seed_const,
    } = find_attr(&input.attrs, &get_seeds_ident)
        .map(GetSeedsArgs::parse_arguments)
        .unwrap_or_default();

    let ident = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    if matches!(data_struct.fields, syn::Fields::Unnamed(_)) {
        abort!(
            data_struct.fields,
            "GetSeeds cannot be derived for tuple structs"
        );
    }

    let idl_impl = (!skip_idl).then(|| {
        let seeds_to_idl = {
            let mut generics = input.generics.clone();
            let where_clause = generics.make_where_clause();
            let field_seeds: Vec<_> = data_struct
                .fields
                .iter()
                .map(|field| {
                    let ty = &field.ty;
                    let docs = get_docs(&field.attrs);
                    let ident = field
                        .ident
                        .clone()
                        .expect("Field must have an identifier")
                        .to_string();
                    where_clause.predicates.push(parse_quote! {
                        #ty: #prelude::TypeToIdl
                    });
                    quote! {
                        #prelude::IdlSeed::Variable {
                            name: #ident.to_string(),
                            description: #docs,
                            ty: <#ty as #prelude::TypeToIdl>::type_to_idl(idl_definition)?,
                        }
                    }
                })
                .collect();
            let idl_seeds = seed_const
                .as_ref()
                .map(|expr| quote!(#prelude::IdlSeed::Const(#expr.to_vec())))
                .into_iter()
                .chain(field_seeds);

            quote! {
                #[cfg(all(feature = "idl", not(target_os = "solana")))]
                #[automatically_derived]
                impl #impl_generics #prelude::SeedsToIdl for #ident #type_generics #where_clause {
                    fn seeds_to_idl(idl_definition: &mut #prelude::IdlDefinition) -> #result<#prelude::IdlSeeds> {
                        Ok(#prelude::IdlSeeds(vec![
                            #(#idl_seeds),*
                        ]))
                    }
                }
            }
        };

        let find_seeds = {
            let find_seeds_ident = format_ident!("Find{ident}");

            let field_find_seeds: Vec<_> = data_struct
                .fields
                .iter()
                .map(|field| {
                    let ident = field.ident.as_ref().expect("Field must have an identifier");
                    quote! {
                        Into::into(&self.#ident)
                    }
                })
                .collect();
            let find_seeds = seed_const
                .as_ref()
                .map(|expr| parse_quote!(#prelude::IdlFindSeed::Const(#expr.to_vec())))
                .into_iter()
                .chain(field_find_seeds);

            let find_fields = data_struct.fields.iter().map(|field| {
                let mut field = field.clone();
                let ty = &field.ty;
                field.vis = parse_quote!(pub);
                field.ty = parse_quote!(#prelude::FindSeed<#ty>);
                field
            });

            quote! {
                #[cfg(all(feature = "idl", not(target_os = "solana")))]
                #[derive(Debug, Clone)]
                pub struct #find_seeds_ident #type_generics #where_clause {
                    #(#find_fields),*
                }

                #[cfg(all(feature = "idl", not(target_os = "solana")))]
                #[automatically_derived]
                impl #impl_generics #prelude::FindIdlSeeds for #find_seeds_ident #type_generics #where_clause {
                    fn find_seeds(&self) -> #result<Vec<#prelude::IdlFindSeed>> {
                        Ok(vec![#(#find_seeds),*])
                    }
                }
            }
        };

        ignore_cfg_module(ident, "_get_seeds_idl", quote! {
            #seeds_to_idl
            #find_seeds
        })
    });

    let field_seeds = data_struct.fields.iter().map(|field| {
        let name = field.ident.as_ref().expect("Field must have an identifier");
        parse_quote!(self.#name.seed())
    });
    let seeds = seed_const
        .into_iter()
        .chain(field_seeds)
        .chain(std::iter::once(parse_quote!(&[])));

    quote! {
        #[automatically_derived]
        impl #impl_generics #prelude::GetSeeds for #ident #type_generics #where_clause {
            fn seeds(&self) -> Vec<&[u8]> {
                use #prelude::Seed;
                vec![#(#seeds),*]
            }
        }

        #idl_impl
    }
}
