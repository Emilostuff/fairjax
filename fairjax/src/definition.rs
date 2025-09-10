use crate::utils::split_by_comma;
use crate::{case::Case, utils::split_by_double_char};
use proc_macro2::{Delimiter, Span, TokenStream, TokenTree};
use syn::spanned::Spanned;
use syn::{Error, Result};

pub struct JoinDefinition {
    mailbox: TokenStream,
    message: TokenStream,
    cases: Vec<Case>,
}

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

    pub fn generate(self) -> TokenStream {
        TokenStream::new()
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
