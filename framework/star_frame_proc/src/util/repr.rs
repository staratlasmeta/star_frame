// Original code from https://github.com/Lokathor/bytemuck/blob/79a15d0a3f7f6eeaf816e05a0420c04d92be9ff5/derive/src/traits.rs, licensed under the MIT License.
// See https://github.com/Lokathor/bytemuck/blob/79a15d0a3f7f6eeaf816e05a0420c04d92be9ff5/LICENSE-MIT for the full license text.
// Modified to use `proc_macro_errors` style instead of results and rejects invalid repr combinations.

use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error2::abort;
use quote::{quote, ToTokens};
use std::cmp;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parenthesized, token, Attribute, LitInt, Token};

pub fn get_repr(attributes: &[Attribute]) -> Representation {
    attributes
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("repr") {
                Some(
                    attr.parse_args::<Representation>()
                        .unwrap_or_else(|e| abort!(attr, e)),
                )
            } else {
                None
            }
        })
        .fold(Representation::default(), |repr1, repr2| Representation {
            repr: match (repr1.repr, repr2.repr) {
                (a, Repr::Rust) => a,
                (Repr::Rust, b) => b,
                _ => abort!(repr2, "conflicting representation hints"),
            },
            modifier: match (repr1.modifier, repr2.modifier) {
                (Some(a), Some(b)) => Some(match (a, b) {
                    (Modifier::Packed(a), Modifier::Packed(b)) => {
                        if a != b {
                            abort!(b, "conflicting packed size")
                        }
                        Modifier::Packed(a)
                    }
                    (Modifier::Align(a), Modifier::Align(b)) => Modifier::Align(cmp::max(a, b)),
                    _ => abort!(repr2, "conflicting representation hints"),
                }),
                (a, None) => a,
                (None, b) => b,
            },
        })
}

macro_rules! mk_repr {(
  $(
    $Xn:ident => $xn:ident
  ),* $(,)?
) => (
  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum IntegerRepr {
    $($Xn),*
  }

  impl<'a> TryFrom<&'a str> for IntegerRepr {
    type Error = &'a str;

    fn try_from(value: &'a str) -> std::result::Result<Self, &'a str> {
      match value {
        $(
          stringify!($xn) => Ok(Self::$Xn),
        )*
        _ => Err(value),
      }
    }
  }

  impl ToTokens for IntegerRepr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      match self {
        $(
          Self::$Xn => tokens.extend(quote!($xn)),
        )*
      }
    }
  }
)}

mk_repr! {
  U8 => u8,
  I8 => i8,
  U16 => u16,
  I16 => i16,
  U32 => u32,
  I32 => i32,
  U64 => u64,
  I64 => i64,
  I128 => i128,
  U128 => u128,
  Usize => usize,
  Isize => isize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Repr {
    Rust,
    C,
    Transparent,
    Integer(IntegerRepr),
}

impl Repr {
    pub fn as_integer(&self) -> Option<IntegerRepr> {
        if let Self::Integer(v) = self {
            Some(*v)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modifier {
    Packed(u32),
    Align(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Representation {
    pub repr: Repr,
    pub modifier: Option<Modifier>,
}

impl Representation {
    pub fn is_packed(&self) -> bool {
        self.modifier == Some(Modifier::Packed(1))
    }
}

impl Default for Representation {
    fn default() -> Self {
        Self {
            modifier: None,
            repr: Repr::Rust,
        }
    }
}

impl Parse for Representation {
    fn parse(input: ParseStream<'_>) -> syn::Result<Representation> {
        let mut ret = Representation::default();
        while !input.is_empty() {
            let keyword = input.parse::<Ident>()?;
            // preëmptively call `.to_string()` *once* (rather than on `is_ident()`)
            let keyword_str = keyword.to_string();
            let new_repr = match keyword_str.as_str() {
                "C" => Repr::C,
                "transparent" => Repr::Transparent,
                "packed" => {
                    if matches!(ret.modifier, Some(Modifier::Align(..))) {
                        return Err(input.error("duplicate representation hint"));
                    }

                    let packed_size = if input.peek(token::Paren) {
                        let contents;
                        parenthesized!(contents in input);
                        LitInt::base10_parse::<u32>(&contents.parse()?)?
                    } else {
                        1
                    };
                    if let Some(Modifier::Packed(size)) = ret.modifier {
                        if size != packed_size {
                            return Err(input.error("conflicting packed size"));
                        }
                    }
                    ret.modifier = Some(Modifier::Packed(packed_size));
                    let _: Option<Token![,]> = input.parse()?;
                    continue;
                }
                "align" => {
                    if matches!(ret.modifier, Some(Modifier::Packed(..))) {
                        return Err(input.error("duplicate representation hint"));
                    }
                    let contents;
                    parenthesized!(contents in input);
                    let new_align = LitInt::base10_parse::<u32>(&contents.parse()?)?;
                    let existing_align = ret
                        .modifier
                        .and_then(|m| match m {
                            Modifier::Align(a) => Some(a),
                            _ => None,
                        })
                        .unwrap_or(1);
                    ret.modifier = Some(Modifier::Align(new_align.max(existing_align)));
                    let _: Option<Token![,]> = input.parse()?;
                    continue;
                }
                ident => {
                    let primitive = IntegerRepr::try_from(ident)
                        .map_err(|_| input.error("unrecognized representation hint"))?;
                    Repr::Integer(primitive)
                }
            };
            ret.repr = match (ret.repr, new_repr) {
                (Repr::Rust, new_repr) => {
                    // This is the first explicit repr.
                    new_repr
                }
                (_, _) => {
                    return Err(input.error("duplicate representation hint"));
                }
            };
            let _: Option<Token![,]> = input.parse()?;
        }
        Ok(ret)
    }
}

impl ToTokens for Representation {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut meta = Punctuated::<_, Token![,]>::new();

        match self.repr {
            Repr::Rust => {}
            Repr::C => meta.push(quote!(C)),
            Repr::Transparent => meta.push(quote!(transparent)),
            Repr::Integer(primitive) => meta.push(quote!(#primitive)),
        }
        if let Some(modifier) = self.modifier.as_ref() {
            match modifier {
                Modifier::Packed(size) => {
                    let lit = LitInt::new(&size.to_string(), Span::call_site());
                    meta.push(quote!(packed(#lit)));
                }
                Modifier::Align(size) => {
                    let lit = LitInt::new(&size.to_string(), Span::call_site());
                    meta.push(quote!(align(#lit)));
                }
            }
        }

        tokens.extend(quote!(
          #[repr(#meta)]
        ));
    }
}

#[cfg(test)]
mod tests {
    use crate::util::repr::Modifier;
    use quote::quote;
    use syn::{parse2, parse_quote};

    use super::{get_repr, IntegerRepr, Repr, Representation};

    #[test]
    fn parse_basic_repr() {
        let attr = parse_quote!(#[repr(C)]);
        let repr = get_repr(&[attr]);
        assert_eq!(
            repr,
            Representation {
                repr: Repr::C,
                ..Default::default()
            }
        );

        let attr = parse_quote!(#[repr(transparent)]);
        let repr = get_repr(&[attr]);
        assert_eq!(
            repr,
            Representation {
                repr: Repr::Transparent,
                ..Default::default()
            }
        );

        let attr = parse_quote!(#[repr(u8)]);
        let repr = get_repr(&[attr]);
        assert_eq!(
            repr,
            Representation {
                repr: Repr::Integer(IntegerRepr::U8),
                ..Default::default()
            }
        );

        let attr = parse_quote!(#[repr(packed)]);
        let repr = get_repr(&[attr]);
        assert_eq!(
            repr,
            Representation {
                modifier: Some(Modifier::Packed(1)),
                ..Default::default()
            }
        );

        let attr = parse_quote!(#[repr(packed(1))]);
        let repr = get_repr(&[attr]);
        assert_eq!(
            repr,
            Representation {
                modifier: Some(Modifier::Packed(1)),
                ..Default::default()
            }
        );

        let attr = parse_quote!(#[repr(packed(2))]);
        let repr = get_repr(&[attr]);
        assert_eq!(
            repr,
            Representation {
                modifier: Some(Modifier::Packed(2)),
                ..Default::default()
            }
        );

        let attr = parse_quote!(#[repr(align(2))]);
        let repr = get_repr(&[attr]);
        assert_eq!(
            repr,
            Representation {
                modifier: Some(Modifier::Align(2)),
                ..Default::default()
            }
        );
    }

    #[test]
    fn parse_advanced_repr() {
        let attr = parse_quote!(#[repr(align(4), align(2))]);
        let repr = get_repr(&[attr]);
        assert_eq!(
            repr,
            Representation {
                modifier: Some(Modifier::Align(4)),
                ..Default::default()
            }
        );

        let attr1 = parse_quote!(#[repr(align(1))]);
        let attr2 = parse_quote!(#[repr(align(4))]);
        let attr3 = parse_quote!(#[repr(align(2))]);
        let repr = get_repr(&[attr1, attr2, attr3]);
        assert_eq!(
            repr,
            Representation {
                modifier: Some(Modifier::Align(4)),
                ..Default::default()
            }
        );
    }

    #[test]
    fn conflicting_reprs() {
        parse2::<Representation>(quote!(#[repr(C, transparent)])).unwrap_err();
        parse2::<Representation>(quote!(#[repr(C, u8)])).unwrap_err();
        parse2::<Representation>(quote!(#[repr(u8, transparent)])).unwrap_err();
        parse2::<Representation>(quote!(#[repr(i64, C)])).unwrap_err();
        parse2::<Representation>(quote!(#[repr(i64, u8)])).unwrap_err();
        parse2::<Representation>(quote!(#[repr(packed(2), packed)])).unwrap_err();
        parse2::<Representation>(quote!(#[repr(align(2), packed)])).unwrap_err();
    }
}
