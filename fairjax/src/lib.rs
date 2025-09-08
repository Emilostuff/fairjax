use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

mod derive;

#[proc_macro_derive(Message)]
pub fn derive_message_trait(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive::derive_message_trait(&input).into()
}
