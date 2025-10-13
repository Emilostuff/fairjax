use crate::parse::pattern::Pattern;
use crate::parse::strategy::Strategy;
use crate::utils::split_by_char;
use proc_macro2::{Delimiter, Span, TokenStream, TokenTree};
use quote::quote;
use syn::{Error, Result};

#[derive(Debug, Clone)]
pub struct Case {
    pub strategy: Strategy,
    pub pattern: Pattern,
    pub guard: TokenStream,
    pub body: TokenStream,
}

// Input Parsing
impl Case {
    pub fn new(pattern: TokenStream, guard: TokenStream, body: TokenStream) -> Result<Self> {
        Ok(Case {
            strategy: Strategy::Auto,
            pattern: Pattern::parse(pattern)?,
            guard,
            body,
        })
    }

    pub fn parse(input: TokenStream) -> Result<Self> {
        let (strategy, inner) = Case::unpack_case(input.clone())?;
        let mut args = split_by_char(inner, ',').into_iter();

        let pattern = match args.next() {
            Some(ts) => Pattern::parse(ts)?,
            None => {
                return Err(Error::new_spanned(
                    input.clone(),
                    "Pattern missing in 'case' declaration",
                ));
            }
        };

        let guard = match args.next() {
            Some(ts) => ts,
            None => {
                return Err(Error::new_spanned(
                    input.clone(),
                    "Guard missing in 'case' declaration",
                ));
            }
        };

        let body = match args.next() {
            Some(ts) => ts,
            None => {
                return Err(Error::new_spanned(
                    input.clone(),
                    "Body missing in 'case' declaration",
                ));
            }
        };

        Ok(Case {
            strategy,
            pattern,
            guard,
            body,
        })
    }

    fn unpack_case(input: TokenStream) -> Result<(Strategy, TokenStream)> {
        let mut iter = input.into_iter().peekable();
        let case_ident = match iter.next() {
            Some(TokenTree::Ident(ident)) if ident == "case" => ident,
            Some(tt) => return Err(Error::new_spanned(tt, "Expected 'case' keyword here")),
            None => {
                return Err(Error::new(
                    Span::call_site(),
                    "Expected a case declaration after ','",
                ));
            }
        };

        let rest = iter.clone();

        let strategy = match iter.peek() {
            Some(TokenTree::Group(_)) => Strategy::Auto,
            Some(_) => match (
                iter.next(),
                iter.next(),
                iter.next(),
                iter.next(),
                iter.next(),
            ) {
                (
                    Some(TokenTree::Punct(c)),
                    Some(TokenTree::Punct(c1)),
                    Some(TokenTree::Punct(gt)),
                    Some(TokenTree::Ident(ident)),
                    Some(TokenTree::Punct(lt)),
                ) if c.as_char() == ':'
                    && c1.as_char() == ':'
                    && gt.as_char() == '<'
                    && lt.as_char() == '>' =>
                {
                    match ident.to_string().as_str() {
                        "Auto" => Strategy::Auto,
                        "StatefulTree" => Strategy::StatefulTree,
                        "BruteForce" => Strategy::BruteForce,
                        _ => {
                            return Err(syn::Error::new_spanned(
                                ident,
                                "Expected one of the following strategies: 'Auto', 'StatefulTree', or 'BruteForce'",
                            ));
                        }
                    }
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        rest.collect::<TokenStream>(),
                        "Expected ()-group or strategy annotation '::<STRATEGY>' after 'case' keyword",
                    ));
                }
            },
            None => {
                return Err(syn::Error::new_spanned(
                    case_ident,
                    "Expected ()-group or strategy annotation '::<STRATEGY>' after 'case' keyword",
                ));
            }
        };

        match iter.peek().map(|x| x.clone()) {
            Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Parenthesis => {
                iter.next();
                match iter.peek() {
                    Some(_) => {
                        return Err(syn::Error::new_spanned(
                            iter.collect::<TokenStream>(),
                            "Unexpected tokens after 'case( .. )'",
                        ));
                    }
                    None => return Ok((strategy, g.stream())),
                }
            }
            Some(_) => {
                return Err(syn::Error::new_spanned(
                    iter.collect::<TokenStream>(),
                    "Expected ()-group after 'case' keyword",
                ));
            }
            None => Err(syn::Error::new_spanned(
                case_ident,
                "Expected ()-group after 'case' keyword",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::compare_token_streams;
    use quote::quote;

    #[test]
    fn test_expand_case() {
        let input = quote! {
            case(A(a, b) && B(_, c) && C(d),
            a == d,
            {
                f(b, c);
            })
        };

        let expected = Case {
            strategy: Strategy::Auto,
            pattern: Pattern::parse(quote!(A(a, b) && B(_, c) && C(d))).unwrap(),
            guard: quote!(a == d),
            body: quote!({
                f(b, c);
            }),
        };

        let output = Case::parse(input).unwrap();
        compare_token_streams(
            &expected.pattern.generate_full_pattern(),
            &output.pattern.generate_full_pattern(),
        );

        compare_token_streams(&expected.guard, &output.guard);
        compare_token_streams(&expected.body, &output.body);
    }
}
