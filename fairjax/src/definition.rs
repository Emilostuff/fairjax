use crate::utils::split_by_comma;
use crate::{case::Case, utils::split_by_double_char};
use proc_macro2::{Delimiter, Span, TokenStream, TokenTree};
use quote::quote;
use syn::spanned::Spanned;
use syn::{Error, Result};

pub struct JoinDefinition {
    mailbox: TokenStream,
    message: TokenStream,
    cases: Vec<Case>,
}

// Input Parsing
impl JoinDefinition {
    pub fn parse(input: TokenStream) -> Result<Self> {
        let args = split_by_comma(input);
        if args.len() < 2 {
            return Err(Error::new(Span::call_site(), "Expected at least one case"));
        }

        let mut args = args.into_iter();

        let mailbox_message_specifier = split_by_double_char(args.next().unwrap(), '>');
        if mailbox_message_specifier.len() != 2 {
            return Err(Error::new(
                mailbox_message_specifier[0].span(),
                "Invalid mailbox/message specifier",
            ));
        }

        let message = mailbox_message_specifier[0].clone();
        let mailbox = mailbox_message_specifier[1].clone();

        // Parse remaining args to cases:
        let cases = args
            .map(|arg| Case::parse(JoinDefinition::extract_case(arg)?))
            .collect::<Result<Vec<Case>>>()?;

        // Return join defenition
        Ok(JoinDefinition {
            mailbox,
            message,
            cases,
        })
    }

    fn extract_case(input: TokenStream) -> Result<TokenStream> {
        let mut iter = input.into_iter();
        match iter.next() {
            Some(TokenTree::Ident(ident)) if ident == "case" => (),
            _ => {
                return Err(Error::new(
                    Span::call_site(),
                    "Expected a 'case' declaration",
                ));
            }
        };

        match iter.next() {
            Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Parenthesis => {
                if iter.next().is_none() {
                    return Ok(g.stream());
                } else {
                    return Err(Error::new(Span::call_site(), "Too many things"));
                }
            }
            _ => return Err(Error::new(Span::call_site(), "Expected ()")),
        }
    }
}

#[cfg(test)]
mod parse_tests {
    use super::*;
}

// Code Generation
impl JoinDefinition {
    pub fn generate(self) -> TokenStream {
        // Init mailbox if not done
        let input_var = quote!(input);
        let match_index_var = quote!(fairest_match_index);

        let action_code = self.generate_action_code(match_index_var.clone(), input_var.clone());
        let msg = self.message;

        quote! {
            let #input_var = [#msg, #msg.clone(), #msg.clone()];
            let #match_index_var = None;
            #action_code
        }
    }

    pub fn generate_action_code(
        &self,
        match_index_var: TokenStream,
        input_var: TokenStream,
    ) -> TokenStream {
        let actions = self
            .cases
            .iter()
            .map(|c| c.generate_action_code(input_var.clone()))
            .collect::<Vec<TokenStream>>();
        let indices = 0..actions.len();

        quote! {
            match #match_index_var {
                #(Some(#indices) => {#actions}),*,
                None => (),
                _ => panic!(),
            }
        }
    }
}

#[cfg(test)]
mod code_gen_tests {
    use super::*;
    use crate::case::Case;
    use crate::utils::compare_token_streams;
    use quote::quote;

    #[test]
    fn test_generate_action_code() {
        let case_0 = Case::new(
            vec![quote!(A(a, b)), quote!(B(_, c)), quote!(C(d))],
            quote!(),
            quote! {
                println!("Success");
            },
        );

        let case_1 = Case::new(
            vec![quote!(E(k, _)), quote!(C(d))],
            quote!(),
            quote! {
                println!("More Success");
            },
        );

        let join_def = JoinDefinition {
            mailbox: quote!(),
            message: quote!(),
            cases: vec![case_0, case_1],
        };

        let input_var = quote!(input);
        let match_index_var = quote!(fairest_match_index);

        let output = join_def.generate_action_code(match_index_var, input_var);
        let expected = quote! {
            match fairest_match_index {
                Some(0usize) => {
                    match (input[0usize], input[1usize], input[2usize]) {
                        (A(a, b), B(_, c), C(d)) => {
                            println!("Success");
                        },
                        _ => panic!("not good")
                    }
                },
                Some(1usize) => {
                    match (input[0usize], input[1usize]) {
                        (E(k, _), C(d)) => {
                            println!("More Success");
                        },
                        _ => panic!("not good")
                    }
                },
                None => (),
                _ => panic!(),
            }
        };

        compare_token_streams(&expected, &output);
    }
}
