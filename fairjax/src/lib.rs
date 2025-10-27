mod compile {
    pub mod case;
    pub mod matcher {
        pub mod brute_force;
        pub mod stateful_tree;
    }
    pub mod definition;
}
mod parse {
    pub mod case;
    pub mod definition;
    pub mod pattern;
    pub mod strategy;
}

mod utils;

#[proc_macro]
pub fn match_fairest_case(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match parse::definition::JoinDefinition::parse(input.into()) {
        Ok(def) => compile::definition::JoinDefinitionGenerator::new(def)
            .generate()
            .into(),
        Err(e) => return e.to_compile_error().into(),
    }
}
