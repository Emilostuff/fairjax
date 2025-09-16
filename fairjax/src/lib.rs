mod case;
mod definition;
mod derive;
mod pattern;
mod utils;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(Message)]
pub fn derive_message_trait(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive::expand_derive_message_trait(&input).into()
}

#[proc_macro]
pub fn match_fairest_case(input: TokenStream) -> TokenStream {
    match definition::JoinDefinition::parse(input.into()) {
        Ok(def) => def.generate().into(),
        Err(e) => return e.to_compile_error().into(),
    }
}
