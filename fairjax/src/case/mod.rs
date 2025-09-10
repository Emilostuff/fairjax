use proc_macro2::{Delimiter, Group, Ident, Span, TokenStream, TokenTree};
use syn::{Error, Result};

pub fn expand_case(input: TokenStream) -> Result<TokenStream> {
    Ok(TokenStream::new())
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use quote::quote;
//     use syn::parse_quote;

//     #[test]
//     fn test_expand_case() {
//         // define input
//         let input = quote! {
//             Fault(_, fid1, _, ts1) && Fix(_, fid2, ts2),
//             fid1 && fid,
//             {
//                 updateMaintenanceStats(ts1, ts2);
//             }
//         };

//         // define expected output
//         let expected = quote! {
//             {
//                 [Fault(_, fid1, _, ts1), Fix(_, fid2, ts2)],
//                 {fid1 && fid},
//                 {updateMaintenanceStats(ts1, ts2);}
//             }
//         };

//         let output = expand_case(input.into()).unwrap();
//         assert_eq!(expected.to_string(), output.to_string());
//     }
// }
