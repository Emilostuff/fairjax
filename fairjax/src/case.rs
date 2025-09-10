use crate::utils::{split_by_comma, split_by_double_char};
use proc_macro2::{Span, TokenStream};
use syn::{Error, Result};

#[derive(Debug)]
pub struct Case {
    pattern: Vec<TokenStream>,
    guard: TokenStream,
    body: TokenStream,
}

impl Case {
    pub fn parse(input: TokenStream) -> Result<Self> {
        let args = split_by_comma(input);

        // Check number of args is correct
        if args.len() != 3 {
            return Err(Error::new(Span::call_site(), "Expected 3 arguments"));
        }

        // Unpack split
        let pattern = args[0].clone();
        let guard = args[1].clone();
        let body = args[2].clone();

        // Check match arm is well-formed i.e. separated by '&&'s
        let pattern = split_by_double_char(pattern, '&');

        Ok(Case {
            pattern,
            guard,
            body,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    fn compare_token_streams(input: (&TokenStream, &TokenStream)) {
        assert_eq!(input.0.to_string(), input.1.to_string());
    }

    fn compare_cases(a: &Case, b: &Case) {
        assert_eq!(a.pattern.len(), b.pattern.len(),);

        let _ = a
            .pattern
            .iter()
            .zip(b.pattern.iter())
            .map(compare_token_streams);

        compare_token_streams((&a.guard, &b.guard));
        compare_token_streams((&a.body, &b.body));
    }

    #[test]
    fn test_expand_case() {
        let input = quote! {
            A(a, b) && B(_, c) && C(d),
            a == d,
            {
                f(b, c);
            }
        };

        let expected = Case {
            pattern: vec![quote!(A(a, b)), quote!(B(_, c)), quote!(C(d))],
            guard: quote!(a == d),
            body: quote!({
                f(b, c);
            }),
        };

        let output = Case::parse(input).unwrap();
        compare_cases(&expected, &output);
    }
}
