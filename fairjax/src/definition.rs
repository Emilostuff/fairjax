use crate::utils::split_by_char;
use crate::{case::Case, utils::split_by_double_char};
use proc_macro2::{Delimiter, Span, TokenStream, TokenTree};
use quote::quote;
use syn::spanned::Spanned;
use syn::{Error, Result};

pub struct JoinDefinition {
    message_type: TokenStream,
    mailbox: TokenStream,
    message: TokenStream,
    cases: Vec<Case>,
}

// Input Parsing
impl JoinDefinition {
    pub fn parse(input: TokenStream) -> Result<Self> {
        let args = split_by_char(input, ',');
        if args.len() < 3 {
            return Err(Error::new(Span::call_site(), "Expected at least one case"));
        }

        let mut args = args.into_iter();

        let message_type = args.next().unwrap();

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
            message_type,
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
        let mailbox_ident = self.mailbox.clone();
        let match_result = quote!(result);

        let action_code = self.generate_action_code(match_result.clone());
        let declaration_code = self.generate_declaration_code();
        let msg = self.message;

        quote! {
            // Run init once
            if !#mailbox_ident.is_initialized() {
                // Check if user modified mailbox
                if #mailbox_ident.is_modified() {
                    panic!("Mailbox was modified prior to initialization");
                }
                #declaration_code

                // finalize init
                #mailbox_ident.init();
            }

            let #match_result = #mailbox_ident.consume(#msg.clone());

            // Run action if match was found
            #action_code
        }
    }

    pub fn generate_action_code(&self, match_result: TokenStream) -> TokenStream {
        let input_var = quote!(input);
        let actions = self
            .cases
            .iter()
            .map(|c| c.generate_action_code(input_var.clone()))
            .collect::<Vec<TokenStream>>();
        let indices = 0..actions.len();

        quote! {
            match #match_result {
                #(Some((#indices, #input_var)) => {#actions}),*,
                None => (),
                _ => panic!(),
            }
        }
    }

    pub fn generate_declaration_code(&self) -> TokenStream {
        let declarations = self
            .cases
            .iter()
            .enumerate()
            .map(|(i, c)| {
                c.generate_declaration_code(self.message_type.clone(), i, self.mailbox.clone())
            })
            .collect::<Vec<TokenStream>>();

        quote! {
            #(#declarations)*
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
            quote!(A(a, b) && B(_, c) && C(d)),
            quote!(),
            quote! {
                println!("Success");
            },
        )
        .unwrap();

        let case_1 = Case::new(
            quote!(E(k, _) && C(d)),
            quote!(),
            quote! {
                println!("More Success");
            },
        )
        .unwrap();

        let join_def = JoinDefinition {
            message_type: quote!(),
            mailbox: quote!(),
            message: quote!(),
            cases: vec![case_0, case_1],
        };

        let match_result = quote!(result);

        let output = join_def.generate_action_code(match_result.clone());
        let expected = quote! {
            match result {
                Some((0usize, input)) => {
                    match (input[0usize], input[1usize], input[2usize]) {
                        (A(a, b), B(_, c), C(d)) => {
                            println!("Success");
                        },
                        _ => panic!("not good")
                    }
                },
                Some((1usize, input)) => {
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
