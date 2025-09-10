use proc_macro2::{TokenStream, TokenTree};

pub fn split_by_comma(input: TokenStream) -> Vec<TokenStream> {
    let mut output = Vec::new();
    let mut iter = input.clone().into_iter().peekable();

    while iter.peek().is_some() {
        let substream: TokenStream = iter
            .by_ref()
            .take_while(|t| match t {
                TokenTree::Punct(c) if c.as_char() == ',' => false,
                _ => true,
            })
            .collect();

        output.push(substream);
    }

    match input.into_iter().last() {
        Some(TokenTree::Punct(c)) if c.as_char() == ',' => output.push(TokenStream::new()),
        None => output.push(TokenStream::new()),
        _ => {}
    }

    output
}

#[cfg(test)]
mod tests2 {
    use super::*;
    use quote::quote;

    #[test]
    fn test_split_by_comma() {
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

        let output = split_by_comma(input.into());

        assert_eq!(expected.len(), output.len());
        for (exp, out) in expected.iter().zip(output.iter()) {
            assert_eq!(exp.to_string(), out.to_string());
        }
    }

    #[test]
    fn test_split_by_comma_with_trailing_comma() {
        let input = quote!(A, B, C,);
        let expected = vec![quote!(A), quote!(B), quote!(C), quote!()];

        let output = split_by_comma(input.into());

        assert_eq!(expected.len(), output.len());
        for (exp, out) in expected.iter().zip(output.iter()) {
            assert_eq!(exp.to_string(), out.to_string());
        }
    }

    #[test]
    fn test_split_by_comma_with_empty_input() {
        let input = quote!();
        let expected = vec![quote!()];

        let output = split_by_comma(input.into());

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
mod tests {
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
