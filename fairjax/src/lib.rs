mod compile {
    pub mod case {
        pub mod action;
        pub mod declaration;
    }
    pub mod definition;
}
mod parse {
    pub mod case;
    pub mod definition;
    pub mod pattern;
    pub mod strategy;
}
mod derive;
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
    match parse::definition::JoinDefinition::parse(input.into()) {
        Ok(def) => compile::definition::JoinDefinitionGenerator::new(def)
            .generate()
            .into(),
        Err(e) => return e.to_compile_error().into(),
    }
}

trait Compile {
    fn generate(self) -> proc_macro2::TokenStream;
}
