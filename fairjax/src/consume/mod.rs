use proc_macro2::{Delimiter, Group, Ident, Span, TokenStream, TokenTree};
use syn::{Error, Result};

pub fn expand_consume(input: TokenStream) -> Result<TokenStream> {
    let join_def = JoinDefinition::parse(input)?;
    Ok(TokenStream::new())
}

#[derive(Debug)]
struct JoinDefinition {
    msg: Ident,
    matcher: Ident,
    block: Group,
}

impl JoinDefinition {
    fn parse(input: proc_macro2::TokenStream) -> Result<Self> {
        let mut iter = input.into_iter();

        // Parse message identifier
        let msg = match iter.next() {
            Some(TokenTree::Ident(ident)) => ident,
            Some(tt) => return Err(Error::new_spanned(tt, "Expected a message identifier here")),
            None => {
                return Err(Error::new(
                    Span::call_site(),
                    "Expected a message identifier, but found nothing",
                ));
            }
        };

        // Parse two punctuations
        let first = iter.next();
        let second = iter.next();

        match (first, second) {
            (Some(TokenTree::Punct(_)), Some(TokenTree::Punct(_))) => (),
            (first_tt, _) => {
                let span = first_tt.as_ref().map_or(Span::call_site(), |t| t.span());
                return Err(Error::new(span, "Expected two punctuation tokens here"));
            }
        }

        // Parse matcher identifier
        let matcher = match iter.next() {
            Some(TokenTree::Ident(ident)) => ident,
            Some(tt) => return Err(Error::new_spanned(tt, "Expected a matcher identifier here")),
            None => {
                return Err(Error::new(
                    Span::call_site(),
                    "Expected a matcher identifier, but found nothing",
                ));
            }
        };

        // Parse block
        let block = match iter.next() {
            Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => group,
            Some(tt) => return Err(Error::new_spanned(tt, "Expected a brace delimeted group")),
            None => return Err(Error::new(Span::call_site(), "Expected a block here")),
        };

        Ok(Self {
            msg,
            matcher,
            block,
        })
    }
}
