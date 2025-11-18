#![recursion_limit = "256"]

mod analyse {
    pub mod bundle;
    pub mod definition;
    pub mod groups;
    pub mod partition;
    pub mod strategy;
}

mod compile {
    pub mod case {
        pub mod accept;
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

mod traits;

use crate::compile::sections::{action::ActionSection, setup::SetupSection};
use crate::compile::top::TopLevelCodeGen;

#[proc_macro]
pub fn fairjax(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match parse::definition::RawJoinDefinition::parse(input.into()) {
        Ok(def) => match analyse::definition::JoinDefinition::analyse(def) {
            Ok(analysed_def) => {
                compile::top::TopLevel::generate::<ActionSection, SetupSection>(&analysed_def)
                    .into()
            }
            Err(e) => e.to_compile_error().into(),
        },
        Err(e) => return e.to_compile_error().into(),
    }
}
