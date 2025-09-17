use crate::utils::{parse_identifier, split_by_char};
use crate::{parse::case::Case, utils::split_by_double_char};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Error, Result};

pub struct JoinDefinition {
    pub message_type: Ident,
    pub mailbox: Ident,
    pub message: Ident,
    pub cases: Vec<Case>,
}

// Input Parsing
impl JoinDefinition {
    pub fn parse(input: TokenStream) -> Result<Self> {
        let mut args = split_by_char(input, ',').into_iter();

        // Parse message type
        let message_type = match args.next() {
            Some(ts) => parse_identifier(&ts, false)?,
            None => {
                return Err(Error::new(
                    Span::call_site(),
                    "Missing message type parameter",
                ));
            }
        };

        // Parse message and mailbox identifier
        let (message, mailbox) = match args.next() {
            Some(tt) => {
                let mut blocks = split_by_double_char(tt.clone(), '>').into_iter();

                match (blocks.next(), blocks.next(), blocks.next()) {
                    (Some(message), Some(mailbox), None) => {
                        let message = parse_identifier(&message, false)?;
                        let mailbox = parse_identifier(&mailbox, false)?;
                        (message, mailbox)
                    }
                    _ => {
                        return Err(Error::new_spanned(
                            tt,
                            "Invalid syntax for message/mailbox parameter (must contain exactly one '>>' operator)",
                        ));
                    }
                }
            }
            None => {
                return Err(Error::new(
                    Span::call_site(),
                    "Missing message/mailbox parameter",
                ));
            }
        };

        // Parse remaining args to cases:
        let cases = args
            .map(|arg| Case::parse(arg))
            .collect::<Result<Vec<Case>>>()?;

        // Return join defenition
        Ok(JoinDefinition {
            message_type,
            mailbox,
            message,
            cases,
        })
    }
}

#[cfg(test)]
mod parse_tests {
    use super::*;
}
