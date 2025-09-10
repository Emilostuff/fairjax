use super::utils::{split_by_comma, split_by_double_ampersand};
use proc_macro2::{Delimiter, Group, Ident, Span, TokenStream, TokenTree};
use syn::{Error, Result};

pub fn expand_case(input: TokenStream) -> Result<TokenStream> {
    let args = split_by_comma(input);

    // Check number of args is correct
    if args.len() != 3 {
        return Err(Error::new(Span::call_site(), "Expected 3 arguments"));
    }

    // Unpack split
    let arm = args[0].clone();
    let pattern = args[1].clone();
    let body = args[2].clone();

    // Check match arm is well-formed i.e. separated by '&&'s
    let arm = split_by_double_ampersand(arm);

    Ok(TokenStream::new())
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use quote::quote;
    // use syn::parse_quote;

    // #[test]
    // fn test_expand_case() {
    //     // define input
    //     let input = quote! {
    //         Fault(_, fid1, _, ts1) && Fix(_, fid2, ts2),
    //         fid1 && fid,
    //         {
    //             updateMaintenanceStats(ts1, ts2);
    //         }
    //     };

    //     // define expected output
    //     let expected = quote! {
    //         {
    //             [Fault(_, fid1, _, ts1), Fix(_, fid2, ts2)],
    //             {fid1 && fid},
    //             {updateMaintenanceStats(ts1, ts2);}
    //         }
    //     };

    //     let output = expand_case(input.into()).unwrap();
    //     assert_eq!(expected.to_string(), output.to_string());
    // }
}
