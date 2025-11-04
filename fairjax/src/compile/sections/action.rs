use crate::{
    compile::{case::action::ActionCodeGen, pattern::full::PatternCompiler},
    parse::case::Case,
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

pub trait ActionSectionCodeGen {
    fn generate<A: ActionCodeGen>(cases: Vec<&dyn Case>, result_name: &'static str) -> TokenStream;
}

pub struct ActionSection;

impl ActionSectionCodeGen for ActionSection {
    fn generate<A: ActionCodeGen>(cases: Vec<&dyn Case>, result_name: &'static str) -> TokenStream {
        let actions = cases
            .iter()
            .map(|c| A::generate::<PatternCompiler>(*c, result_name))
            .collect::<Vec<TokenStream>>();

        let indices = cases.iter().map(|c| c.index());
        let result_ident = Ident::new(result_name, Span::call_site());

        quote! {
            #[allow(unused_variables)]
            match #result_ident.case_id() {
                #(&fairjax_core::CaseId(#indices) => #actions),*,
                _ => panic!(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro_utils::assert_tokens;
    use proc_macro2::TokenStream;
    use quote::{ToTokens, format_ident};
    use syn::Expr;

    // Mock Case
    struct MockCase {
        id: usize,
    }

    impl Case for MockCase {
        fn index(&self) -> usize {
            self.id
        }
        fn strategy(&self) -> crate::parse::strategy::Strategy {
            unimplemented!()
        }
        fn pattern(&self) -> &dyn crate::parse::pattern::Pattern {
            unimplemented!()
        }
        fn guard(&self) -> Option<Expr> {
            unimplemented!()
        }
        fn body(&self) -> Expr {
            unimplemented!()
        }

        fn span(&self) -> Span {
            Span::call_site()
        }
    }

    // Mock ActionCodeGen
    struct MockActionCodeGen;

    impl ActionCodeGen for MockActionCodeGen {
        fn generate<P: crate::compile::pattern::full::PatternCodeGen>(
            case: &dyn Case,
            _result_name: &'static str,
        ) -> TokenStream {
            format_ident!("ACTION_{}", case.index()).to_token_stream()
        }
    }

    #[test]
    fn test_generate_single_case() {
        let case = MockCase { id: 0 };

        let generated = ActionSection::generate::<MockActionCodeGen>(vec![&case], "result");

        assert_tokens!(generated, {
            #[allow(unused_variables)]
            match result.case_id() {
                &fairjax_core::CaseId(0usize) => ACTION_0,
                _ => panic!(),
            }
        })
    }

    #[test]
    fn test_generate_3_cases() {
        let cases = vec![MockCase { id: 0 }, MockCase { id: 1 }, MockCase { id: 2 }];

        let generated = ActionSection::generate::<MockActionCodeGen>(
            cases.iter().map(|c| c as &dyn Case).collect(),
            "result",
        );

        assert_tokens!(generated, {
            #[allow(unused_variables)]
            match result.case_id() {
                &fairjax_core::CaseId(0usize) => ACTION_0,
                &fairjax_core::CaseId(1usize) => ACTION_1,
                &fairjax_core::CaseId(2usize) => ACTION_2,
                _ => panic!(),
            }
        })
    }
}
