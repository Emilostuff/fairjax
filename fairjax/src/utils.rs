use proc_macro2::{Group, TokenStream, TokenTree};
use quote::quote;

pub fn split_by_char(input: TokenStream, ch: char) -> Vec<TokenStream> {
    let mut output = Vec::new();
    let mut iter = input.clone().into_iter().peekable();

    while iter.peek().is_some() {
        let substream: TokenStream = iter
            .by_ref()
            .take_while(|t| match t {
                TokenTree::Punct(c) if c.as_char() == ch => false,
                _ => true,
            })
            .collect();

        output.push(substream);
    }

    match input.into_iter().last() {
        Some(TokenTree::Punct(c)) if c.as_char() == ch => output.push(TokenStream::new()),
        None => output.push(TokenStream::new()),
        _ => {}
    }

    output
}

#[cfg(test)]
mod split_by_char_tests {
    use super::*;
    use quote::quote;

    #[test]
    fn test_split_by_char() {
        let input = quote!(A(a, b), B { a, b }, {
            a;
            b;
            c;
        });
        let expected = vec![
            quote!(A(a, b)),
            quote!(B { a, b }),
            quote!({
                a;
                b;
                c;
            }),
        ];

        let output = split_by_char(input.into(), ',');

        assert_eq!(expected.len(), output.len());
        for (exp, out) in expected.iter().zip(output.iter()) {
            assert_eq!(exp.to_string(), out.to_string());
        }
    }

    #[test]
    fn test_split_by_char_with_trailing_comma() {
        let input = quote!(A, B, C,);
        let expected = vec![quote!(A), quote!(B), quote!(C), quote!()];

        let output = split_by_char(input.into(), ',');

        assert_eq!(expected.len(), output.len());
        for (exp, out) in expected.iter().zip(output.iter()) {
            assert_eq!(exp.to_string(), out.to_string());
        }
    }

    #[test]
    fn test_split_by_char_with_empty_input() {
        let input = quote!();
        let expected = vec![quote!()];

        let output = split_by_char(input.into(), ',');

        assert_eq!(expected.len(), output.len());
        for (exp, out) in expected.iter().zip(output.iter()) {
            assert_eq!(exp.to_string(), out.to_string());
        }
    }
}

pub fn split_by_double_char(input: TokenStream, ch: char) -> Vec<TokenStream> {
    let mut output = Vec::new();
    let mut iter = input.clone().into_iter().peekable();

    loop {
        let mut substream = Vec::new();
        while let Some(tt) = iter.next() {
            match tt {
                TokenTree::Punct(ref c) if c.as_char() == ch => match iter.peek() {
                    Some(TokenTree::Punct(c)) if c.as_char() == ch => {
                        iter.next();
                        break;
                    }
                    _ => substream.push(tt),
                },
                _ => substream.push(tt),
            }
        }

        output.push(substream.into_iter().collect());

        if iter.peek().is_none() {
            break;
        }
    }

    output
}

#[cfg(test)]
mod split_by_double_char_tests {
    use super::*;
    use quote::quote;

    #[test]
    fn test_split_by_double_ampersand() {
        let input = quote!(A && B);
        let expected = vec![quote!(A), quote!(B)];

        let output = split_by_double_char(input.into(), '&');

        assert_eq!(expected.len(), output.len());
        for (exp, out) in expected.iter().zip(output.iter()) {
            assert_eq!(exp.to_string(), out.to_string());
        }
    }

    // #[test]
    // fn test_split_by_double_ampersand_with_trailing_ampersands() {
    //     let input = quote!(A && B && C &&);
    //     let expected = vec![quote!(A), quote!(B), quote!(C), quote!()];

    //     let output = split_by_double_ampersand(input.into());

    //     assert_eq!(expected.len(), output.len());
    //     for (exp, out) in expected.iter().zip(output.iter()) {
    //         assert_eq!(exp.to_string(), out.to_string());
    //     }
    // }

    // #[test]
    // fn test_split_by_double_ampersand_with_trailing_ampersand() {
    //     let input = quote!(A && B && C &);
    //     let expected = vec![quote!(A), quote!(B), quote!(C &)];

    //     let output = split_by_double_ampersand(input.into());

    //     assert_eq!(expected.len(), output.len());
    //     for (exp, out) in expected.iter().zip(output.iter()) {
    //         assert_eq!(exp.to_string(), out.to_string());
    //     }
    // }
}

pub struct GroupExtraction {
    pub prefix: TokenStream,
    pub group: Group,
    pub postfix: TokenStream,
}

pub fn extract_group(input: &TokenStream) -> Option<GroupExtraction> {
    let mut prefix = Vec::new();
    let mut iter = input.clone().into_iter();

    while let Some(tt) = iter.next() {
        match tt {
            TokenTree::Group(group) => {
                return Some(GroupExtraction {
                    prefix: prefix.into_iter().collect(),
                    group,
                    postfix: iter.collect(),
                });
            }
            _ => prefix.push(tt),
        }
    }
    None
}

#[cfg(test)]
mod extract_group_tests {
    use super::*;
    use proc_macro2::Delimiter;
    use quote::quote;

    #[test]
    fn test_group() {
        let input = quote!(some::prefix(group)postfix);

        let prefix = quote!(some::prefix);
        let group = Group::new(Delimiter::Parenthesis, quote!(group));
        let postfix = quote!(postfix);

        let output = extract_group(&input).unwrap();

        compare_token_streams(&prefix, &output.prefix);
        assert_eq!(&group.delimiter(), &output.group.delimiter());
        compare_token_streams(&group.stream(), &output.group.stream());
        compare_token_streams(&postfix, &output.postfix);
    }
}

pub fn parse_identifier(input: &TokenStream, allow_tail: bool) -> syn::Result<proc_macro2::Ident> {
    let mut iter = input.clone().into_iter().peekable();
    match iter.next() {
        Some(TokenTree::Ident(ident)) => match iter.peek() {
            None => Ok(ident),
            Some(_) if allow_tail => Ok(ident),
            Some(_) => {
                let tail = iter.collect::<TokenStream>();
                Err(syn::Error::new_spanned(
                    tail.clone(),
                    format!(
                        "Unexpected tokens: '{}' after identifier: '{}'",
                        tail, ident
                    ),
                ))
            }
        },
        _ => Err(syn::Error::new_spanned(
            input,
            format!("Expected identifier, got: '{}'", input),
        )),
    }
}

pub fn compare_token_streams(a_input: &TokenStream, b_input: &TokenStream) {
    let a_string = a_input.to_string();
    let b_string = b_input.to_string();
    let mut a_iter = a_string.split_whitespace();
    let mut b_iter = b_string.split_whitespace();
    let mut accum = String::new();
    loop {
        match (a_iter.next(), b_iter.next()) {
            (Some(a), Some(b)) if a.to_string() == b.to_string() => {
                accum.push_str(&format!("{} ", a));
            }
            (Some(a), Some(b)) => {
                panic!(
                    "For tokenstream: [\n{} ...\n]\n\nExpected: '{}', got: '{}'",
                    accum,
                    a.to_string(),
                    b.to_string()
                );
            }
            (None, None) => break,
            _ => panic!("Token streams have different lengths"),
        }
    }
}
