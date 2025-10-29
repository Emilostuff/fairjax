#![recursion_limit = "256"]

mod compile {
    pub mod case {
        pub mod action;
        pub mod guard;
    }
    pub mod matchers;
    pub mod sections {
        pub mod action;
        pub mod setup;
    }
    pub mod pattern {
        pub mod full;
        pub mod sub;
    }

    pub mod top;
}
mod parse {
    pub mod case;
    pub mod context;
    pub mod definition;
    pub mod pattern;
    pub mod strategy;
    pub mod sub_pattern;
}

use crate::compile::sections::{action::ActionSection, setup::SetupSection};
use crate::compile::top::TopLevelCodeGen;

#[proc_macro]
pub fn fairjax(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match parse::definition::JoinDefinition::parse(input.into()) {
        Ok(def) => compile::top::TopLevel::generate::<ActionSection, SetupSection>(&def).into(),
        Err(e) => return e.to_compile_error().into(),
    }
}
