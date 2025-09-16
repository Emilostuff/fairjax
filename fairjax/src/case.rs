use crate::pattern::Pattern;
use crate::utils::split_by_char;
use proc_macro2::{Delimiter, Ident, Span, TokenStream, TokenTree};
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{Error, Result};

#[derive(Debug)]
pub struct Case {
    pattern: Pattern,
    guard: TokenStream,
    body: TokenStream,
}

// Input Parsing
impl Case {
    pub fn new(pattern: TokenStream, guard: TokenStream, body: TokenStream) -> Result<Self> {
        Ok(Case {
            pattern: Pattern::parse(pattern)?,
            guard,
            body,
        })
    }

    pub fn parse(input: TokenStream) -> Result<Self> {
        let inner = Case::unpack_case(input.clone())?;
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
            pattern,
            guard,
            body,
        })
    }

    fn unpack_case(input: TokenStream) -> Result<TokenStream> {
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
                    None => return Ok(g.stream()),
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
mod parse_tests {
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

// Code Generation
impl Case {
    pub fn generate_action_code(&self, input_var: TokenStream) -> TokenStream {
        let pattern = self.generate_pattern_match_code();
        let body = &self.body;
        let unpacking = self.generate_input_unpacking_code(input_var.clone());

        quote! {
            match (#unpacking) {
                (#pattern) => {
                    #body
                },
                _ => panic!("not good")
            }
        }
    }

    fn generate_pattern_match_code(&self) -> TokenStream {
        self.pattern.generate_full_pattern()
    }

    fn generate_input_unpacking_code(&self, input_var: TokenStream) -> TokenStream {
        let indices = 0..self.pattern.len();
        quote! {
            #(#input_var[#indices]),*
        }
    }

    fn generate_guard_fn(&self, guard_ident: Ident) -> TokenStream {
        let unpacking = self.generate_input_unpacking_code(quote!(messages));
        let pattern = self.pattern.generate_full_pattern();
        let guard = self.guard.clone();
        let span = guard.span();

        quote_spanned! {span=>
            fn #guard_ident(messages: &Vec<&Msg>) -> bool {
                match (#unpacking) {
                    (#pattern) => #guard,
                    _ => unreachable!(),
                }
            }
        }
    }

    pub fn generate_declaration_code(
        &self,
        message_type: Ident,
        case_index: usize,
        mailbox_ident: Ident,
    ) -> TokenStream {
        let match_group_ident = format_ident!("FairjaxGenerated{}", case_index);
        let guard_ident = format_ident!("fairjax_pattern_guard_{}", case_index);

        let declaration_code = self
            .pattern
            .generate_declaration_code(message_type.clone(), match_group_ident.clone());
        let guard_code = self.generate_guard_fn(guard_ident.clone());

        quote! {
            // Generate match group impl
            #declaration_code

            // Declare guard
            #guard_code

            // Generate pattern matcher
            let pm = fairjax_core::pattern::PatternMatcher::<#match_group_ident, #message_type>::new(#guard_ident);

            // Add to mailbox
            #mailbox_ident.add_pattern(Box::new(pm));
        }
    }
}

#[cfg(test)]
mod code_gen_tests {
    use super::*;
    use crate::utils::compare_token_streams;
    use quote::quote;

    #[test]
    fn test_generate_input_unpacking_code() {
        let case = Case {
            pattern: Pattern::parse(quote!(A && B && C)).unwrap(),
            guard: quote!(),
            body: quote!(),
        };
        let input_var = quote!(input);

        let output = case.generate_input_unpacking_code(input_var);
        let expected = quote!(input[0usize], input[1usize], input[2usize]);

        compare_token_streams(&expected, &output);
    }

    #[test]
    fn test_generate_pattern_match_code() {
        let case = Case {
            pattern: Pattern::parse(quote!(A(a, b) && B(_, c) && C(d))).unwrap(),
            guard: quote!(),
            body: quote!(),
        };

        let output = case.generate_pattern_match_code();
        let expected = quote!(A(a, b), B(_, c), C(d));

        compare_token_streams(&expected, &output);
    }

    #[test]
    fn test_generate_action_code() {
        let case = Case {
            pattern: Pattern::parse(quote!(A(a, b) && B(_, c) && C(d))).unwrap(),
            guard: quote!(),
            body: quote! {
                println!("Success");
            },
        };

        let input_var = quote!(input);
        let output = case.generate_action_code(input_var);
        let expected = quote! {
            match (input[0usize], input[1usize], input[2usize]) {
                (A(a, b), B(_, c), C(d)) => {
                    println!("Success");
                },
                _ => panic!("not good")
            }
        };

        compare_token_streams(&expected, &output);
    }
}
