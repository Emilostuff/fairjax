mod case;
mod consume;
mod derive;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(Message)]
pub fn derive_message_trait(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive::expand_derive_message_trait(&input).into()
}

#[proc_macro]
pub fn match_fairest_case(input: TokenStream) -> TokenStream {
    consume::expand_consume(input.into())
        .map(|_| TokenStream::new())
        .unwrap_or_else(|e| e.to_compile_error().into())
}

#[proc_macro]
pub fn receive(input: TokenStream) -> TokenStream {
    consume::expand_consume(input.into())
        .map(|_| TokenStream::new())
        .unwrap_or_else(|e| e.to_compile_error().into())
}

#[proc_macro]
pub fn case(input: TokenStream) -> TokenStream {
    case::expand_case(input.into())
        .map(|_| TokenStream::new())
        .unwrap_or_else(|e| e.to_compile_error().into())
}
