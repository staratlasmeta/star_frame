use easy_proc::ArgumentList;
use proc_macro2::TokenStream;
use proc_macro_error2::abort;
use quote::quote;
use syn::{parse_quote, Data, DeriveInput};

use crate::util::Paths;

#[derive(ArgumentList, Default, Debug)]
#[repr(C)]
struct ZeroCopyArgs {
    #[argument(presence)]
    pod: bool,
    #[argument(presence)]
    skip_packed: bool,
}

pub fn zero_copy_impl(input: DeriveInput, args: TokenStream) -> TokenStream {
    Paths!(bytemuck, copy, clone, prelude);

    let args = ZeroCopyArgs::parse_arguments(&parse_quote!(#[attribute(#args)]));

    if let Data::Union(union_data) = &input.data {
        abort!(
            union_data.union_token,
            "`#[zero_copy]` cannot be used on unions"
        );
    }

    let repr = if let Data::Enum(enum_data) = &input.data {
        if args.pod {
            abort!(
                enum_data.enum_token,
                "`#[zero_copy(pod)]` cannot be used on enums"
            );
        }
        if args.skip_packed {
            abort!(
                enum_data.enum_token,
                "`#[zero_copy(skip_packed)]` cannot be used on enums"
            );
        }
        quote!()
    } else {
        let packed = (!args.skip_packed).then(|| quote! { packed, });
        quote! { #[repr(C, #packed)] }
    };

    let remaining_derives = if args.pod {
        quote! { #bytemuck::Pod }
    } else {
        quote! { #bytemuck::CheckedBitPattern, #bytemuck::NoUninit }
    };

    quote! {
        #[derive(#copy, #clone, #prelude::Align1, #bytemuck::Zeroable, #remaining_derives)]
        #repr
        #input
    }
}
