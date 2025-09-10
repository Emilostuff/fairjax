use proc_macro2::{Delimiter, Group, Ident, Span, TokenStream, TokenTree};
use quote::{ToTokens, TokenStreamExt};
use syn::{Error, Result, Token};

macro_rules! parse_next {
    // With guard
    ($input:expr, $pat:pat if $guard:expr => $val:expr, $err:literal) => {
        match $input {
            $pat if $guard => $val,
            Some(tt) => {
                return Err(syn::Error::new_spanned(tt, concat!("Expected ", $err)));
            }
            None => {
                return Err(syn::Error::new(
                    proc_macro2::Span::call_site(),
                    concat!("Expected ", $err),
                ));
            }
        }
    };

    // Without guard
    ($input:expr, $pat:pat => $val:expr, $err:literal) => {
        match $input {
            $pat => $val,
            Some(tt) => {
                return Err(syn::Error::new_spanned(tt, concat!("Expected ", $err)));
            }
            None => {
                return Err(syn::Error::new(
                    proc_macro2::Span::call_site(),
                    concat!("Expected ", $err),
                ));
            }
        }
    };
}

macro_rules! parse_until_fat_arrow {
    ($iter:expr) => {{
        let mut collected = TokenStream::new();

        loop {
            let Some(tt) = $iter.next() else {
                return Err(syn::Error::new(
                    proc_macro2::Span::call_site(),
                    "Expected `=>` but found end of input",
                ));
            };

            if let proc_macro2::TokenTree::Punct(eq) = &tt {
                if eq.as_char() == '=' && eq.spacing() == proc_macro2::Spacing::Joint {
                    if let Some(proc_macro2::TokenTree::Punct(gt)) = $iter.peek() {
                        if gt.as_char() == '>' && gt.spacing() == proc_macro2::Spacing::Alone {
                            // consume '>' from the iterator
                            $iter.next();
                            break;
                        }
                    }
                }
            }

            collected.append(tt);
        }

        collected
    }};
}

pub fn expand_consume(input: TokenStream) -> Result<TokenStream> {
    // Parse input
    let join_def = JoinDefinition::parse(input)?;

    // Build backend (TODO)
    Ok(TokenStream::new())
}

#[derive(Debug)]
struct JoinDefinition {
    message: Ident,
    matcher: Ident,
    join_patterns: Vec<JoinPattern>,
}

impl JoinDefinition {
    fn parse(input: proc_macro2::TokenStream) -> Result<Self> {
        use TokenTree::*;

        let mut iter = input.into_iter();

        // Parse message identifier
        let message = parse_next!(iter.next(), Some(Ident(ident)) => ident, "identifier");

        // Parse two punctuations
        parse_next!(iter.next(), Some(Punct(c)) if c.as_char() == '=' && c.spacing() == proc_macro2::Spacing::Joint => (),"'='");
        parse_next!(iter.next(), Some(Punct(c)) if c.as_char() == '>' => (),"'>'");

        // Parse matcher identifier
        let matcher = parse_next!(iter.next(), Some(Ident(ident)) => ident, "identifier");

        // Parse join patterns
        let join_patterns = parse_next!(
           iter.next(),
           Some(Group(g)) if g.delimiter() == Delimiter::Brace => JoinPattern::parse(g.stream())?,
           "a parenthesis-delimited group"
        );

        if let Some(_) = iter.next() {
            return Err(Error::new_spanned(iter.next().unwrap(), "Expected nothing"));
        }

        Ok(Self {
            message,
            matcher,
            join_patterns,
        })
    }
}

#[derive(Debug)]
struct JoinPattern {
    pattern: Group,
    guard: TokenStream,
    body: Group,
}

impl JoinPattern {
    fn parse(input: TokenStream) -> Result<Vec<Self>> {
        use TokenTree::*;

        let mut output = Vec::new();
        let mut iter = input.into_iter().peekable();

        loop {
            let pattern = parse_next!(
                iter.next(),
                Some(Group(g)) if g.delimiter() == Delimiter::Parenthesis => g,
                "a parenthesis-delimited group"
            );

            // Parse guard expression
            let guard = parse_until_fat_arrow!(iter);

            // Parse body block
            let body = parse_next!(
                iter.next(),
                Some(Group(g)) if g.delimiter() == Delimiter::Brace => g,
                "a brace-delimited group"
            );

            parse_next!(iter.next(), Some(Punct(c)) if c.as_char() == ',' => (),"','");

            output.push(Self {
                pattern,
                guard,
                body,
            });

            // peek to see if done
            if iter.peek().is_none() {
                return Ok(output);
            }
        }
    }
}
